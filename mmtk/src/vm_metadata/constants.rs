use std::usize;

use mmtk::util::metadata::side_metadata::{
    metadata_address_range_size, metadata_bytes_per_chunk, SideMetadataSpec,
    GLOBAL_SIDE_METADATA_BASE_ADDRESS,
};
use mmtk::util::metadata::{HeaderMetadataSpec, MetadataSpec};

use crate::java_header::AVAILABLE_BITS_OFFSET;
pub(crate) use mmtk::util::constants::{
    BITS_IN_BYTE, LOG_BITS_IN_BYTE, LOG_BITS_IN_WORD, LOG_MIN_OBJECT_SIZE,
};

const FORWARDING_BITS_OFFSET: isize = AVAILABLE_BITS_OFFSET << LOG_BITS_IN_BYTE;

/// Return the space size of the given metadata spec.
/// As mmtk-jikesrvm only supports 32-bits:
///  - For global metadata, the space size is the whole address range size, and
///  - For local metadata, the space size is the size of metadata space needed per chunk.
///
#[cfg(not(debug_assert))]
const fn side_metadata_size(metadata_spec: MetadataSpec) -> usize {
    match metadata_spec {
        MetadataSpec::OnSide(s) => {
            if s.is_global {
                metadata_address_range_size(&s)
            } else {
                metadata_bytes_per_chunk(s.log_min_obj_size, s.log_num_of_bits)
            }
        }
        // this match arm is unreachable, but for a const function unreachable is not allowed
        _ => 0,
    }
}

#[cfg(debug_assert)]
fn side_metadata_size(metadata_spec: MetadataSpec) -> usize {
    debug_assert!(metadata_spec.is_on_side());
    match metadata_spec {
        MetadataSpec::OnSide(s) => {
            if s.is_global {
                metadata_address_range_size(&s)
            } else {
                metadata_bytes_per_chunk(s.log_min_obj_size, s.log_num_of_bits)
            }
        }
        // this match arm is unreachable, but for a const function unreachable is not allowed
        _ => {
            unreachable!()
        },
    }
}

pub(crate) const LOG_BITS_IN_U16: usize = 4;

// Global MetadataSpecs - Start

/// Global logging bit metadata spec
/// 1 bit per object
pub(crate) const LOGGING_SIDE_METADATA_SPEC: MetadataSpec =
    MetadataSpec::OnSide(SideMetadataSpec {
        is_global: true,
        offset: GLOBAL_SIDE_METADATA_BASE_ADDRESS.as_usize(),
        log_num_of_bits: 0,
        log_min_obj_size: LOG_MIN_OBJECT_SIZE as usize,
    });

// Global MetadataSpecs - End

// PolicySpecific MetadataSpecs - Start

/// PolicySpecific forwarding pointer metadata spec
/// 1 word per object
pub(crate) const FORWARDING_POINTER_METADATA_SPEC: MetadataSpec =
    MetadataSpec::InHeader(HeaderMetadataSpec {
        bit_offset: FORWARDING_BITS_OFFSET,
        num_of_bits: 1 << LOG_BITS_IN_WORD,
    });

/// PolicySpecific object forwarding status metadata spec
/// 2 bits per object
pub(crate) const FORWARDING_BITS_METADATA_SPEC: MetadataSpec =
    MetadataSpec::InHeader(HeaderMetadataSpec {
        bit_offset: FORWARDING_BITS_OFFSET,
        num_of_bits: 2,
    });

/// PolicySpecific mark bit metadata spec
/// 1 bit per object
pub(crate) const MARKING_METADATA_SPEC: MetadataSpec = MetadataSpec::InHeader(HeaderMetadataSpec {
    bit_offset: FORWARDING_BITS_OFFSET,
    num_of_bits: 1,
});

/// PolicySpecific mark-and-nursery bits metadata spec
/// 2-bits per object
pub(crate) const LOS_METADATA_SPEC: MetadataSpec = MetadataSpec::InHeader(HeaderMetadataSpec {
    bit_offset: FORWARDING_BITS_OFFSET,
    num_of_bits: 2,
});

// TODO: This is not used now, but probably needs to be double checked before being used.
pub(crate) const UNLOGGED_SIDE_METADATA_SPEC: MetadataSpec =
    MetadataSpec::OnSide(SideMetadataSpec {
        is_global: false,
        offset: 0,
        log_num_of_bits: 0,
        log_min_obj_size: LOG_MIN_OBJECT_SIZE as usize,
    });

// PolicySpecific MetadataSpecs - End
