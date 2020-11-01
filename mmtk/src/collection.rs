use libc::c_void;

use mmtk::vm::Collection;
use mmtk::util::Address;
use mmtk::MutatorContext;
use mmtk::util::opaque_pointer::OpaquePointer;
use mmtk::scheduler::*;
use mmtk::scheduler::gc_works::*;
use entrypoint::*;
use JTOC_BASE;
use JikesRVM;

pub static mut BOOT_THREAD: OpaquePointer = OpaquePointer::UNINITIALIZED;

#[derive(Default)]
pub struct VMCollection {}

// FIXME: Shouldn't these all be unsafe because of tls?
impl Collection<JikesRVM> for VMCollection {
    #[inline(always)]
    fn stop_all_mutators<E: ProcessEdgesWork<VM = JikesRVM>>(tls: OpaquePointer) {
        trace!("stop_all_mutators");
        unsafe {
            jtoc_call!(BLOCK_ALL_MUTATORS_FOR_GC_METHOD_OFFSET, tls);
        }
        trace!("stop_all_mutators finish");
    }

    #[inline(always)]
    fn resume_mutators(tls: OpaquePointer) {
        trace!("stop_all_mutators");
        unsafe {
            jtoc_call!(UNBLOCK_ALL_MUTATORS_FOR_GC_METHOD_OFFSET, tls);
        }
        trace!("stop_all_mutators finish");
    }

    #[inline(always)]
    fn block_for_gc(tls: OpaquePointer) {
        unsafe {
            jtoc_call!(BLOCK_FOR_GC_METHOD_OFFSET, tls);
        }
    }

    fn spawn_worker_thread(tls: OpaquePointer, ctx: Option<&GCWorker<JikesRVM>>) {
        let ctx_ptr = if let Some(r) = ctx {
            r as *const GCWorker<JikesRVM> as *mut GCWorker<JikesRVM>
        } else {
            std::ptr::null_mut()
        };
        unsafe {
            jtoc_call!(SPAWN_COLLECTOR_THREAD_METHOD_OFFSET, tls, ctx_ptr);
        }
    }

    fn prepare_mutator<T: MutatorContext<JikesRVM>>(tls: OpaquePointer, m: &T) {
        unsafe {
            jtoc_call!(PREPARE_MUTATOR_METHOD_OFFSET, tls, tls);
        }
    }

    fn out_of_memory(tls: OpaquePointer) {
        unsafe {
            jtoc_call!(OUT_OF_MEMORY_METHOD_OFFSET, tls);
        }
    }
}

impl VMCollection {
    #[inline(always)]
    pub unsafe fn thread_from_id(thread_id: usize) -> Address {
        ((JTOC_BASE + THREAD_BY_SLOT_FIELD_OFFSET).load::<Address>() + 4 * thread_id).load::<Address>()
    }

    #[inline(always)]
    pub unsafe fn thread_from_index(thread_index: usize) -> Address {
        ((JTOC_BASE + THREADS_FIELD_OFFSET).load::<Address>() + 4 * thread_index).load::<Address>()
    }
}