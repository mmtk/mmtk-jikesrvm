use collection::VMCollection;
use entrypoint::*;
use mmtk::util::opaque_pointer::*;
use mmtk::util::{Address, SynchronizedCounter};
use mmtk::vm::ActivePlan;
use mmtk::Mutator;
use mmtk::Plan;
use std::mem;
use JikesRVM;
use JTOC_BASE;
use SINGLETON;

static MUTATOR_COUNTER: SynchronizedCounter = SynchronizedCounter::new(0);

#[derive(Default)]
pub struct VMActivePlan {}

impl ActivePlan<JikesRVM> for VMActivePlan {
    fn number_of_mutators() -> usize {
        let num_threads = unsafe { (JTOC_BASE + NUM_THREADS_FIELD_OFFSET).load::<usize>() };
        let mut num_mutators = 0usize;
        for idx in 0..num_threads {
            let t = unsafe { VMCollection::thread_from_index(idx) };
            let is_mutator = unsafe { !(t + IS_COLLECTOR_FIELD_OFFSET).load::<bool>() };
            if is_mutator {
                num_mutators += 1;
            }
        }
        num_mutators
    }

    fn global() -> &'static dyn Plan<VM = JikesRVM> {
        &*SINGLETON.plan
    }

    fn is_mutator(tls: VMThread) -> bool {
        let thread: Address = unsafe { mem::transmute(tls) };
        ! unsafe { (thread + IS_COLLECTOR_FIELD_OFFSET).load::<bool>() }
    }

    // XXX: Are they actually static
    fn mutator(tls: VMMutatorThread) -> &'static mut Mutator<JikesRVM> {
        unsafe {
            let thread: Address = mem::transmute(tls);
            let mutator = (thread + MMTK_HANDLE_FIELD_OFFSET).load::<usize>();
            &mut *(mutator as *mut Mutator<JikesRVM>)
        }
    }

    fn reset_mutator_iterator() {
        MUTATOR_COUNTER.reset();
    }

    fn get_next_mutator() -> Option<&'static mut Mutator<JikesRVM>> {
        // We don't need this in the loop for STW-GC
        let num_threads = unsafe { (JTOC_BASE + NUM_THREADS_FIELD_OFFSET).load::<usize>() };
        loop {
            let idx = MUTATOR_COUNTER.increment();
            if idx >= num_threads {
                return None;
            } else {
                let t = unsafe { VMCollection::thread_from_index(idx) };
                let is_mutator = unsafe { !(t + IS_COLLECTOR_FIELD_OFFSET).load::<bool>() };
                if is_mutator {
                    unsafe {
                        let mutator = (t + MMTK_HANDLE_FIELD_OFFSET).load::<usize>();
                        let ret = &mut *(mutator as *mut Mutator<JikesRVM>);
                        return Some(ret);
                    }
                }
            }
        }
    }
}
