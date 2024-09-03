use crate::jikesrvm_calls;
use crate::scanning::SLOTS_BUFFER_CAPACITY;
use crate::JikesRVM;
use crate::JikesRVMSlot;
use mmtk::scheduler::*;
use mmtk::util::opaque_pointer::*;
use mmtk::vm::RootsWorkFactory;
use mmtk::MMTK;
use JTOC_BASE;

#[cfg(target_pointer_width = "32")]
const REF_SLOT_SIZE: usize = 1;
#[cfg(target_pointer_width = "64")]
const REF_SLOT_SIZE: usize = 2;

const CHUNK_SIZE_MASK: usize = 0xFFFFFFFF - (REF_SLOT_SIZE - 1);

pub fn scan_statics(
    tls: VMWorkerThread,
    factory: &mut impl RootsWorkFactory<JikesRVMSlot>,
    subwork_id: usize,
    total_subwork: usize,
) {
    unsafe {
        let slots = JTOC_BASE;
        // let cc = VMActivePlan::collector(tls);

        let number_of_collectors: usize = total_subwork;
        let number_of_references: usize = jikesrvm_calls::get_number_of_reference_slots(tls);
        let chunk_size: usize = (number_of_references / number_of_collectors) & CHUNK_SIZE_MASK;
        let thread_ordinal = subwork_id;

        let start: usize = if thread_ordinal == 0 {
            REF_SLOT_SIZE
        } else {
            thread_ordinal * chunk_size
        };
        let end: usize = if thread_ordinal + 1 == number_of_collectors {
            number_of_references
        } else {
            (thread_ordinal + 1) * chunk_size
        };

        let mut slot_list = Vec::with_capacity(SLOTS_BUFFER_CAPACITY);

        let mut slot = start;
        while slot < end {
            let slot_offset = slot * 4;
            // TODO: check_reference?
            slot_list.push(JikesRVMSlot::from_address(slots + slot_offset));
            if slot_list.len() >= SLOTS_BUFFER_CAPACITY {
                factory.create_process_roots_work(slot_list);
                slot_list = Vec::with_capacity(SLOTS_BUFFER_CAPACITY);
            }
            // trace.process_root_edge(slots + slot_offset, true);
            slot += REF_SLOT_SIZE;
        }
        if !slot_list.is_empty() {
            factory.create_process_roots_work(slot_list);
        }
    }
}

pub struct ScanStaticRoots<F: RootsWorkFactory<JikesRVMSlot>> {
    factory: F,
    subwork_id: usize,
    total_subwork: usize,
}

impl<F: RootsWorkFactory<JikesRVMSlot>> ScanStaticRoots<F> {
    pub fn new(factory: F, subwork_id: usize, total_subwork: usize) -> Self {
        Self {
            factory,
            subwork_id,
            total_subwork,
        }
    }
}

impl<F: RootsWorkFactory<JikesRVMSlot>> GCWork<JikesRVM> for ScanStaticRoots<F> {
    fn do_work(&mut self, worker: &mut GCWorker<JikesRVM>, _mmtk: &'static MMTK<JikesRVM>) {
        scan_statics(
            worker.tls,
            &mut self.factory,
            self.subwork_id,
            self.total_subwork,
        );
    }
}
