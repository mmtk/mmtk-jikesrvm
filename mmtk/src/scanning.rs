use libc::c_void;
use std::mem::size_of;
use std::slice;

use mmtk::vm::Scanning;
use mmtk::*;
use mmtk::scheduler::*;
use mmtk::scheduler::gc_works::*;
use mmtk::util::{ObjectReference, Address, SynchronizedCounter};
use mmtk::util::OpaquePointer;
use crate::unboxed_size_constants::LOG_BYTES_IN_ADDRESS;
use crate::unboxed_size_constants::BYTES_IN_ADDRESS;
use mmtk::vm::ObjectModel;
use mmtk::vm::ActivePlan;
use mmtk::vm::Collection;
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
use crate::scan_statics::ScanStaticRoots;
use crate::scan_boot_image::ScanBootImageRoots;
use std::marker::PhantomData;
use std::mem;
use std::ops::Drop;

static COUNTER: SynchronizedCounter = SynchronizedCounter::new(0);

#[derive(Default)]
pub struct VMScanning {}

const DUMP_REF: bool = false;

// Fix the size of any `ProcessEdgesWork` to 4096. THe constant is also hard-coded in Java code.
pub const PROCESS_EDGES_WORK_SIZE: usize = 4096;

pub extern fn create_process_edges_work<W: ProcessEdgesWork<VM=JikesRVM>>(ptr: *mut Address, length: usize) -> *mut Address {
    debug_assert_eq!(W::CAPACITY, PROCESS_EDGES_WORK_SIZE);
    if !ptr.is_null() {
        let mut buf = unsafe { Vec::<Address>::from_raw_parts(ptr, length, W::CAPACITY) };
        SINGLETON.scheduler.work_buckets[WorkBucketId::Closure].add(W::new(buf, false));
    }
    let (ptr, length, capacity) =  Vec::with_capacity(W::CAPACITY).into_raw_parts();
    debug_assert_eq!(capacity, W::CAPACITY);
    ptr
}

impl Scanning<JikesRVM> for VMScanning {
    const SINGLE_THREAD_MUTATOR_SCANNING: bool = false;
    fn scan_objects<W: ProcessEdgesWork<VM=JikesRVM>>(objects: &[ObjectReference], worker: &mut GCWorker<JikesRVM>) {
        let mut closure = ObjectsClosure::<W>(Vec::with_capacity(W::CAPACITY), worker, PhantomData);
        for o in objects {
            Self::scan_object_fields(*o, &mut closure);
        }
    }
    fn scan_thread_roots<W: ProcessEdgesWork<VM=JikesRVM>>() {
        unreachable!()
    }
    fn scan_thread_root<W: ProcessEdgesWork<VM=JikesRVM>>(mutator: &'static mut Mutator<SelectedPlan<JikesRVM>>, tls: OpaquePointer) {
        let process_edges = create_process_edges_work::<W>;
        Self::compute_thread_roots(process_edges as _, false, mutator.get_tls(), tls);
    }
    fn scan_vm_specific_roots<W: ProcessEdgesWork<VM=JikesRVM>>() {
        let workers = SINGLETON.scheduler.num_workers();
        for i in 0..workers {
            SINGLETON.scheduler.work_buckets[WorkBucketId::Prepare].add(ScanStaticRoots::<W>::new(i, workers));
            SINGLETON.scheduler.work_buckets[WorkBucketId::Prepare].add(ScanBootImageRoots::<W>::new(i, workers));
            SINGLETON.scheduler.work_buckets[WorkBucketId::Prepare].add(ScanGlobalRoots::<W>::new(i, workers));
        }
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
}

struct ObjectsClosure<'a, E: ProcessEdgesWork<VM = JikesRVM>>(Vec<Address>, &'a mut GCWorker<JikesRVM>, PhantomData<E>);

impl<'a, E: ProcessEdgesWork<VM = JikesRVM>> ObjectsClosure<'a, E> {
    #[inline]
    fn process_edge(&mut self, slot: Address) {
        if self.0.len() == 0 {
            self.0.reserve(E::CAPACITY);
        }
        self.0.push(slot);
        if self.0.len() >= E::CAPACITY {
            let mut new_edges = Vec::new();
            mem::swap(&mut new_edges, &mut self.0);
            self.1
                .add_work(WorkBucketId::Closure, E::new(new_edges, false));
        }
    }
}

impl<'a, E: ProcessEdgesWork<VM = JikesRVM>> Drop for ObjectsClosure<'a, E> {
    #[inline]
    fn drop(&mut self) {
        let mut new_edges = Vec::new();
        mem::swap(&mut new_edges, &mut self.0);
        self.1
            .add_work(WorkBucketId::Closure, E::new(new_edges, false));
    }
}

impl VMScanning {
    fn scan_object_fields<'a, E: ProcessEdgesWork<VM=JikesRVM>>(object: ObjectReference, closure: &mut ObjectsClosure<'a, E>) {
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
    fn compute_thread_roots(process_edges: *mut extern fn (ptr: *mut Address, length: usize) -> *mut Address, new_roots_sufficient: bool, mutator: OpaquePointer, tls: OpaquePointer) {
        unsafe {
            let process_code_locations = MOVES_CODE;
            // `mutator` is a jikesrvm tls pointer. Transmute it to addess to read its internal information
            let thread = unsafe { mem::transmute::<OpaquePointer, Address>(mutator) };
            debug_assert!(!thread.is_zero());
            if (thread + IS_COLLECTOR_FIELD_OFFSET).load::<bool>() {
                return;
            }
            let thread_usize = thread.as_usize();
            debug!("Calling JikesRVM to compute thread roots, thread_usize={:x}", thread_usize);
            jtoc_call!(SCAN_THREAD_METHOD_OFFSET, tls, thread_usize, process_edges,
                process_code_locations, new_roots_sufficient);
            debug!("Returned from JikesRVM thread roots");
        }
    }
    fn scan_global_roots(tls: OpaquePointer, subwork_id: usize, total_subworks: usize, mut callback: impl FnMut(Address)) {
        unsafe {
            // let cc = VMActivePlan::collector(tls);

            let jni_functions = (JTOC_BASE + JNI_FUNCTIONS_FIELD_OFFSET).load::<Address>();
            trace!("jni_functions: {:?}", jni_functions);

            let threads = total_subworks;
            // @Intrinsic JNIFunctions.length()
            let mut size = (jni_functions + ARRAY_LENGTH_OFFSET).load::<usize>();
            trace!("size: {:?}", size);
            let mut chunk_size = size / threads;
            trace!("chunk_size: {:?}", chunk_size);
            let mut start = subwork_id * chunk_size;
            trace!("start: {:?}", start);
            let mut end = if subwork_id + 1 == threads {
                size
            } else {
                threads * chunk_size
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
            start = 0;
            trace!("start: {:?}", start);
            end = size;
            trace!("end: {:?}", end);

            for i in start..end {
                callback(jni_global_refs + (i << LOG_BYTES_IN_ADDRESS));
            }
        }
    }
}

pub struct ScanGlobalRoots<E: ProcessEdgesWork<VM=JikesRVM>>(usize, usize, PhantomData<E>);

impl <E: ProcessEdgesWork<VM=JikesRVM>> ScanGlobalRoots<E> {
    pub fn new(subwork_id: usize, total_subworks: usize) -> Self {
        Self(subwork_id, total_subworks, PhantomData)
    }
}

impl <E: ProcessEdgesWork<VM=JikesRVM>> GCWork<JikesRVM> for ScanGlobalRoots<E> {
    fn do_work(&mut self, worker: &mut GCWorker<JikesRVM>, mmtk: &'static MMTK<JikesRVM>) {
        let mut edges = Vec::with_capacity(E::CAPACITY);
        VMScanning::scan_global_roots(worker.tls, self.0, self.1, |edge| {
            edges.push(edge);
            if edges.len() >= E::CAPACITY {
                let mut new_edges = Vec::with_capacity(E::CAPACITY);
                mem::swap(&mut new_edges, &mut edges);
                SINGLETON.scheduler.work_buckets[WorkBucketId::Closure].add(E::new(new_edges, true));
            }
        });
        SINGLETON.scheduler.work_buckets[WorkBucketId::Closure].add(E::new(edges, true));
    }
}