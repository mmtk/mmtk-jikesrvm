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
use mmtk::vm::Scanning;
use mmtk::*;
use object_model::VMObjectModel;
use std::marker::PhantomData;
use std::mem;
use std::ops::Drop;
use JikesRVM;
use JTOC_BASE;

#[derive(Default)]
pub struct VMScanning {}

const DUMP_REF: bool = false;

// Fix the size of any `ProcessEdgesWork` to 4096. THe constant is also hard-coded in Java code.
pub const PROCESS_EDGES_WORK_SIZE: usize = 4096;

pub(crate) extern "C" fn create_process_edges_work<W: ProcessEdgesWork<VM = JikesRVM>>(
    ptr: *mut Address,
    length: usize,
) -> *mut Address {
    debug_assert_eq!(W::CAPACITY, PROCESS_EDGES_WORK_SIZE);
    if !ptr.is_null() {
        let buf = unsafe { Vec::<Address>::from_raw_parts(ptr, length, W::CAPACITY) };
        memory_manager::add_work_packet(
            &SINGLETON,
            WorkBucketStage::Closure,
            W::new(buf, false, &SINGLETON),
        );
    }
    let (ptr, _length, capacity) = Vec::with_capacity(W::CAPACITY).into_raw_parts();
    debug_assert_eq!(capacity, W::CAPACITY);
    ptr
}

impl Scanning<JikesRVM> for VMScanning {
    const SINGLE_THREAD_MUTATOR_SCANNING: bool = false;
    fn scan_objects<W: ProcessEdgesWork<VM = JikesRVM>>(
        objects: &[ObjectReference],
        worker: &mut GCWorker<JikesRVM>,
    ) {
        let mut closure = ObjectsClosure::<W>(Vec::with_capacity(W::CAPACITY), worker, PhantomData);
        for o in objects {
            Self::scan_object_fields(*o, &mut closure);
        }
    }
    fn scan_thread_roots<W: ProcessEdgesWork<VM = JikesRVM>>() {
        unreachable!()
    }
    fn scan_thread_root<W: ProcessEdgesWork<VM = JikesRVM>>(
        mutator: &'static mut Mutator<JikesRVM>,
        tls: VMWorkerThread,
    ) {
        let process_edges = create_process_edges_work::<W>;
        Self::compute_thread_roots(process_edges as _, false, mutator.get_tls(), tls);
    }
    fn scan_vm_specific_roots<W: ProcessEdgesWork<VM = JikesRVM>>() {
        let workers = memory_manager::num_of_workers(&SINGLETON);
        for i in 0..workers {
            memory_manager::add_work_packet(
                &SINGLETON,
                WorkBucketStage::Prepare,
                ScanStaticRoots::<W>::new(i, workers),
            );
            // SINGLETON.scheduler.work_buckets[WorkBucketStage::Prepare]
            //     .add(ScanBootImageRoots::<W>::new(i, workers));
            memory_manager::add_work_packet(
                &SINGLETON,
                WorkBucketStage::Prepare,
                ScanGlobalRoots::<W>::new(i, workers),
            );
        }
    }
    fn scan_object<T: TransitiveClosure>(
        trace: &mut T,
        object: ObjectReference,
        tls: VMWorkerThread,
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

struct ObjectsClosure<'a, E: ProcessEdgesWork<VM = JikesRVM>>(
    Vec<Address>,
    &'a mut GCWorker<JikesRVM>,
    PhantomData<E>,
);

impl<'a, E: ProcessEdgesWork<VM = JikesRVM>> ObjectsClosure<'a, E> {
    #[inline]
    fn process_edge(&mut self, slot: Address) {
        if self.0.is_empty() {
            self.0.reserve(E::CAPACITY);
        }
        self.0.push(slot);
        if self.0.len() >= E::CAPACITY {
            let mut new_edges = Vec::new();
            mem::swap(&mut new_edges, &mut self.0);
            self.1.add_work(
                WorkBucketStage::Closure,
                E::new(new_edges, false, &SINGLETON),
            );
        }
    }
}

impl<'a, E: ProcessEdgesWork<VM = JikesRVM>> Drop for ObjectsClosure<'a, E> {
    #[inline]
    fn drop(&mut self) {
        let mut new_edges = Vec::new();
        mem::swap(&mut new_edges, &mut self.0);
        self.1.add_work(
            WorkBucketStage::Closure,
            E::new(new_edges, false, &SINGLETON),
        );
    }
}

impl VMScanning {
    fn scan_object_fields<E: ProcessEdgesWork<VM = JikesRVM>>(
        object: ObjectReference,
        closure: &mut ObjectsClosure<E>,
    ) {
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
                closure.process_edge(object.to_address() + (i << LOG_BYTES_IN_ADDRESS));
            }
        } else {
            let len_ptr: usize = elt0_ptr - size_of::<isize>();
            let len = unsafe { *(len_ptr as *const isize) };
            let offsets = unsafe { slice::from_raw_parts(elt0_ptr as *const isize, len as usize) };

            for offset in offsets.iter() {
                closure.process_edge(object.to_address() + *offset);
            }
        }
    }
    fn compute_thread_roots(
        process_edges: *mut extern "C" fn(ptr: *mut Address, length: usize) -> *mut Address,
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
            jtoc_call!(
                SCAN_THREAD_METHOD_OFFSET,
                tls,
                thread_usize,
                process_edges,
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

pub struct ScanGlobalRoots<E: ProcessEdgesWork<VM = JikesRVM>>(usize, usize, PhantomData<E>);

impl<E: ProcessEdgesWork<VM = JikesRVM>> ScanGlobalRoots<E> {
    pub fn new(subwork_id: usize, total_subwork: usize) -> Self {
        Self(subwork_id, total_subwork, PhantomData)
    }
}

impl<E: ProcessEdgesWork<VM = JikesRVM>> GCWork<JikesRVM> for ScanGlobalRoots<E> {
    fn do_work(&mut self, worker: &mut GCWorker<JikesRVM>, _mmtk: &'static MMTK<JikesRVM>) {
        let mut edges = Vec::with_capacity(E::CAPACITY);
        VMScanning::scan_global_roots(worker.tls, self.0, self.1, |edge| {
            edges.push(edge);
            if edges.len() >= E::CAPACITY {
                let mut new_edges = Vec::with_capacity(E::CAPACITY);
                mem::swap(&mut new_edges, &mut edges);
                memory_manager::add_work_packet(
                    &SINGLETON,
                    WorkBucketStage::Closure,
                    E::new(new_edges, true, &SINGLETON),
                )
            }
        });
        memory_manager::add_work_packet(
            &SINGLETON,
            WorkBucketStage::Closure,
            E::new(edges, true, &SINGLETON),
        );
    }
}
