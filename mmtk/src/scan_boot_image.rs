use crate::unboxed_size_constants::*;
use crate::{JikesRVM, SINGLETON};
use entrypoint::*;
use java_size_constants::*;
use mmtk::scheduler::gc_work::*;
use mmtk::scheduler::*;
use mmtk::util::conversions;
use mmtk::util::Address;
use mmtk::util::OpaquePointer;
use mmtk::MMTK;
use std::marker::PhantomData;
use std::mem;
use std::sync::atomic::{AtomicUsize, Ordering};
use JTOC_BASE;

const FILTER: bool = true;

const LOG_CHUNK_BYTES: usize = 12;
const LONGENCODING_MASK: usize = 0x1;
const RUN_MASK: usize = 0x2;
const LONGENCODING_OFFSET_BYTES: usize = 4;

static ROOTS: AtomicUsize = AtomicUsize::new(0);
static REFS: AtomicUsize = AtomicUsize::new(0);

pub fn scan_boot_image<W: ProcessEdgesWork<VM = JikesRVM>>(
    _tls: OpaquePointer,
    subwork_id: usize,
    total_subwork: usize,
) {
    unsafe {
        let boot_record = (JTOC_BASE + THE_BOOT_RECORD_FIELD_OFFSET).load::<Address>();
        let map_start = (boot_record + BOOT_IMAGE_R_MAP_START_OFFSET).load::<Address>();
        let map_end = (boot_record + BOOT_IMAGE_R_MAP_END_OFFSET).load::<Address>();
        let image_start = (boot_record + BOOT_IMAGE_DATA_START_FIELD_OFFSET).load::<Address>();

        // let collector = VMActivePlan::collector(tls);

        let stride = total_subwork << LOG_CHUNK_BYTES;
        trace!("stride={}", stride);
        let start = subwork_id << LOG_CHUNK_BYTES;
        trace!("start={}", start);
        let mut cursor = map_start + start;
        trace!("cursor={:x}", cursor);

        ROOTS.store(0, Ordering::Relaxed);
        ROOTS.store(0, Ordering::Relaxed);

        let mut edges = vec![];
        while cursor < map_end {
            trace!("Processing chunk at {:x}", cursor);
            process_chunk(cursor, image_start, map_start, map_end, |edge| {
                edges.push(edge);
                if edges.len() >= W::CAPACITY {
                    let mut new_edges = Vec::with_capacity(W::CAPACITY);
                    mem::swap(&mut new_edges, &mut edges);
                    SINGLETON.scheduler.work_buckets[WorkBucketStage::Closure]
                        .add(W::new(new_edges, true, &SINGLETON));
                }
            });
            trace!("Chunk processed successfully");
            cursor += stride;
        }
        SINGLETON.scheduler.work_buckets[WorkBucketStage::Closure]
            .add(W::new(edges, true, &SINGLETON));
    }
}

fn process_chunk(
    chunk_start: Address,
    image_start: Address,
    _map_start: Address,
    map_end: Address,
    mut report_edge: impl FnMut(Address),
) {
    let mut value: usize;
    let mut offset: usize = 0;
    let mut cursor: Address = chunk_start;
    unsafe {
        while {
            value = cursor.load::<u8>() as usize;
            value != 0
        } {
            /* establish the offset */
            if (value & LONGENCODING_MASK) != 0 {
                offset = decode_long_encoding(cursor);
                cursor += LONGENCODING_OFFSET_BYTES;
            } else {
                offset += value & 0xfc;
                cursor += 1isize;
            }
            /* figure out the length of the run, if any */
            let mut runlength: usize = 0;
            if (value & RUN_MASK) != 0 {
                runlength = cursor.load::<u8>() as usize;
                cursor += 1isize;
            }
            /* enqueue the specified slot or slots */
            debug_assert!(conversions::is_address_aligned(Address::from_usize(offset)));
            let mut slot: Address = image_start + offset;
            if cfg!(feature = "debug") {
                REFS.fetch_add(1, Ordering::Relaxed);
            }

            if !FILTER || slot.load::<Address>() > map_end {
                if cfg!(feature = "debug") {
                    ROOTS.fetch_add(1, Ordering::Relaxed);
                }
                report_edge(slot);
            }
            if runlength != 0 {
                for _ in 0..runlength {
                    offset += BYTES_IN_ADDRESS;
                    slot = image_start + offset;
                    debug_assert!(conversions::is_address_aligned(slot));
                    if cfg!(feature = "debug") {
                        REFS.fetch_add(1, Ordering::Relaxed);
                    }
                    if !FILTER || slot.load::<Address>() > map_end {
                        if cfg!(feature = "debug") {
                            ROOTS.fetch_add(1, Ordering::Relaxed);
                        }
                        // TODO: check_reference(slot) ?
                        report_edge(slot);
                    }
                }
            }
        }
    }
}

fn decode_long_encoding(cursor: Address) -> usize {
    unsafe {
        let mut value: usize;
        value = cursor.load::<u8>() as usize & 0x000000fc;
        value |= (((cursor + 1isize).load::<u8>() as usize) << BITS_IN_BYTE) & 0x0000ff00;
        value |= (((cursor + 2isize).load::<u8>() as usize) << (2 * BITS_IN_BYTE)) & 0x00ff0000;
        value |= (((cursor + 3isize).load::<u8>() as usize) << (3 * BITS_IN_BYTE)) & 0xff000000;
        value
    }
}

pub struct ScanBootImageRoots<E: ProcessEdgesWork<VM = JikesRVM>>(usize, usize, PhantomData<E>);

impl<E: ProcessEdgesWork<VM = JikesRVM>> ScanBootImageRoots<E> {
    pub fn new(subwork_id: usize, total_subwork: usize) -> Self {
        Self(subwork_id, total_subwork, PhantomData)
    }
}

impl<E: ProcessEdgesWork<VM = JikesRVM>> GCWork<JikesRVM> for ScanBootImageRoots<E> {
    fn do_work(&mut self, _worker: &mut GCWorker<JikesRVM>, _mmtk: &'static MMTK<JikesRVM>) {
        scan_boot_image::<E>(OpaquePointer::UNINITIALIZED, self.0, self.1);
    }
}
