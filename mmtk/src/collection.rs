use entrypoint::*;
use mmtk::scheduler::*;
use mmtk::util::opaque_pointer::*;
use mmtk::util::Address;
use mmtk::vm::{Collection, GCThreadContext};
use mmtk::MutatorContext;
use JikesRVM;
use JTOC_BASE;

pub static mut BOOT_THREAD: OpaquePointer = OpaquePointer::UNINITIALIZED;

#[derive(Default)]
pub struct VMCollection {}

// FIXME: Shouldn't these all be unsafe because of tls?
impl Collection<JikesRVM> for VMCollection {
    #[inline(always)]
    fn stop_all_mutators<E: ProcessEdgesWork<VM = JikesRVM>>(tls: VMWorkerThread) {
        unsafe {
            jtoc_call!(BLOCK_ALL_MUTATORS_FOR_GC_METHOD_OFFSET, tls);
        }
    }

    #[inline(always)]
    fn resume_mutators(tls: VMWorkerThread) {
        unsafe {
            jtoc_call!(UNBLOCK_ALL_MUTATORS_FOR_GC_METHOD_OFFSET, tls);
        }
    }

    #[inline(always)]
    fn block_for_gc(tls: VMMutatorThread) {
        unsafe {
            jtoc_call!(BLOCK_FOR_GC_METHOD_OFFSET, tls);
        }
    }

    fn spawn_gc_thread(tls: VMThread, ctx: GCThreadContext<JikesRVM>) {
        let (ctx_ptr, is_controller) = match ctx {
            GCThreadContext::Controller(c) => (Box::into_raw(c) as *mut libc::c_void, 1),
            GCThreadContext::Worker(c) => (Box::into_raw(c) as *mut libc::c_void, 0),
        };
        unsafe {
            jtoc_call!(
                SPAWN_COLLECTOR_THREAD_METHOD_OFFSET,
                tls,
                ctx_ptr,
                is_controller
            );
        }
    }

    fn prepare_mutator<T: MutatorContext<JikesRVM>>(
        tls_worker: VMWorkerThread,
        tls_mutator: VMMutatorThread,
        _m: &T,
    ) {
        unsafe {
            jtoc_call!(
                PREPARE_MUTATOR_METHOD_OFFSET,
                // convert to primitive types, so they can be used in asm!
                std::mem::transmute::<_, usize>(tls_worker),
                std::mem::transmute::<_, usize>(tls_mutator)
            );
        }
    }

    fn out_of_memory(tls: VMThread) {
        unsafe {
            jtoc_call!(OUT_OF_MEMORY_METHOD_OFFSET, tls);
        }
    }

    fn schedule_finalization(tls: VMWorkerThread) {
        unsafe {
            jtoc_call!(SCHEDULE_FINALIZER_METHOD_OFFSET, tls);
        }
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
