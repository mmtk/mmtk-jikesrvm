use std::arch::asm;
use std::mem::size_of;
use std::slice;

// use crate::scan_boot_image::ScanBootImageRoots;
use crate::scan_statics::ScanStaticRoots;
use crate::unboxed_size_constants::LOG_BYTES_IN_ADDRESS;
use crate::SINGLETON;
use active_plan::VMActivePlan;
use entrypoint::*;
use java_header_constants::*;
use memory_manager_constants::*;
use mmtk::memory_manager;
use mmtk::scheduler::*;
use mmtk::util::opaque_pointer::*;
use mmtk::util::{Address, ObjectReference};
use mmtk::vm::ActivePlan;
use mmtk::vm::EdgeVisitor;
use mmtk::vm::RootsWorkFactory;
use mmtk::vm::Scanning;
use mmtk::*;
use object_model::VMObjectModel;
use std::mem;
use JikesRVM;
use JTOC_BASE;

#[derive(Default)]
pub struct VMScanning {}

const DUMP_REF: bool = false;

/// This allows JikesRVM Java code to call the dynamic methods of RootsWorkFactory.
/// Note that we cannot pass a `&dyn` directly, because it is not pointer-sized.
pub(crate) struct ScopedDynamicFactoryInvoker<'f> {
    pub factory: &'f dyn RootsWorkFactory,
}

impl<'f> ScopedDynamicFactoryInvoker<'f> {
    pub(crate) fn new(factory: &'f dyn RootsWorkFactory) -> Self {
        Self { factory }
    }

    pub(crate) fn invoke(&mut self, edges: Vec<Address>) {
        self.factory.create_process_edge_roots_work(edges);
    }

    pub(crate) fn as_self_ptr(&self) -> *mut libc::c_void {
        unsafe { std::mem::transmute(self) }
    }
}

// The mmtk-core no longer exposes the capacity of work packets through the API.
// This value is chosen by the mmtk-jikesrvm binding.  The Rust and the Java code agree upon this buffer size.
// See the Java constant `RustScanThread.EDGES_BUFFER_CAPACITY`.
pub(crate) const EDGES_BUFFER_CAPACITY: usize = 4096;

extern "C" fn report_edges_and_renew_buffer(
    ptr: *mut Address,
    length: usize,
    invoker_ptr: *mut libc::c_void,
) -> *mut Address {
    if !ptr.is_null() {
        let buf = unsafe { Vec::<Address>::from_raw_parts(ptr, length, EDGES_BUFFER_CAPACITY) };
        let invoker: &mut ScopedDynamicFactoryInvoker<'static> =
            unsafe { &mut *(invoker_ptr as *mut ScopedDynamicFactoryInvoker) };
        invoker.invoke(buf);
    }
    let (ptr, _, capacity) = {
        // TODO: Use Vec::into_raw_parts() when the method is available.
        use std::mem::ManuallyDrop;
        let new_vec = Vec::with_capacity(EDGES_BUFFER_CAPACITY);
        let mut me = ManuallyDrop::new(new_vec);
        (me.as_mut_ptr(), me.len(), me.capacity())
    };
    debug_assert_eq!(capacity, EDGES_BUFFER_CAPACITY);
    ptr
}

impl Scanning<JikesRVM> for VMScanning {
    const SINGLE_THREAD_MUTATOR_SCANNING: bool = false;
    fn scan_thread_roots(_tls: VMWorkerThread, _factory: Box<dyn RootsWorkFactory>) {
        unreachable!()
    }
    fn scan_thread_root(
        tls: VMWorkerThread,
        mutator: &'static mut Mutator<JikesRVM>,
        factory: Box<dyn RootsWorkFactory>,
    ) {
        Self::compute_thread_roots(factory.as_ref(), false, mutator.get_tls(), tls);
    }
    fn scan_vm_specific_roots(_tls: VMWorkerThread, factory: Box<dyn RootsWorkFactory>) {
        let workers = memory_manager::num_of_workers(&SINGLETON);
        for i in 0..workers {
            memory_manager::add_work_packets(
                &SINGLETON,
                WorkBucketStage::Prepare,
                vec![
                    Box::new(ScanStaticRoots::new(factory.fork(), i, workers)) as _,
                    // Box::new(ScanBootImageRoots::new(factory.fork(), i, workers)) as _,
                    Box::new(ScanGlobalRoots::new(factory.fork(), i, workers)) as _,
                ],
            );
        }
    }
    fn scan_object<EV: EdgeVisitor>(
        tls: VMWorkerThread,
        object: ObjectReference,
        edge_visitor: &mut EV,
    ) {
        if DUMP_REF {
            let obj_ptr = object.value();
            unsafe {
                jtoc_call!(DUMP_REF_METHOD_OFFSET, tls, obj_ptr);
            }
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
                edge_visitor.visit_edge(object.to_address() + (i << LOG_BYTES_IN_ADDRESS));
            }
        } else {
            let len_ptr: usize = elt0_ptr - size_of::<isize>();
            let len = unsafe { *(len_ptr as *const isize) };
            let offsets = unsafe { slice::from_raw_parts(elt0_ptr as *const isize, len as usize) };

            for offset in offsets.iter() {
                edge_visitor.visit_edge(object.to_address() + *offset);
            }
        }
    }

    fn notify_initial_thread_scan_complete(partial_scan: bool, tls: VMWorkerThread) {
        if !partial_scan {
            unsafe {
                jtoc_call!(SNIP_OBSOLETE_COMPILED_METHODS_METHOD_OFFSET, tls);
            }
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
}

impl VMScanning {
    fn compute_thread_roots(
        factory: &dyn RootsWorkFactory,
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
            let thread_usize = thread.as_usize();
            debug!(
                "Calling JikesRVM to compute thread roots, thread_usize={:x}",
                thread_usize
            );
            let invoker = ScopedDynamicFactoryInvoker::new(factory);
            jtoc_call!(
                SCAN_THREAD_METHOD_OFFSET,
                tls,
                thread_usize,
                report_edges_and_renew_buffer,
                invoker.as_self_ptr(),
                process_code_locations as i32,
                new_roots_sufficient as i32
            );
            debug!("Returned from JikesRVM thread roots");
        }
    }
    fn scan_global_roots(
        tls: VMWorkerThread,
        subwork_id: usize,
        total_subwork: usize,
        mut callback: impl FnMut(Address),
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
                if jtoc_call!(IMPLEMENTED_IN_JAVA_METHOD_OFFSET, tls, i) != 0 {
                    trace!("function implemented in java {:?}", function_address_slot);
                    callback(function_address_slot);
                } else {
                    // Function implemented as a C function, must not be
                    // scanned.
                }
            }

            let linkage_triplets = (JTOC_BASE + LINKAGE_TRIPLETS_FIELD_OFFSET).load::<Address>();
            if !linkage_triplets.is_zero() {
                for i in start..end {
                    callback(linkage_triplets + i * 4);
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
                callback(jni_global_refs + (i << LOG_BYTES_IN_ADDRESS));
            }
        }
    }
}

pub struct ScanGlobalRoots {
    factory: Box<dyn RootsWorkFactory>,
    subwork_id: usize,
    total_subwork: usize,
}

impl ScanGlobalRoots {
    pub fn new(
        factory: Box<dyn RootsWorkFactory>,
        subwork_id: usize,
        total_subwork: usize,
    ) -> Self {
        Self {
            factory,
            subwork_id,
            total_subwork,
        }
    }
}

impl GCWork<JikesRVM> for ScanGlobalRoots {
    fn do_work(&mut self, worker: &mut GCWorker<JikesRVM>, _mmtk: &'static MMTK<JikesRVM>) {
        let mut edges = Vec::with_capacity(EDGES_BUFFER_CAPACITY);
        VMScanning::scan_global_roots(worker.tls, self.subwork_id, self.total_subwork, |edge| {
            edges.push(edge);
            if edges.len() >= EDGES_BUFFER_CAPACITY {
                let new_edges = mem::replace(&mut edges, Vec::with_capacity(EDGES_BUFFER_CAPACITY));
                self.factory.create_process_edge_roots_work(new_edges);
            }
        });
        if !edges.is_empty() {
            self.factory.create_process_edge_roots_work(edges);
        }
    }
}
