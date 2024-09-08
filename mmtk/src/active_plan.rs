use crate::collection::VMCollection;
use crate::entrypoint::*;
use crate::JikesRVM;
use crate::JTOC_BASE;
use mmtk::util::opaque_pointer::*;
use mmtk::util::Address;
use mmtk::vm::ActivePlan;
use mmtk::Mutator;
use std::mem;

use std::sync::{Mutex, MutexGuard};

lazy_static! {
    static ref MUTATOR_LOCK: Mutex<()> = Mutex::new(());
}

struct JikesRVMMutatorIterator<'a> {
    _guard: MutexGuard<'a, ()>,
    counter: usize,
}

impl<'a> JikesRVMMutatorIterator<'a> {
    fn new(guard: MutexGuard<'a, ()>) -> Self {
        Self {
            _guard: guard,
            counter: 0,
        }
    }
}

impl<'a> Iterator for JikesRVMMutatorIterator<'a> {
    type Item = &'a mut Mutator<JikesRVM>;

    fn next(&mut self) -> Option<Self::Item> {
        // We don't need this in the loop for STW-GC
        let num_threads = unsafe { (JTOC_BASE + NUM_THREADS_FIELD_OFFSET).load::<usize>() };
        loop {
            let idx = self.counter;
            self.counter += 1;
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

#[derive(Default)]
pub struct VMActivePlan {}

impl ActivePlan<JikesRVM> for VMActivePlan {
    fn number_of_mutators() -> usize {
        Self::mutators().count()
    }

    fn is_mutator(tls: VMThread) -> bool {
        let thread: Address = unsafe { mem::transmute(tls) };
        !unsafe { (thread + IS_COLLECTOR_FIELD_OFFSET).load::<bool>() }
    }

    // XXX: Are they actually static
    fn mutator(tls: VMMutatorThread) -> &'static mut Mutator<JikesRVM> {
        unsafe {
            let thread: Address = mem::transmute(tls);
            let mutator = (thread + MMTK_HANDLE_FIELD_OFFSET).load::<usize>();
            &mut *(mutator as *mut Mutator<JikesRVM>)
        }
    }

    fn mutators<'a>() -> Box<dyn Iterator<Item = &'a mut Mutator<JikesRVM>> + 'a> {
        let guard = MUTATOR_LOCK.lock().unwrap();
        Box::new(JikesRVMMutatorIterator::new(guard))
    }
}
