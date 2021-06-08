use mmtk::util::metadata::MetadataSpec;
use mmtk::util::metadata::{
    metadata_address_range_size, metadata_bytes_per_chunk, GLOBAL_SIDE_METADATA_BASE_ADDRESS,
};

use crate::java_header::AVAILABLE_BITS_OFFSET;

const fn side_metadata_size(metadata_spec: MetadataSpec) -> usize {
    if metadata_spec.is_global {
        metadata_address_range_size(metadata_spec)
    } else {
        metadata_bytes_per_chunk(metadata_spec.log_min_obj_size, metadata_spec.num_of_bits)
    }
}

// We only support 32-bits in JikesRVM
pub(crate) const LOG_BITS_IN_WORD: usize = 5;
pub(crate) const LOG_MIN_OBJECT_SIZE: usize = 2;

// Global MetadataSpecs
pub(crate) const LOGGING_SIDE_METADATA_SPEC: MetadataSpec = MetadataSpec {
    is_side_metadata: true,
    is_global: true,
    offset: GLOBAL_SIDE_METADATA_BASE_ADDRESS.as_usize(),
    num_of_bits: 1,
    log_min_obj_size: 3,
};

// PolicySpecific MetadataSpecs
pub(crate) const FORWARDING_POINTER_METADATA_SPEC: MetadataSpec = MetadataSpec {
    is_side_metadata: false,
    is_global: false,
    offset: 0,
    num_of_bits: 1 << LOG_BITS_IN_WORD,
    log_min_obj_size: LOG_MIN_OBJECT_SIZE,
};

pub(crate) const FORWARDING_BITS_SIDE_METADATA_SPEC: MetadataSpec = MetadataSpec {
    is_side_metadata: false,
    is_global: false,
    offset: AVAILABLE_BITS_OFFSET as isize,
    num_of_bits: 2,
    log_min_obj_size: LOG_MIN_OBJECT_SIZE,
};

pub(crate) const MARKING_SIDE_METADATA_SPEC: MetadataSpec = MetadataSpec {
    is_side_metadata: false,
    is_global: false,
    offset: AVAILABLE_BITS_OFFSET,
    num_of_bits: 1,
    log_min_obj_size: LOG_MIN_OBJECT_SIZE,
};

pub(crate) const LOS_SIDE_METADATA_SPEC: MetadataSpec = MetadataSpec {
    is_side_metadata: false,
    is_global: false,
    offset: AVAILABLE_BITS_OFFSET,
    num_of_bits: 2,
    log_min_obj_size: LOG_MIN_OBJECT_SIZE,
};

// TODO: This is not used now, but probably needs to be double checked before being used.
pub(crate) const UNLOGGED_SIDE_METADATA_SPEC: MetadataSpec = MetadataSpec {
    is_side_metadata: true,
    is_global: false,
    offset: 0,
    num_of_bits: 1,
    log_min_obj_size: LOG_MIN_OBJECT_SIZE as usize,
};

pub(crate) const LAST_GLOBAL_SIDE_METADATA_OFFSET: usize =
    GLOBAL_SIDE_METADATA_BASE_ADDRESS.as_usize() + side_metadata_size(LOGGING_SIDE_METADATA_SPEC);

pub(crate) const LAST_LOCAL_SIDE_METADATA_OFFSET: usize =
    UNLOGGED_SIDE_METADATA_SPEC.offset as usize + side_metadata_size(UNLOGGED_SIDE_METADATA_SPEC);
