use crate::{JikesRVM, SINGLETON};
use entrypoint::*;
use mmtk::scheduler::gc_work::*;
use mmtk::scheduler::*;
use mmtk::util::opaque_pointer::*;
use mmtk::MMTK;
use std::marker::PhantomData;
use JTOC_BASE;

#[cfg(target_pointer_width = "32")]
const REF_SLOT_SIZE: usize = 1;
#[cfg(target_pointer_width = "64")]
const REF_SLOT_SIZE: usize = 2;

const CHUNK_SIZE_MASK: usize = 0xFFFFFFFF - (REF_SLOT_SIZE - 1);

pub fn scan_statics<W: ProcessEdgesWork<VM = JikesRVM>>(
    tls: VMWorkerThread,
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

        let mut edges = Vec::with_capacity(W::CAPACITY);

        let mut slot = start;
        while slot < end {
            let slot_offset = slot * 4;
            // TODO: check_reference?
            edges.push(slots + slot_offset);
            if edges.len() >= W::CAPACITY {
                SINGLETON.scheduler.work_buckets[WorkBucketStage::Closure]
                    .add(W::new(edges, true, &SINGLETON));
                edges = Vec::with_capacity(W::CAPACITY);
            }
            // trace.process_root_edge(slots + slot_offset, true);
            slot += REF_SLOT_SIZE;
        }
        SINGLETON.scheduler.work_buckets[WorkBucketStage::Closure]
            .add(W::new(edges, true, &SINGLETON));
    }
}

pub struct ScanStaticRoots<E: ProcessEdgesWork<VM = JikesRVM>>(usize, usize, PhantomData<E>);

impl<E: ProcessEdgesWork<VM = JikesRVM>> ScanStaticRoots<E> {
    pub fn new(subwork_id: usize, total_subwork: usize) -> Self {
        Self(subwork_id, total_subwork, PhantomData)
    }
}

impl<E: ProcessEdgesWork<VM = JikesRVM>> GCWork<JikesRVM> for ScanStaticRoots<E> {
    fn do_work(&mut self, worker: &mut GCWorker<JikesRVM>, _mmtk: &'static MMTK<JikesRVM>) {
        scan_statics::<E>(worker.tls, self.0, self.1);
    }
}
