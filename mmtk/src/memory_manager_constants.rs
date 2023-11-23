use crate::unboxed_size_constants::*;
use crate::SELECTED_CONSTRAINTS;

/** {@code true} if the selected plan needs support for linearly scanning the heap */
pub const NEEDS_LINEAR_SCAN: bool = SELECTED_CONSTRAINTS.needs_linear_scan;
/** {@code true} if the selected plan requires concurrent worker threads */
pub const NEEDS_CONCURRENT_WORKERS: bool = SELECTED_CONSTRAINTS.needs_concurrent_workers;
/** {@code true} if the selected plan needs support for generating a GC trace */
pub const GENERATE_GC_TRACE: bool = false;
/** {@code true} if the selected plan may move objects */
pub const MOVES_OBJECTS: bool = SELECTED_CONSTRAINTS.moves_objects;
/** {@code true} if the selected plan moves TIB objects */
pub const MOVES_TIBS: bool = false;
/** {@code true} if the selected plan moves code */
pub const MOVES_CODE: bool = false;

#[cfg(feature = "nogc")]
pub const NUM_SPECIALIZED_SCANS: usize = 0;
#[cfg(any(feature = "semispace", feature = "marksweep"))]
pub const NUM_SPECIALIZED_SCANS: usize = 1;

pub const GC_HEADER_BITS: usize = 2;
pub const GC_HEADER_BYTES: usize = 0 << LOG_BYTES_IN_WORD;
