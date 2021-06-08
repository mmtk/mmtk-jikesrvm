use std::sync::atomic::{AtomicUsize, Ordering};

use mmtk::util::{metadata as mmtk_meta, ObjectReference};

use crate::java_size_constants::LOG_BITS_IN_BYTE;

pub(crate) fn load_metadata(
    metadata_spec: mmtk_meta::MetadataSpec,
    object: ObjectReference,
    mask: Option<usize>,
    atomic_ordering: Option<Ordering>,
) -> usize {
    if metadata_spec.is_side_metadata {
        if let Some(order) = atomic_ordering {
            mmtk_meta::load_atomic(metadata_spec, object.to_address(), order)
        } else {
            unsafe { mmtk_meta::load(metadata_spec, object.to_address()) }
        }
    } else {
        if metadata_spec.num_of_bits <= 8 {
            debug_assert!(
                (metadata_spec.offset >> LOG_BITS_IN_BYTE)
                    == ((metadata_spec.offset + metadata_spec.num_of_bits) >> LOG_BITS_IN_WORD),
                "Metadata: ({:?}) stretches over two words!",
                metadata_spec
            );
            // e.g. 30 => 0; -2 => -1
            let word_offset = metadata_spec.offset >> LOG_BITS_IN_WORD;
            // e.g. 30 => (30-0)=30; -2 => (-2--32)=30
            let bit_shift = metadata_spec.offset - (word_offset << LOG_BITS_IN_WORD);
            // e.g (30, 1 log_bits) => (1<<(1<<1)) = 0b100 - 1 = 0b11 << 30 = 0xC000_0000
            let mask = ((1usize << metadata_spec.num_of_bits) - 1) << bit_shift;

            let word = unsafe {
                (*(object.to_address() + word_offset).to_ptr::<AtomicUsize>())
                    .load(Ordering::SeqCst)
            };

            (word & mask) >> bit_shift
        } else if metadata_spec.num_of_bits == 16 {
            // e.g. 30 => 0; -2 => -1
            let word_offset = metadata_spec.offset >> LOG_BITS_IN_WORD;
            // e.g. 30 => (30-0)=30; -2 => (-2--32)=30
            let bit_shift = metadata_spec.offset - (word_offset << LOG_BITS_IN_WORD);
            // e.g (30, 1 log_bits) => (1<<(1<<1)) = 0b100 - 1 = 0b11 << 30 = 0xC000_0000
            let mask = ((1usize << metadata_spec.num_of_bits) - 1) << bit_shift;

            let word = unsafe {
                (*(object.to_address() + word_offset).to_ptr::<AtomicUsize>())
                    .load(Ordering::SeqCst)
            };

            (word & mask) >> bit_shift
        } else if metadata_spec.num_of_bits == 32 {
            // e.g. 30 => 0; -2 => -1
            let word_offset = metadata_spec.offset >> LOG_BITS_IN_WORD;
            // e.g. 30 => (30-0)=30; -2 => (-2--32)=30
            let bit_shift = metadata_spec.offset - (word_offset << LOG_BITS_IN_WORD);
            // e.g (30, 1 log_bits) => (1<<(1<<1)) = 0b100 - 1 = 0b11 << 30 = 0xC000_0000
            let mask = ((1usize << metadata_spec.num_of_bits) - 1) << bit_shift;

            let word = unsafe {
                (*(object.to_address() + word_offset).to_ptr::<AtomicUsize>())
                    .load(Ordering::SeqCst)
            };

            (word & mask) >> bit_shift
        }
    }
}

fn store_metadata(
    metadata_spec: MetadataSpec,
    object: ObjectReference,
    val: usize,
    mask: Option<usize>,
    atomic_ordering: Option<Ordering>,
) {
    if metadata_spec.is_side_metadata {
        if let Some(order) = atomic_ordering {
            mmtk_meta::store_atomic(metadata_spec, object.to_address(), val, order);
        } else {
            unsafe {
                mmtk_meta::store(metadata_spec, object.to_address(), val);
            }
        }
    } else {
    }
}
