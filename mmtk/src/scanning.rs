use std::convert::TryFrom;
use std::mem::size_of;
use std::slice;

use crate::jikesrvm_calls;
use crate::object_model::JikesObj;
use mmtk::vm::ObjectTracer;
use mmtk::vm::ObjectTracerContext;
// use crate::scan_boot_image::ScanBootImageRoots;
use crate::active_plan::VMActivePlan;
use crate::entrypoint::*;
use crate::java_header_constants::*;
use crate::memory_manager_constants::*;
use crate::scan_statics::ScanStaticRoots;
use crate::unboxed_size_constants::LOG_BYTES_IN_ADDRESS;
use crate::JikesRVM;
use crate::JikesRVMSlot;
use crate::JTOC_BASE;
use crate::SINGLETON;
use mmtk::memory_manager;
use mmtk::scheduler::*;
use mmtk::util::opaque_pointer::*;
use mmtk::util::{Address, ObjectReference};
use mmtk::vm::ActivePlan;
use mmtk::vm::RootsWorkFactory;
use mmtk::vm::Scanning;
use mmtk::vm::SlotVisitor;
use mmtk::MMTK;
use mmtk::*;
use std::mem;

#[derive(Default)]
pub struct VMScanning {}

const DUMP_REF: bool = false;

// The mmtk-core no longer exposes the capacity of work packets through the API.
// This value is chosen by the mmtk-jikesrvm binding.  The Rust and the Java code agree upon this buffer size.
// See the Java constant `RustScanThread.SLOTS_BUFFER_CAPACITY`.
pub(crate) const SLOTS_BUFFER_CAPACITY: usize = 4096;

extern "C" fn report_slots_and_renew_buffer<F: RootsWorkFactory<JikesRVMSlot>>(
    ptr: *mut JikesRVMSlot,
    length: usize,
    factory: *mut libc::c_void,
) -> *mut Address {
    if !ptr.is_null() {
        let buf =
            unsafe { Vec::<JikesRVMSlot>::from_raw_parts(ptr, length, SLOTS_BUFFER_CAPACITY) };
        let factory: &mut F = unsafe { &mut *(factory as *mut F) };
        factory.create_process_roots_work(buf);
    }
    let (ptr, _, capacity) = {
        // TODO: Use Vec::into_raw_parts() when the method is available.
        use std::mem::ManuallyDrop;
        let new_vec = Vec::with_capacity(SLOTS_BUFFER_CAPACITY);
        let mut me = ManuallyDrop::new(new_vec);
        (me.as_mut_ptr(), me.len(), me.capacity())
    };
    debug_assert_eq!(capacity, SLOTS_BUFFER_CAPACITY);
    ptr
}

extern "C" fn trace_object_callback_for_jikesrvm<T: ObjectTracer>(
    tracer_ptr: *mut libc::c_void,
    object: JikesObj,
) -> JikesObj {
    debug_assert!(!tracer_ptr.is_null());
    let tracer: &mut T = unsafe { &mut *(tracer_ptr as *mut T) };
    let object = ObjectReference::try_from(object).unwrap();
    tracer.trace_object(object).into()
}

impl Scanning<JikesRVM> for VMScanning {
    fn scan_roots_in_mutator_thread(
        tls: VMWorkerThread,
        mutator: &'static mut Mutator<JikesRVM>,
        mut factory: impl RootsWorkFactory<JikesRVMSlot>,
    ) {
        Self::compute_thread_roots(&mut factory, false, mutator.get_tls(), tls);
    }
    fn scan_vm_specific_roots(_tls: VMWorkerThread, factory: impl RootsWorkFactory<JikesRVMSlot>) {
        let workers = memory_manager::num_of_workers(&SINGLETON);
        for i in 0..workers {
            memory_manager::add_work_packets(
                &SINGLETON,
                WorkBucketStage::Prepare,
                vec![
                    Box::new(ScanStaticRoots::new(factory.clone(), i, workers)) as _,
                    // Box::new(ScanBootImageRoots::new(factory.clone(), i, workers)) as _,
                    Box::new(ScanGlobalRoots::new(factory.clone(), i, workers)) as _,
                ],
            );
        }
    }
    fn scan_object<EV: SlotVisitor<JikesRVMSlot>>(
        tls: VMWorkerThread,
        object: ObjectReference,
        slot_visitor: &mut EV,
    ) {
        let jikes_obj = JikesObj::from(object);
        if DUMP_REF {
            jikesrvm_calls::dump_ref(tls, jikes_obj);
        }
        trace!("Getting reference array");
        let elt0_ptr: usize = {
            let rvm_type = jikes_obj.load_rvm_type();
            rvm_type.reference_offsets()
        };
        trace!("elt0_ptr: {}", elt0_ptr);
        // In a primitive array this field points to a zero-length array.
        // In a reference array this field is null.
        // In a class with pointers, it contains the offsets of reference-containing instance fields
        if elt0_ptr == 0 {
            // object is a REFARRAY
            let length = jikes_obj.get_array_length();
            for i in 0..length {
                slot_visitor.visit_slot(JikesRVMSlot::from_address(
                    jikes_obj.to_address() + (i << LOG_BYTES_IN_ADDRESS),
                ));
            }
        } else {
            let len_ptr: usize = elt0_ptr - size_of::<isize>();
            let len = unsafe { *(len_ptr as *const isize) };
            let offsets = unsafe { slice::from_raw_parts(elt0_ptr as *const isize, len as usize) };

            for offset in offsets.iter() {
                slot_visitor
                    .visit_slot(JikesRVMSlot::from_address(jikes_obj.to_address() + *offset));
            }
        }
    }

    fn notify_initial_thread_scan_complete(partial_scan: bool, tls: VMWorkerThread) {
        if !partial_scan {
            jikesrvm_calls::snip_obsolete_compiled_methods(tls);
        }

        unsafe {
            // FIXME: in the original JikesRVM code, it is indeed calling mutator.flush_remembered_sets() from a collector thread.
            // I guess it is because the above method (snipObsoleteCompliedMethods()) may mutate objects, and those are caught by barriers,
            // and we need to flush remembered sets before scanning objects. So it is okay.
            // However, I am not sure if this is still correct for Rust MMTk.
            // We need to be really careful when implementing a Rust MMTk plan with barriers. Basically we need to check:
            // 1. If a worker thread mutates objects, are those caught by barriers?
            // 2. Can we call flush_remembered_sets() on a worker thread, and will that correctly flush remembered sets from the above?
            // 3. Do we always wait for all thread/stack scanning to finish before starting scanning object?
            //    We said we want to scan objects as soon as one thread finishes the scanning.
            let mutator: VMMutatorThread = std::mem::transmute(tls);
            VMActivePlan::mutator(mutator).flush_remembered_sets();
        }
    }

    // fn compute_static_roots<T: TraceLocal>(trace: &mut T, tls: OpaquePointer) {
    //     unreachable!()
    // }

    // fn compute_global_roots<T: TraceLocal>(trace: &mut T, tls: OpaquePointer) {
    //     unreachable!()
    // }

    // fn compute_thread_roots<T: TraceLocal>(trace: &mut T, tls: OpaquePointer) {
    //     // Self::compute_thread_roots(trace, false, tls)
    //     unreachable!()
    // }

    // fn compute_new_thread_roots<T: TraceLocal>(trace: &mut T, tls: OpaquePointer) {
    //     // Self::compute_thread_roots(trace, true, tls)
    //     unreachable!()
    // }

    // fn compute_bootimage_roots<T: TraceLocal>(trace: &mut T, tls: OpaquePointer) {
    //     super::scan_boot_image::scan_boot_image(trace, tls);
    // }

    fn supports_return_barrier() -> bool {
        // FIXME: Really?
        cfg!(target_arch = "x86")
    }

    fn prepare_for_roots_re_scanning() {
        // I guess we do not need to do anything special. However I will leave it as unimplemented for now.
        unimplemented!()
    }

    fn process_weak_refs(
        worker: &mut GCWorker<JikesRVM>,
        tracer_context: impl ObjectTracerContext<JikesRVM>,
    ) -> bool {
        process_weak_refs_inner(worker, tracer_context)
    }

    fn forward_weak_refs(
        worker: &mut GCWorker<JikesRVM>,
        tracer_context: impl ObjectTracerContext<JikesRVM>,
    ) {
        forward_weak_refs_inner(worker, tracer_context)
    }
}

fn forward_weak_refs_inner<C>(worker: &mut GCWorker<JikesRVM>, tracer_context: C)
where
    C: ObjectTracerContext<JikesRVM>,
{
    let tls = worker.tls;

    let is_nursery = SINGLETON
        .get_plan()
        .generational()
        .map_or(false, |plan| plan.is_current_gc_nursery());

    tracer_context.with_tracer(worker, |tracer| {
        jikesrvm_calls::do_reference_processing_helper_forward(
            tls,
            trace_object_callback_for_jikesrvm::<C::TracerType>,
            tracer as *mut _ as *mut libc::c_void,
            is_nursery,
        )
    });
}

fn process_weak_refs_inner<C>(worker: &mut GCWorker<JikesRVM>, tracer_context: C) -> bool
where
    C: ObjectTracerContext<JikesRVM>,
{
    let tls = worker.tls;

    let is_nursery = SINGLETON
        .get_plan()
        .generational()
        .map_or(false, |plan| plan.is_current_gc_nursery());

    let need_retain = SINGLETON.is_emergency_collection();

    tracer_context.with_tracer(worker, |tracer| {
        jikesrvm_calls::do_reference_processing_helper_scan(
            tls,
            trace_object_callback_for_jikesrvm::<C::TracerType>,
            tracer as *mut _ as *mut libc::c_void,
            is_nursery,
            need_retain,
        )
    })
}

impl VMScanning {
    fn compute_thread_roots<F: RootsWorkFactory<JikesRVMSlot>>(
        factory: &mut F,
        new_roots_sufficient: bool,
        mutator: VMMutatorThread,
        tls: VMWorkerThread,
    ) {
        unsafe {
            let process_code_locations = MOVES_CODE;
            // `mutator` is a jikesrvm tls pointer. Transmute it to addess to read its internal information
            let thread = mem::transmute::<VMMutatorThread, Address>(mutator);
            debug_assert!(!thread.is_zero());
            if (thread + IS_COLLECTOR_FIELD_OFFSET).load::<bool>() {
                return;
            }
            debug!("Calling JikesRVM to compute thread roots, thread={thread}");
            jikesrvm_calls::scan_thread(
                tls,
                mutator,
                report_slots_and_renew_buffer::<F>,
                factory as *mut F as *mut libc::c_void,
                process_code_locations,
                new_roots_sufficient,
            );
            debug!("Returned from JikesRVM thread roots");
        }
    }
    fn scan_global_roots(
        tls: VMWorkerThread,
        subwork_id: usize,
        total_subwork: usize,
        mut callback: impl FnMut(JikesRVMSlot),
    ) {
        unsafe {
            // let cc = VMActivePlan::collector(tls);

            let jni_functions = (JTOC_BASE + JNI_FUNCTIONS_FIELD_OFFSET).load::<Address>();
            trace!("jni_functions: {:?}", jni_functions);

            let threads = total_subwork;
            // @Intrinsic JNIFunctions.length()
            let mut size = (jni_functions + ARRAY_LENGTH_OFFSET).load::<usize>();
            trace!("size: {:?}", size);
            let mut chunk_size = size / threads;
            trace!("chunk_size: {:?}", chunk_size);
            let start = subwork_id * chunk_size;
            trace!("start: {:?}", start);
            let end = if subwork_id + 1 == threads {
                size
            } else {
                (subwork_id + 1) * chunk_size
            };
            trace!("end: {:?}", end);

            for i in start..end {
                let function_address_slot = jni_functions + (i << LOG_BYTES_IN_ADDRESS);
                if jikesrvm_calls::implemented_in_java(tls, i) {
                    trace!("function implemented in java {:?}", function_address_slot);
                    callback(JikesRVMSlot::from_address(function_address_slot));
                } else {
                    // Function implemented as a C function, must not be
                    // scanned.
                }
            }

            let linkage_triplets = (JTOC_BASE + LINKAGE_TRIPLETS_FIELD_OFFSET).load::<Address>();
            if !linkage_triplets.is_zero() {
                for i in start..end {
                    callback(JikesRVMSlot::from_address(linkage_triplets + i * 4));
                }
            }

            let jni_global_refs = (JTOC_BASE + JNI_GLOBAL_REFS_FIELD2_OFFSET).load::<Address>();
            trace!("jni_global_refs address: {:?}", jni_global_refs);
            size = (jni_global_refs - 4).load::<usize>();
            trace!("jni_global_refs size: {:?}", size);
            chunk_size = size / threads;
            trace!("chunk_size: {:?}", chunk_size);
            let start = subwork_id * chunk_size;
            trace!("start: {:?}", start);
            let end = if subwork_id + 1 == threads {
                size
            } else {
                (subwork_id + 1) * chunk_size
            };
            trace!("end: {:?}", end);

            for i in start..end {
                callback(JikesRVMSlot::from_address(
                    jni_global_refs + (i << LOG_BYTES_IN_ADDRESS),
                ));
            }
        }
    }
}

pub struct ScanGlobalRoots<F: RootsWorkFactory<JikesRVMSlot>> {
    factory: F,
    subwork_id: usize,
    total_subwork: usize,
}

impl<F: RootsWorkFactory<JikesRVMSlot>> ScanGlobalRoots<F> {
    pub fn new(factory: F, subwork_id: usize, total_subwork: usize) -> Self {
        Self {
            factory,
            subwork_id,
            total_subwork,
        }
    }
}

impl<F: RootsWorkFactory<JikesRVMSlot>> GCWork<JikesRVM> for ScanGlobalRoots<F> {
    fn do_work(&mut self, worker: &mut GCWorker<JikesRVM>, _mmtk: &'static MMTK<JikesRVM>) {
        let mut slots = Vec::with_capacity(SLOTS_BUFFER_CAPACITY);
        VMScanning::scan_global_roots(worker.tls, self.subwork_id, self.total_subwork, |slot| {
            slots.push(slot);
            if slots.len() >= SLOTS_BUFFER_CAPACITY {
                let new_slots = mem::replace(&mut slots, Vec::with_capacity(SLOTS_BUFFER_CAPACITY));
                self.factory.create_process_roots_work(new_slots);
            }
        });
        if !slots.is_empty() {
            self.factory.create_process_roots_work(slots);
        }
    }
}
