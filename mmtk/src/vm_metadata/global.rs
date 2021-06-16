use std::sync::atomic::{AtomicU16, AtomicU32, AtomicU8, AtomicUsize, Ordering};

use mmtk::util::{metadata as mmtk_meta, ObjectReference};

use super::constants::*;

#[inline(always)]
pub(crate) fn load_metadata(
    metadata_spec: mmtk_meta::MetadataSpec,
    object: ObjectReference,
    optional_mask: Option<usize>,
    atomic_ordering: Option<Ordering>,
) -> usize {
    if metadata_spec.is_side_metadata {
        if let Some(order) = atomic_ordering {
            mmtk_meta::side_metadata::load_atomic(metadata_spec, object.to_address(), order)
        } else {
            unsafe { mmtk_meta::side_metadata::load(metadata_spec, object.to_address()) }
        }
    } else {
        debug_assert!(optional_mask.is_none() || metadata_spec.num_of_bits >= BITS_IN_BYTE,"optional_mask is only supported for 8X-bits in-header metadata. Problematic MetadataSpec: ({:?})", metadata_spec);

        let res: usize = if metadata_spec.num_of_bits < 8 {
            debug_assert!(
                (metadata_spec.offset >> LOG_BITS_IN_BYTE)
                    == ((metadata_spec.offset + metadata_spec.num_of_bits as isize - 1)
                        >> LOG_BITS_IN_BYTE),
                "Metadata << 8-bits: ({:?}) stretches over two bytes!",
                metadata_spec
            );
            let offset = metadata_spec.offset >> LOG_BITS_IN_BYTE;
            let shift = metadata_spec.offset - (offset << LOG_BITS_IN_BYTE);
            let mask = ((1u8 << metadata_spec.num_of_bits) - 1) << shift;

            let loaded_val = unsafe {
                if let Some(order) = atomic_ordering {
                    (object.to_address() + offset).atomic_load::<AtomicU8>(order)
                } else {
                    (object.to_address() + offset).load::<u8>()
                }
            };

            ((loaded_val & mask) >> shift) as usize
        } else if metadata_spec.num_of_bits == 8 {
            debug_assert!(
                metadata_spec.offset.trailing_zeros() as usize >= LOG_BITS_IN_BYTE as usize,
                "Metadata 16-bits: ({:?}) offset must be byte aligned!",
                metadata_spec
            );
            let offset = metadata_spec.offset >> LOG_BITS_IN_BYTE;

            unsafe {
                if let Some(order) = atomic_ordering {
                    (object.to_address() + offset).atomic_load::<AtomicU8>(order) as usize
                } else {
                    (object.to_address() + offset).load::<u8>() as usize
                }
            }
        } else if metadata_spec.num_of_bits == 16 {
            debug_assert!(
                metadata_spec.offset.trailing_zeros() as usize >= LOG_BITS_IN_U16,
                "Metadata 16-bits: ({:?}) offset must be 2-bytes aligned!",
                metadata_spec
            );
            let offset = metadata_spec.offset >> LOG_BITS_IN_BYTE;

            unsafe {
                if let Some(order) = atomic_ordering {
                    (object.to_address() + offset).atomic_load::<AtomicU16>(order) as usize
                } else {
                    (object.to_address() + offset).load::<u16>() as usize
                }
            }
        } else if metadata_spec.num_of_bits == 32 {
            debug_assert!(
                metadata_spec.offset.trailing_zeros() as usize >= LOG_BITS_IN_WORD,
                "Metadata 32-bits: ({:?}) offset must be 4-bytes aligned!",
                metadata_spec
            );
            let offset = metadata_spec.offset >> LOG_BITS_IN_BYTE;

            unsafe {
                if let Some(order) = atomic_ordering {
                    (object.to_address() + offset).atomic_load::<AtomicUsize>(order)
                } else {
                    (object.to_address() + offset).load::<usize>()
                }
            }
        } else {
            unreachable!()
        };

        if let Some(mask) = optional_mask {
            res & mask
        } else {
            res
        }
    }
}

#[inline(always)]
pub(crate) fn store_metadata(
    metadata_spec: mmtk_meta::MetadataSpec,
    object: ObjectReference,
    val: usize,
    optional_mask: Option<usize>,
    atomic_ordering: Option<Ordering>,
) {
    if metadata_spec.is_side_metadata {
        if let Some(order) = atomic_ordering {
            mmtk_meta::side_metadata::store_atomic(metadata_spec, object.to_address(), val, order);
        } else {
            unsafe {
                mmtk_meta::side_metadata::store(metadata_spec, object.to_address(), val);
            }
        }
    } else {
        debug_assert!(optional_mask.is_none() || metadata_spec.num_of_bits >= 8,"optional_mask is only supported for 8X-bits in-header metadata. Problematic MetadataSpec: ({:?})", metadata_spec);

        if metadata_spec.num_of_bits < 8 {
            debug_assert!(
                (metadata_spec.offset >> LOG_BITS_IN_BYTE)
                    == ((metadata_spec.offset + metadata_spec.num_of_bits as isize - 1)
                        >> LOG_BITS_IN_BYTE),
                "Metadata << 8-bits: ({:?}) stretches over two bytes!",
                metadata_spec
            );
            let offset = metadata_spec.offset >> LOG_BITS_IN_BYTE;
            let shift = metadata_spec.offset - (offset << LOG_BITS_IN_BYTE);
            debug_assert!(shift >= 0);
            let shift = shift as usize;
            let mask = ((1u8 << metadata_spec.num_of_bits) - 1) << shift;

            let new_metadata = (val as u8) << shift;
            let meta_addr = object.to_address() + offset;
            if let Some(order) = atomic_ordering {
                unsafe {
                    loop {
                        let old_val = meta_addr.atomic_load::<AtomicU8>(order);
                        let new_val = (old_val & !mask) | new_metadata;
                        if meta_addr
                            .compare_exchange::<AtomicU8>(old_val, new_val, order, order)
                            .is_ok()
                        {
                            break;
                        }
                    }
                }
            } else {
                unsafe {
                    let old_val = meta_addr.load::<u8>();
                    let new_val = (old_val & !mask) | new_metadata;
                    meta_addr.store::<u8>(new_val);
                }
            }
        } else if metadata_spec.num_of_bits == 8 {
            debug_assert!(
                metadata_spec.offset.trailing_zeros() as usize >= LOG_BITS_IN_BYTE as usize,
                "Metadata 8-bits: ({:?}) offset must be byte-aligned!",
                metadata_spec
            );
            let offset = metadata_spec.offset >> LOG_BITS_IN_BYTE;
            let meta_addr = object.to_address() + offset;

            unsafe {
                if let Some(order) = atomic_ordering {
                    // if the optional mask is provided (e.g. for forwarding pointer), we need to use compare_exchange
                    if let Some(mask) = optional_mask {
                        loop {
                            let old_val = meta_addr.atomic_load::<AtomicU8>(order);
                            let new_val = (old_val & !(mask as u8)) | (val as u8 & (mask as u8));
                            if meta_addr
                                .compare_exchange::<AtomicU8>(old_val, new_val, order, order)
                                .is_ok()
                            {
                                break;
                            }
                        }
                    } else {
                        meta_addr.atomic_store::<AtomicU8>(val as u8, order);
                    }
                } else {
                    let val = if let Some(mask) = optional_mask {
                        let old_val = meta_addr.load::<u8>();
                        (old_val & !(mask as u8)) | (val as u8 & (mask as u8))
                    } else {
                        val as u8
                    };
                    meta_addr.store(val as u8);
                }
            }
        } else if metadata_spec.num_of_bits == 16 {
            debug_assert!(
                metadata_spec.offset.trailing_zeros() as usize >= LOG_BITS_IN_U16,
                "Metadata 16-bits: ({:?}) offset must be 2-bytes aligned!",
                metadata_spec
            );
            let offset = metadata_spec.offset >> LOG_BITS_IN_BYTE;
            let meta_addr = object.to_address() + offset;

            unsafe {
                if let Some(order) = atomic_ordering {
                    // if the optional mask is provided (e.g. for forwarding pointer), we need to use compare_exchange
                    if let Some(mask) = optional_mask {
                        loop {
                            let old_val = meta_addr.atomic_load::<AtomicU16>(order);
                            let new_val = (old_val & !(mask as u16)) | (val as u16 & (mask as u16));
                            if meta_addr
                                .compare_exchange::<AtomicU16>(old_val, new_val, order, order)
                                .is_ok()
                            {
                                break;
                            }
                        }
                    } else {
                        meta_addr.atomic_store::<AtomicU16>(val as u16, order);
                    }
                } else {
                    let val = if let Some(mask) = optional_mask {
                        let old_val = meta_addr.load::<u16>();
                        (old_val & !(mask as u16)) | (val as u16 & (mask as u16))
                    } else {
                        val as u16
                    };

                    meta_addr.store(val as u16);
                }
            }
        } else if metadata_spec.num_of_bits == 32 {
            debug_assert!(
                metadata_spec.offset.trailing_zeros() as usize >= LOG_BITS_IN_WORD,
                "Metadata 32-bits: ({:?}) offset must be 4-bytes aligned!",
                metadata_spec
            );
            let offset = metadata_spec.offset >> LOG_BITS_IN_BYTE;
            let meta_addr = object.to_address() + offset;

            unsafe {
                if let Some(order) = atomic_ordering {
                    // if the optional mask is provided (e.g. for forwarding pointer), we need to use compare_exchange
                    if let Some(mask) = optional_mask {
                        loop {
                            let old_val = meta_addr.atomic_load::<AtomicU32>(order);
                            let new_val = (old_val & !(mask as u32)) | (val as u32 & (mask as u32));
                            if meta_addr
                                .compare_exchange::<AtomicU32>(old_val, new_val, order, order)
                                .is_ok()
                            {
                                break;
                            }
                        }
                    } else {
                        meta_addr.atomic_store::<AtomicU32>(val as u32, order);
                    }
                } else {
                    let val = if let Some(mask) = optional_mask {
                        let old_val = meta_addr.load::<u32>();
                        (old_val & !(mask as u32)) | (val as u32 & (mask as u32))
                    } else {
                        val as u32
                    };

                    meta_addr.store(val as u32);
                }
            }
        }
    }
}

#[inline(always)]
pub(crate) fn compare_exchange_metadata(
    metadata_spec: mmtk_meta::MetadataSpec,
    object: ObjectReference,
    old_metadata: usize,
    new_metadata: usize,
    optional_mask: Option<usize>,
    success_order: Ordering,
    failure_order: Ordering,
) -> bool {
    if metadata_spec.is_side_metadata {
        mmtk_meta::side_metadata::compare_exchange_atomic(
            metadata_spec,
            object.to_address(),
            old_metadata,
            new_metadata,
            success_order,
            failure_order,
        )
    } else {
        debug_assert!(optional_mask.is_none() || metadata_spec.num_of_bits >= 8,"optional_mask is only supported for 8X-bits in-header metadata. Problematic MetadataSpec: ({:?})", metadata_spec);

        if metadata_spec.num_of_bits < 8 {
            debug_assert!(
                (metadata_spec.offset >> LOG_BITS_IN_BYTE as isize)
                    == ((metadata_spec.offset + metadata_spec.num_of_bits as isize - 1)
                        >> LOG_BITS_IN_BYTE),
                "Metadata << 8-bits: ({:?}) stretches over two bytes!",
                metadata_spec
            );
            let offset = metadata_spec.offset >> LOG_BITS_IN_BYTE;
            let shift = metadata_spec.offset - (offset << LOG_BITS_IN_BYTE);
            let mask = ((1u8 << metadata_spec.num_of_bits) - 1) << shift;

            // let new_metadata = ((val as u8) << bit_shift);
            let meta_addr = object.to_address() + offset;
            unsafe {
                let real_old_val = meta_addr.atomic_load::<AtomicU8>(success_order);
                let expected_old_val = (real_old_val & !mask) | ((old_metadata as u8) << shift);
                let expected_new_val = (expected_old_val & !mask) | ((new_metadata as u8) << shift);
                meta_addr
                    .compare_exchange::<AtomicU8>(
                        expected_old_val,
                        expected_new_val,
                        success_order,
                        failure_order,
                    )
                    .is_ok()
            }
        } else if metadata_spec.num_of_bits == 8 {
            debug_assert!(
                metadata_spec.offset.trailing_zeros() as usize >= LOG_BITS_IN_BYTE as usize,
                "Metadata 8-bits: ({:?}) offset must be byte-aligned!",
                metadata_spec
            );
            let offset = metadata_spec.offset >> LOG_BITS_IN_BYTE;
            let meta_addr = object.to_address() + offset;

            let (old_metadata, new_metadata) = if let Some(mask) = optional_mask {
                let old_val = unsafe { meta_addr.atomic_load::<AtomicU8>(success_order) };
                let expected_new_val = (old_val & !(mask as u8)) | new_metadata as u8;
                let expected_old_val = (old_val & !(mask as u8)) | old_metadata as u8;
                (expected_old_val, expected_new_val)
            } else {
                (old_metadata as u8, new_metadata as u8)
            };

            unsafe {
                meta_addr
                    .compare_exchange::<AtomicU8>(
                        old_metadata,
                        new_metadata,
                        success_order,
                        failure_order,
                    )
                    .is_ok()
            }
        } else if metadata_spec.num_of_bits == 16 {
            debug_assert!(
                metadata_spec.offset.trailing_zeros() as usize >= LOG_BITS_IN_U16,
                "Metadata 16-bits: ({:?}) offset must be 2-bytes aligned!",
                metadata_spec
            );
            let offset = metadata_spec.offset >> LOG_BITS_IN_BYTE;
            let meta_addr = object.to_address() + offset;

            let (old_metadata, new_metadata) = if let Some(mask) = optional_mask {
                let old_val = unsafe { meta_addr.atomic_load::<AtomicU16>(success_order) };
                let expected_new_val = (old_val & !(mask as u16)) | new_metadata as u16;
                let expected_old_val = (old_val & !(mask as u16)) | old_metadata as u16;
                (expected_old_val, expected_new_val)
            } else {
                (old_metadata as u16, new_metadata as u16)
            };

            unsafe {
                meta_addr
                    .compare_exchange::<AtomicU16>(
                        old_metadata,
                        new_metadata,
                        success_order,
                        failure_order,
                    )
                    .is_ok()
            }
        } else if metadata_spec.num_of_bits == 32 {
            debug_assert!(
                metadata_spec.offset.trailing_zeros() as usize >= LOG_BITS_IN_WORD,
                "Metadata 32-bits: ({:?}) offset must be 4-bytes aligned!",
                metadata_spec
            );
            let offset = metadata_spec.offset >> LOG_BITS_IN_BYTE;
            let meta_addr = object.to_address() + offset;

            let (old_metadata, new_metadata) = if let Some(mask) = optional_mask {
                let old_val = unsafe { meta_addr.atomic_load::<AtomicU32>(success_order) };
                let expected_new_val = (old_val & !(mask as u32)) | new_metadata as u32;
                let expected_old_val = (old_val & !(mask as u32)) | old_metadata as u32;
                (expected_old_val, expected_new_val)
            } else {
                (old_metadata as u32, new_metadata as u32)
            };

            unsafe {
                meta_addr
                    .compare_exchange::<AtomicU32>(
                        old_metadata,
                        new_metadata,
                        success_order,
                        failure_order,
                    )
                    .is_ok()
            }
        } else {
            unreachable!()
        }
    }
}

#[inline(always)]
pub(crate) fn fetch_add_metadata(
    metadata_spec: mmtk_meta::MetadataSpec,
    object: ObjectReference,
    val: usize,
    order: Ordering,
) -> usize {
    if metadata_spec.is_side_metadata {
        mmtk_meta::side_metadata::fetch_add_atomic(metadata_spec, object.to_address(), val, order)
    } else {
        #[allow(clippy::collapsible_else_if)]
        if metadata_spec.num_of_bits < 8 {
            debug_assert!(
                (metadata_spec.offset >> LOG_BITS_IN_BYTE)
                    == ((metadata_spec.offset + metadata_spec.num_of_bits as isize - 1)
                        >> LOG_BITS_IN_BYTE),
                "Metadata << 8-bits: ({:?}) stretches over two bytes!",
                metadata_spec
            );
            let offset = metadata_spec.offset >> LOG_BITS_IN_BYTE;
            let shift = metadata_spec.offset - (offset << LOG_BITS_IN_BYTE);
            let mask = ((1u8 << metadata_spec.num_of_bits) - 1) << shift;

            // let new_metadata = ((val as u8) << bit_shift);
            let meta_addr = object.to_address() + offset;
            loop {
                unsafe {
                    let old_val = meta_addr.atomic_load::<AtomicU8>(order);
                    let old_metadata = (old_val & mask) >> shift;
                    // new_metadata may contain overflow and should be and with the mask
                    let new_metadata = (old_metadata + val as u8) & (mask >> shift);
                    let new_val = (old_val & !mask) | ((new_metadata as u8) << shift);
                    if meta_addr
                        .compare_exchange::<AtomicU8>(old_val, new_val, order, order)
                        .is_ok()
                    {
                        return old_metadata as usize;
                    }
                }
            }
        } else if metadata_spec.num_of_bits == 8 {
            debug_assert!(
                metadata_spec.offset.trailing_zeros() as usize >= LOG_BITS_IN_BYTE as usize,
                "Metadata 8-bits: ({:?}) offset must be byte-aligned!",
                metadata_spec
            );
            let offset = metadata_spec.offset >> LOG_BITS_IN_BYTE;

            unsafe {
                (*(object.to_address() + offset).to_ptr::<AtomicU8>())
                    .fetch_add(val as u8, order)
                    .into()
            }
        } else if metadata_spec.num_of_bits == 16 {
            debug_assert!(
                metadata_spec.offset.trailing_zeros() as usize >= LOG_BITS_IN_U16,
                "Metadata 16-bits: ({:?}) offset must be 2-bytes aligned!",
                metadata_spec
            );
            let offset = metadata_spec.offset >> LOG_BITS_IN_BYTE;

            unsafe {
                (*(object.to_address() + offset).to_ptr::<AtomicU16>())
                    .fetch_add(val as u16, order)
                    .into()
            }
        } else if metadata_spec.num_of_bits == 32 {
            debug_assert!(
                metadata_spec.offset.trailing_zeros() as usize >= LOG_BITS_IN_WORD,
                "Metadata 32-bits: ({:?}) offset must be 4-bytes aligned!",
                metadata_spec
            );
            let offset = metadata_spec.offset >> LOG_BITS_IN_BYTE;

            unsafe {
                (*(object.to_address() + offset).to_ptr::<AtomicU32>()).fetch_add(val as u32, order)
                    as usize
            }
        } else {
            unreachable!()
        }
    }
}

#[inline(always)]
pub(crate) fn fetch_sub_metadata(
    metadata_spec: mmtk_meta::MetadataSpec,
    object: ObjectReference,
    val: usize,
    order: Ordering,
) -> usize {
    if metadata_spec.is_side_metadata {
        mmtk_meta::side_metadata::fetch_sub_atomic(metadata_spec, object.to_address(), val, order)
    } else {
        #[allow(clippy::collapsible_else_if)]
        if metadata_spec.num_of_bits < 8 {
            debug_assert!(
                (metadata_spec.offset >> LOG_BITS_IN_BYTE)
                    == ((metadata_spec.offset + metadata_spec.num_of_bits as isize - 1)
                        >> LOG_BITS_IN_BYTE),
                "Metadata << 8-bits: ({:?}) stretches over two bytes!",
                metadata_spec
            );
            let offset = metadata_spec.offset >> LOG_BITS_IN_BYTE;
            let shift = metadata_spec.offset - (offset << LOG_BITS_IN_BYTE);
            let mask = ((1u8 << metadata_spec.num_of_bits) - 1) << shift;

            // let new_metadata = ((val as u8) << bit_shift);
            let meta_addr = object.to_address() + offset;
            loop {
                unsafe {
                    let old_val = meta_addr.atomic_load::<AtomicU8>(order);
                    let old_metadata = (old_val & mask) >> shift;
                    // new_metadata may contain overflow and should be and with the mask
                    let new_metadata = (old_metadata - val as u8) & (mask >> shift);
                    let new_val = (old_val & !mask) | ((new_metadata as u8) << shift);
                    if meta_addr
                        .compare_exchange::<AtomicU8>(old_val, new_val, order, order)
                        .is_ok()
                    {
                        return old_metadata as usize;
                    }
                }
            }
        } else if metadata_spec.num_of_bits == 8 {
            debug_assert!(
                metadata_spec.offset.trailing_zeros() as usize >= LOG_BITS_IN_BYTE as usize,
                "Metadata 8-bits: ({:?}) offset must be byte-aligned!",
                metadata_spec
            );
            let offset = metadata_spec.offset >> LOG_BITS_IN_BYTE;

            unsafe {
                (*(object.to_address() + offset).to_ptr::<AtomicU8>())
                    .fetch_sub(val as u8, order)
                    .into()
            }
        } else if metadata_spec.num_of_bits == 16 {
            debug_assert!(
                metadata_spec.offset.trailing_zeros() as usize >= LOG_BITS_IN_U16,
                "Metadata 16-bits: ({:?}) offset must be 2-bytes aligned!",
                metadata_spec
            );
            let offset = metadata_spec.offset >> LOG_BITS_IN_BYTE;

            unsafe {
                (*(object.to_address() + offset).to_ptr::<AtomicU16>())
                    .fetch_sub(val as u16, order)
                    .into()
            }
        } else if metadata_spec.num_of_bits == 32 {
            debug_assert!(
                metadata_spec.offset.trailing_zeros() as usize >= LOG_BITS_IN_WORD,
                "Metadata 32-bits: ({:?}) offset must be 4-bytes aligned!",
                metadata_spec
            );
            let offset = metadata_spec.offset >> LOG_BITS_IN_BYTE;

            unsafe {
                (*(object.to_address() + offset).to_ptr::<AtomicU32>()).fetch_sub(val as u32, order)
                    as usize
            }
        } else {
            unreachable!()
        }
    }
}
