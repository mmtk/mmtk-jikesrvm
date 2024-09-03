//! This module contains wrappers for concrete calls into JikesRVM.  This ensures those functions
//! are always called with the right parameter types, especially with respect to the difference
//! between the MMTk-level `ObjectReference` and `JikesObj` which is the JikesRVM-level
//! `ObjectReference`.

#[macro_use]
pub mod helpers;

// Keep an eye on the imports.  This module should not use `mmtk::util::address::ObjectReference`.
use mmtk::{
    scheduler::GCWorker,
    util::{Address, OpaquePointer, VMMutatorThread, VMThread, VMWorkerThread},
};

use self::helpers::{FromAsmResult, ToAsmArg};
use crate::{entrypoint::*, object_model::JikesObj, JikesRVM, JikesRVMSlot};

pub fn block_all_mutators_for_gc(tls: VMWorkerThread) {
    unsafe {
        jtoc_call!(BLOCK_ALL_MUTATORS_FOR_GC_METHOD_OFFSET, tls);
    }
}

pub fn unblock_all_mutators_for_gc(tls: VMWorkerThread) {
    unsafe {
        jtoc_call!(UNBLOCK_ALL_MUTATORS_FOR_GC_METHOD_OFFSET, tls);
    }
}

pub fn prepare_mutator(tls: VMWorkerThread, mutator_tls: VMMutatorThread) {
    unsafe {
        // asm! is not smart enough to figure out VMMutatorThread has repr(transparent) and
        // therefore the same representation as a pointer.
        let mutator_tls_usize = std::mem::transmute::<_, usize>(mutator_tls);
        jtoc_call!(PREPARE_MUTATOR_METHOD_OFFSET, tls, mutator_tls_usize);
    }
}

pub fn block_for_gc(tls: VMMutatorThread) {
    unsafe {
        jtoc_call!(BLOCK_FOR_GC_METHOD_OFFSET, tls);
    }
}

pub fn spawn_collector_thread(tls: VMThread, worker_instance: *mut GCWorker<JikesRVM>) {
    unsafe {
        jtoc_call!(SPAWN_COLLECTOR_THREAD_METHOD_OFFSET, tls, worker_instance);
    }
}

pub fn out_of_memory(tls: VMThread) {
    unsafe {
        jtoc_call!(OUT_OF_MEMORY_METHOD_OFFSET, tls);
    }
}

pub fn schedule_finalizer(tls: VMWorkerThread) {
    unsafe {
        jtoc_call!(SCHEDULE_FINALIZER_METHOD_OFFSET, tls);
    }
}

pub fn mm_entrypoint_test(tls: OpaquePointer, a: usize, b: usize, c: usize, d: usize) -> usize {
    unsafe { jtoc_call!(MM_ENTRYPOINT_TEST_METHOD_OFFSET, tls, a, b, c, d) }
}

pub fn enqueue_reference(tls: VMWorkerThread, reff: JikesObj) {
    let reff = reff.to_jtoc_call_arg();
    unsafe {
        jtoc_call!(ENQUEUE_REFERENCE_METHOD_OFFSET, tls, reff);
    }
}

pub fn scan_boot_image(tls: OpaquePointer, root: *const usize) {
    unsafe {
        jtoc_call!(SCAN_BOOT_IMAGE_METHOD_OFFSET, tls, root);
    }
}

pub fn get_number_of_reference_slots(tls: VMWorkerThread) -> usize {
    unsafe { jtoc_call!(GET_NUMBER_OF_REFERENCE_SLOTS_METHOD_OFFSET, tls) }
}

pub fn dump_ref(tls: VMWorkerThread, reff: JikesObj) {
    let reff = reff.to_jtoc_call_arg();
    unsafe {
        jtoc_call!(DUMP_REF_METHOD_OFFSET, tls, reff);
    }
}

pub fn snip_obsolete_compiled_methods(tls: VMWorkerThread) {
    unsafe {
        jtoc_call!(SNIP_OBSOLETE_COMPILED_METHODS_METHOD_OFFSET, tls);
    }
}

pub fn do_reference_processing_helper_forward(
    tls: VMWorkerThread,
    trace_object_callback: extern "C" fn(*mut libc::c_void, JikesObj) -> JikesObj,
    tracer: *mut libc::c_void,
    is_nursery: bool,
) {
    let is_nursery = is_nursery.to_jtoc_call_arg();
    unsafe {
        jtoc_call!(
            DO_REFERENCE_PROCESSING_HELPER_FORWARD_METHOD_OFFSET,
            tls,
            trace_object_callback,
            tracer,
            is_nursery
        );
    }
}

pub fn do_reference_processing_helper_scan(
    tls: VMWorkerThread,
    trace_object_callback: extern "C" fn(*mut libc::c_void, JikesObj) -> JikesObj,
    tracer: *mut libc::c_void,
    is_nursery: bool,
    need_retain: bool,
) -> bool {
    let is_nursery = is_nursery.to_jtoc_call_arg();
    let need_retain = need_retain.to_jtoc_call_arg();
    let result: usize = unsafe {
        jtoc_call!(
            DO_REFERENCE_PROCESSING_HELPER_SCAN_METHOD_OFFSET,
            tls,
            trace_object_callback,
            tracer,
            is_nursery,
            need_retain
        )
    };
    bool::from_asm_result(result)
}

pub fn scan_thread(
    tls: VMWorkerThread,
    thread: VMMutatorThread,
    report_slots: extern "C" fn(*mut JikesRVMSlot, usize, *mut libc::c_void) -> *mut Address,
    report_slots_extra_data: *mut libc::c_void,
    process_code_locations: bool,
    new_roots_sufficient: bool,
) {
    let thread = thread.to_jtoc_call_arg();
    let process_code_locations = process_code_locations.to_jtoc_call_arg();
    let new_roots_sufficient = new_roots_sufficient.to_jtoc_call_arg();
    unsafe {
        jtoc_call!(
            SCAN_THREAD_METHOD_OFFSET,
            tls,
            thread,
            report_slots,
            report_slots_extra_data,
            process_code_locations,
            new_roots_sufficient
        );
    }
}

pub fn implemented_in_java(tls: VMWorkerThread, function_table_index: usize) -> bool {
    let result =
        unsafe { jtoc_call!(IMPLEMENTED_IN_JAVA_METHOD_OFFSET, tls, function_table_index) };
    bool::from_asm_result(result)
}
