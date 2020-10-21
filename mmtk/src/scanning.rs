use libc::c_void;
use std::mem::size_of;
use std::slice;

use mmtk::vm::Scanning;
use mmtk::{TransitiveClosure, TraceLocal, MutatorContext, Plan, SelectedPlan, ParallelCollector, SelectedMutator};
use mmtk::util::{ObjectReference, Address, SynchronizedCounter};
use mmtk::util::OpaquePointer;
use mmtk::vm::unboxed_size_constants::LOG_BYTES_IN_ADDRESS;
use mmtk::vm::unboxed_size_constants::BYTES_IN_ADDRESS;
use mmtk::vm::ObjectModel;
use mmtk::vm::ActivePlan;
use mmtk::vm::Collection;
use mmtk::scheduler::gc_works::ProcessEdgesWork;

use memory_manager_constants::*;
use java_header_constants::*;
use scan_sanity;
use entrypoint::*;
use JTOC_BASE;
use object_model::VMObjectModel;
use active_plan::VMActivePlan;
use collection::VMCollection;
use java_header::TIB_OFFSET;
use tib_layout_constants::TIB_TYPE_INDEX;
use JikesRVM;
use crate::SINGLETON;

static COUNTER: SynchronizedCounter = SynchronizedCounter::new(0);

#[derive(Default)]
pub struct VMScanning {}

const DUMP_REF: bool = false;

impl Scanning<JikesRVM> for VMScanning {
    const SINGLE_THREAD_MUTATOR_SCANNING: bool = false;
    fn scan_objects<W: ProcessEdgesWork<VM=JikesRVM>>(objects: &[ObjectReference]) {
        let mut edges = Vec::with_capacity(W::CAPACITY);
        for o in objects {
            self.scan_object_fields(*o, |edge| {
                edges.push(edge);
                if edges.len() >= W::CAPACITY {
                    SINGLETON.scheduler.closure_stage.add(W::new(edges, false));
                    edges = Vec::with_capacity(W::CAPACITY);
                }
            })
        }
        SINGLETON.scheduler.closure_stage.add(W::new(edges, false));
    }
    fn scan_thread_roots<W: ProcessEdgesWork<VM=JikesRVM>>() {
        unreachable!()
    }
    fn scan_thread_root<W: ProcessEdgesWork<VM=JikesRVM>>(mutator: &'static mut SelectedMutator<JikesRVM>) {
        let process_edges = create_process_edges_work::<W>;
        Self::compute_thread_roots(mutator.get_tls().to_address(), false, tls, process_edges as _);
    }
    fn scan_vm_specific_roots<W: ProcessEdgesWork<VM=JikesRVM>>() {
        SINGLETON.scheduler.prepare_stage.add(ScanStaticRoots::<W>::new());
        SINGLETON.scheduler.prepare_stage.add(ScanGlobalRoots::<W>::new());
        SINGLETON.scheduler.prepare_stage.add(ScanBootImageRoots::<W>::new());
    }
    fn scan_object<T: TransitiveClosure>(trace: &mut T, object: ObjectReference, tls: OpaquePointer) {
        if DUMP_REF {
            let obj_ptr = object.value();
            unsafe { jtoc_call!(DUMP_REF_METHOD_OFFSET, tls, obj_ptr); }
        }
        trace!("Getting reference array");
        let elt0_ptr: usize = unsafe {
            let rvm_type = VMObjectModel::load_rvm_type(object);
            (rvm_type + REFERENCE_OFFSETS_FIELD_OFFSET).load::<usize>()
        };
        trace!("elt0_ptr: {}", elt0_ptr);
        // In a primitive array this field points to a zero-length array.
        // In a reference array this field is null.
        // In a class with pointers, it contains the offsets of reference-containing instance fields
        if elt0_ptr == 0 {
            // object is a REFARRAY
            let length = VMObjectModel::get_array_length(object);
            for i in 0..length {
                trace.process_edge(object.to_address() + (i << LOG_BYTES_IN_ADDRESS));
            }
        } else {
            let len_ptr: usize = elt0_ptr - size_of::<isize>();
            let len = unsafe { *(len_ptr as *const isize) };
            let offsets = unsafe { slice::from_raw_parts(elt0_ptr as *const isize, len as usize) };

            for offset in offsets.iter() {
                trace.process_edge(object.to_address() + *offset);
            }
        }
    }

    fn reset_thread_counter() {
        COUNTER.reset();
    }

    fn notify_initial_thread_scan_complete(partial_scan: bool, tls: OpaquePointer) {
        if !partial_scan {
            unsafe {
                jtoc_call!(SNIP_OBSOLETE_COMPILED_METHODS_METHOD_OFFSET, tls);
            }
        }

        unsafe {
            VMActivePlan::mutator(tls).flush_remembered_sets();
        }
    }

    fn compute_static_roots<T: TraceLocal>(trace: &mut T, tls: OpaquePointer) {
        unreachable!()
    }

    fn compute_global_roots<T: TraceLocal>(trace: &mut T, tls: OpaquePointer) {
        unreachable!()
    }

    fn compute_thread_roots<T: TraceLocal>(trace: &mut T, tls: OpaquePointer) {
        // Self::compute_thread_roots(trace, false, tls)
        unreachable!()
    }

    fn compute_new_thread_roots<T: TraceLocal>(trace: &mut T, tls: OpaquePointer) {
        // Self::compute_thread_roots(trace, true, tls)
        unreachable!()
    }

    fn compute_bootimage_roots<T: TraceLocal>(trace: &mut T, tls: OpaquePointer) {
        super::scan_boot_image::scan_boot_image(trace, tls);
    }

    fn supports_return_barrier() -> bool {
        // FIXME: Really?
        cfg!(target_arch = "x86")
    }
}

pub extern fn create_process_edges_work<W: ProcessEdgesWork<VM=OpenJDK>>(ptr: *const Address, len: usize) {
    let mut buf = Vec::with_capacity(len);
    for i in 0..len {
        buf.push(unsafe { *ptr.add(i) });
    }
    SINGLETON.scheduler.closure_stage.add(W::new(buf, false));
}

impl VMScanning {
    fn scan_object_fields(object: ObjectReference, callback: impl FnMut(Address)) {
        trace!("Getting reference array");
        let elt0_ptr: usize = unsafe {
            let rvm_type = VMObjectModel::load_rvm_type(object);
            (rvm_type + REFERENCE_OFFSETS_FIELD_OFFSET).load::<usize>()
        };
        trace!("elt0_ptr: {}", elt0_ptr);
        // In a primitive array this field points to a zero-length array.
        // In a reference array this field is null.
        // In a class with pointers, it contains the offsets of reference-containing instance fields
        if elt0_ptr == 0 {
            // object is a REFARRAY
            let length = VMObjectModel::get_array_length(object);
            for i in 0..length {
                callback(object.to_address() + (i << LOG_BYTES_IN_ADDRESS));
            }
        } else {
            let len_ptr: usize = elt0_ptr - size_of::<isize>();
            let len = unsafe { *(len_ptr as *const isize) };
            let offsets = unsafe { slice::from_raw_parts(elt0_ptr as *const isize, len as usize) };

            for offset in offsets.iter() {
                callback(object.to_address() + *offset);
            }
        }
    }
    fn compute_thread_roots(thread: Address, new_roots_sufficient: bool, tls: OpaquePointer, process_edges: *const extern "C" fn(buf: *const Address, size: usize)) {
        if (thread + IS_COLLECTOR_FIELD_OFFSET).load::<bool>() {
            return;
        }
        let trace_ptr = trace as *mut T;
        let thread_usize = thread.as_usize();
        debug!("Calling JikesRVM to compute thread roots, thread_usize={:x}", thread_usize);
        jtoc_call!(SCAN_THREAD_METHOD_OFFSET, tls, thread_usize, trace_ptr,
            process_code_locations, new_roots_sufficient, process_edges);
        debug!("Returned from JikesRVM thread roots");
        // unsafe {
        //     let process_code_locations = MOVES_CODE;

        //     let num_threads =
        //         (JTOC_BASE + NUM_THREADS_FIELD_OFFSET).load::<usize>();

        //     loop {
        //         let thread_index = COUNTER.increment();
        //         if thread_index > num_threads {
        //             break;
        //         }

        //         let thread = VMCollection::thread_from_index(thread_index);

        //         if thread.is_zero() {
        //             continue;
        //         }

        //         if (thread + IS_COLLECTOR_FIELD_OFFSET).load::<bool>() {
        //             continue;
        //         }

        //         let trace_ptr = trace as *mut T;
        //         let thread_usize = thread.as_usize();
        //         debug!("Calling JikesRVM to compute thread roots, thread_usize={:x}", thread_usize);
        //         jtoc_call!(SCAN_THREAD_METHOD_OFFSET, tls, thread_usize, trace_ptr,
        //             process_code_locations, new_roots_sufficient, process_edges);
        //         debug!("Returned from JikesRVM thread roots");
        //     }
        // }
    }
    fn scan_global_roots(tls: OpaquePointer, callback: impl FnMut(Address)) {
        unsafe {
            let cc = VMActivePlan::collector(tls);

            let jni_functions = (JTOC_BASE + JNI_FUNCTIONS_FIELD_OFFSET).load::<Address>();
            trace!("jni_functions: {:?}", jni_functions);

            let threads = 1;//cc.parallel_worker_count();
            // @Intrinsic JNIFunctions.length()
            let mut size = (jni_functions + ARRAY_LENGTH_OFFSET).load::<usize>();
            trace!("size: {:?}", size);
            let mut chunk_size = size / threads;
            trace!("chunk_size: {:?}", chunk_size);
            let mut start = 0;
            trace!("start: {:?}", start);
            let mut end = size;
            trace!("end: {:?}", end);

            for i in start..end {
                let function_address_slot = jni_functions + (i << LOG_BYTES_IN_ADDRESS);
                if jtoc_call!(IMPLEMENTED_IN_JAVA_METHOD_OFFSET, tls, i) != 0 {
                    trace!("function implemented in java {:?}", function_address_slot);
                    callback(unsafe { Address::from_usize(function_address_slot) });
                } else {
                    // Function implemented as a C function, must not be
                    // scanned.
                }
            }

            let linkage_triplets = (JTOC_BASE + LINKAGE_TRIPLETS_FIELD_OFFSET).load::<Address>();
            if !linkage_triplets.is_zero() {
                for i in start..end {
                    callback(unsafe { Address::from_usize(linkage_triplets + i * 4) });
                }
            }

            let jni_global_refs = (JTOC_BASE + JNI_GLOBAL_REFS_FIELD2_OFFSET).load::<Address>();
            trace!("jni_global_refs address: {:?}", jni_global_refs);
            size = (jni_global_refs - 4).load::<usize>();
            trace!("jni_global_refs size: {:?}", size);
            chunk_size = size / threads;
            trace!("chunk_size: {:?}", chunk_size);
            start = 0;
            trace!("start: {:?}", start);
            end = size;
            trace!("end: {:?}", end);

            for i in start..end {
                callback(unsafe { Address::from_usize(jni_global_refs + (i << LOG_BYTES_IN_ADDRESS)) });
            }
        }
    }
}

pub struct ScanGlobalRoots<E: ProcessEdgesWork<VM=OpenJDK>>(PhantomData<E>);

impl <E: ProcessEdgesWork<VM=OpenJDK>> ScanGlobalRoots<E> {
    pub fn new() -> Self { Self(PhantomData) }
}

impl <E: ProcessEdgesWork<VM=OpenJDK>> GCWork<OpenJDK> for ScanGlobalRoots<E> {
    fn do_work(&mut self, worker: &mut GCWorker<OpenJDK>, mmtk: &'static MMTK<OpenJDK>) {
        let mut edges = Vec::with_capacity(E::CAPACITY);
        Scanning::scan_global_roots(OpaquePointer::UNINITIALIZED, |edge| {
            edges.push(edge);
            if edges.len() >= W::CAPACITY {
                SINGLETON.scheduler.closure_stage.add(W::new(edges, true));
                edges = Vec::with_capacity(W::CAPACITY);
            }
        });
        SINGLETON.scheduler.closure_stage.add(W::new(edges, true));
    }
}