use crate::entrypoint::*;
use crate::JikesRVM;
use crate::JTOC_BASE;
use mmtk::util::alloc::AllocationError;
use mmtk::util::opaque_pointer::*;
use mmtk::util::Address;
use mmtk::vm::ActivePlan;
use mmtk::vm::{Collection, GCThreadContext};
use mmtk::Mutator;

use crate::jikesrvm_calls;

pub static mut BOOT_THREAD: OpaquePointer = OpaquePointer::UNINITIALIZED;

#[derive(Default)]
pub struct VMCollection {}

// FIXME: Shouldn't these all be unsafe because of tls?
impl Collection<JikesRVM> for VMCollection {
    #[inline(always)]
    fn stop_all_mutators<F>(tls: VMWorkerThread, mut mutator_visitor: F)
    where
        F: FnMut(&'static mut Mutator<JikesRVM>),
    {
        jikesrvm_calls::block_all_mutators_for_gc(tls);

        for mutator in crate::active_plan::VMActivePlan::mutators() {
            // Prepare mutator
            jikesrvm_calls::prepare_mutator(tls, mutator.mutator_tls);
            // Tell MMTk the thread is ready for stack scanning
            mutator_visitor(mutator);
        }
    }

    #[inline(always)]
    fn resume_mutators(tls: VMWorkerThread) {
        jikesrvm_calls::unblock_all_mutators_for_gc(tls);
    }

    #[inline(always)]
    fn block_for_gc(tls: VMMutatorThread) {
        jikesrvm_calls::block_for_gc(tls);
    }

    fn spawn_gc_thread(tls: VMThread, ctx: GCThreadContext<JikesRVM>) {
        let ctx_ptr = match ctx {
            GCThreadContext::Worker(c) => Box::into_raw(c),
        };
        jikesrvm_calls::spawn_collector_thread(tls, ctx_ptr);
    }

    fn out_of_memory(tls: VMThread, _err_kind: AllocationError) {
        jikesrvm_calls::out_of_memory(tls);
    }

    fn schedule_finalization(tls: VMWorkerThread) {
        jikesrvm_calls::schedule_finalizer(tls);
    }
}

impl VMCollection {
    /// # Safety
    /// Caller needs to make sure thread_id is valid.
    #[inline(always)]
    pub unsafe fn thread_from_id(thread_id: usize) -> Address {
        ((JTOC_BASE + THREAD_BY_SLOT_FIELD_OFFSET).load::<Address>() + 4 * thread_id)
            .load::<Address>()
    }

    /// # Safety
    /// Caller needs to make sure thread_index is valid.
    #[inline(always)]
    pub unsafe fn thread_from_index(thread_index: usize) -> Address {
        ((JTOC_BASE + THREADS_FIELD_OFFSET).load::<Address>() + 4 * thread_index).load::<Address>()
    }
}
