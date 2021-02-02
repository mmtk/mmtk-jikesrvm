use std::mem;
use libc::c_void;
use mmtk::vm::ActivePlan;
use mmtk::Plan;
use mmtk::util::{Address, SynchronizedCounter};
use mmtk::util::OpaquePointer;
use entrypoint::*;
use collection::VMCollection;
use JTOC_BASE;
use JikesRVM;
use SINGLETON;
use mmtk::scheduler::*;
use mmtk::Mutator;

static MUTATOR_COUNTER: SynchronizedCounter = SynchronizedCounter::new(0);

#[derive(Default)]
pub struct VMActivePlan<> {}

impl ActivePlan<JikesRVM> for VMActivePlan {
    unsafe fn worker(tls: OpaquePointer) -> &'static mut GCWorker<JikesRVM> {
        let thread: Address = unsafe { mem::transmute(tls) };
        let system_thread = unsafe { (thread + SYSTEM_THREAD_FIELD_OFFSET).load::<Address>() };
        let cc = unsafe {
            &mut *((system_thread + WORKER_INSTANCE_FIELD_OFFSET)
                .load::<*mut GCWorker<JikesRVM>>())
        };
        cc
    }

    fn number_of_mutators() -> usize {
        unsafe {
            (JTOC_BASE + NUM_THREADS_FIELD_OFFSET).load::<usize>()
        }
    }

    fn global() -> &'static dyn Plan<VM=JikesRVM> {
        &*SINGLETON.plan
    }

    unsafe fn is_mutator(tls: OpaquePointer) -> bool {
        let thread: Address = unsafe { mem::transmute(tls) };
        !(thread + IS_COLLECTOR_FIELD_OFFSET).load::<bool>()
    }

    // XXX: Are they actually static
    unsafe fn mutator(tls: OpaquePointer) -> &'static mut Mutator<JikesRVM> {
        let thread: Address = unsafe { mem::transmute(tls) };
        let mutator = (thread + MMTK_HANDLE_FIELD_OFFSET).load::<usize>();
        &mut *(mutator as *mut Mutator<JikesRVM>)
    }

    fn reset_mutator_iterator() {
        MUTATOR_COUNTER.reset();
    }

    fn get_next_mutator() -> Option<&'static mut Mutator<JikesRVM>> {
        loop {
            let idx = MUTATOR_COUNTER.increment();
            let num_threads = unsafe { (JTOC_BASE + NUM_THREADS_FIELD_OFFSET).load::<usize>() };
            if idx >= num_threads {
                return None;
            } else {
                let t = unsafe { VMCollection::thread_from_index(idx) };
                let active_mutator_context = unsafe { (t + ACTIVE_MUTATOR_CONTEXT_FIELD_OFFSET)
                    .load::<bool>() };
                if active_mutator_context {
                    unsafe {
                        let mutator = (t + MMTK_HANDLE_FIELD_OFFSET).load::<usize>();
                        let ret =
                            &mut *(mutator as *mut Mutator<JikesRVM>);
                        return Some(ret);
                    }
                }
            }
        }
    }
}