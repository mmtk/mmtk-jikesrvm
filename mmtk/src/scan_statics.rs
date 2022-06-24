use crate::scanning::EDGES_BUFFER_CAPACITY;
use crate::JikesRVM;
use entrypoint::*;
use mmtk::scheduler::*;
use mmtk::util::opaque_pointer::*;
use mmtk::vm::RootsWorkFactory;
use mmtk::MMTK;
use std::arch::asm;
use JTOC_BASE;

#[cfg(target_pointer_width = "32")]
const REF_SLOT_SIZE: usize = 1;
#[cfg(target_pointer_width = "64")]
const REF_SLOT_SIZE: usize = 2;

const CHUNK_SIZE_MASK: usize = 0xFFFFFFFF - (REF_SLOT_SIZE - 1);

pub fn scan_statics(
    tls: VMWorkerThread,
    factory: &mut impl RootsWorkFactory,
    subwork_id: usize,
    total_subwork: usize,
) {
    unsafe {
        let slots = JTOC_BASE;
        // let cc = VMActivePlan::collector(tls);

        let number_of_collectors: usize = total_subwork;
        let number_of_references: usize =
            jtoc_call!(GET_NUMBER_OF_REFERENCE_SLOTS_METHOD_OFFSET, tls);
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

        let mut edges = Vec::with_capacity(EDGES_BUFFER_CAPACITY);

        let mut slot = start;
        while slot < end {
            let slot_offset = slot * 4;
            // TODO: check_reference?
            edges.push(slots + slot_offset);
            if edges.len() >= EDGES_BUFFER_CAPACITY {
                factory.create_process_edge_roots_work(edges);
                edges = Vec::with_capacity(EDGES_BUFFER_CAPACITY);
            }
            // trace.process_root_edge(slots + slot_offset, true);
            slot += REF_SLOT_SIZE;
        }
        if !edges.is_empty() {
            factory.create_process_edge_roots_work(edges);
        }
    }
}

pub struct ScanStaticRoots<F: RootsWorkFactory> {
    factory: F,
    subwork_id: usize,
    total_subwork: usize,
}

impl<F: RootsWorkFactory> ScanStaticRoots<F> {
    pub fn new(factory: F, subwork_id: usize, total_subwork: usize) -> Self {
        Self {
            factory,
            subwork_id,
            total_subwork,
        }
    }
}

impl<F: RootsWorkFactory> GCWork<JikesRVM> for ScanStaticRoots<F> {
    fn do_work(&mut self, worker: &mut GCWorker<JikesRVM>, _mmtk: &'static MMTK<JikesRVM>) {
        scan_statics(
            worker.tls,
            &mut self.factory,
            self.subwork_id,
            self.total_subwork,
        );
    }
}
