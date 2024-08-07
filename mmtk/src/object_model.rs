use libc::*;
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::unboxed_size_constants::*;
use crate::vm_metadata;
use mmtk::util::alloc::fill_alignment_gap;
use mmtk::util::conversions;
use mmtk::util::copy::*;
use mmtk::util::{Address, ObjectReference};
use mmtk::vm::*;

use entrypoint::*;
use java_header::*;
use java_header_constants::{
    ADDRESS_BASED_HASHING, ALIGNMENT_MASK, ARRAY_LENGTH_OFFSET, DYNAMIC_HASH_OFFSET,
    HASHCODE_BYTES, HASHCODE_OFFSET, HASH_STATE_HASHED, HASH_STATE_HASHED_AND_MOVED,
    HASH_STATE_MASK, HASH_STATE_UNHASHED,
};
use java_size_constants::{BYTES_IN_DOUBLE, BYTES_IN_INT};
use memory_manager_constants::*;
use tib_layout_constants::*;
use JikesRVM;

/// Used as a parameter of `move_object` to specify where to move an object to.
enum MoveTarget {
    /// Move an object to the address returned from `alloc_copy`.
    ToAddress(Address),
    /// Move an object to an `ObjectReference` pointing to an object previously computed from
    /// `get_reference_when_copied_to`.
    ToObject(ObjectReference),
}

/** Should we gather stats on hash code state transitions for address-based hashing? */
const HASH_STATS: bool = false;
/** count number of Object.hashCode() operations */
#[allow(dead_code)]
static HASH_REQUESTS: AtomicUsize = AtomicUsize::new(0);
/** count transitions from UNHASHED to HASHED */
#[allow(dead_code)]
static HASH_TRANSITION1: AtomicUsize = AtomicUsize::new(0);
/** count transitions from HASHED to HASHED_AND_MOVED */
#[allow(dead_code)]
static HASH_TRANSITION2: AtomicUsize = AtomicUsize::new(0);

// FIXME [ZC]: There are places we use Address::load<> instead of atomics operations
// where the memory location is indeed accessed by multiple threads (collector/mutator).
// Compiler optimizations and compiler/hardware reordering will affect the correctness of the
// emitted code.
// This is perhaps more serious with Rust release build or on machines with weaker memory models.

#[derive(Default)]
pub struct VMObjectModel {}

impl VMObjectModel {
    #[inline(always)]
    pub fn load_rvm_type(object: ObjectReference) -> Address {
        unsafe {
            let tib = (object.to_raw_address() + TIB_OFFSET).load::<Address>();
            (tib + TIB_TYPE_INDEX * BYTES_IN_ADDRESS).load::<Address>()
        }
    }

    #[inline(always)]
    pub fn load_tib(object: ObjectReference) -> Address {
        unsafe { (object.to_raw_address() + TIB_OFFSET).load::<Address>() }
    }

    #[allow(dead_code)]
    pub(crate) fn get_align_when_copied(object: ObjectReference) -> usize {
        trace!("ObjectModel.get_align_when_copied");
        let rvm_type = Self::load_rvm_type(object);

        if unsafe { (rvm_type + IS_ARRAY_TYPE_FIELD_OFFSET).load::<bool>() } {
            Self::get_alignment_array(rvm_type)
        } else {
            Self::get_alignment_class(rvm_type)
        }
    }

    #[allow(dead_code)]
    pub(crate) fn get_align_offset_when_copied(object: ObjectReference) -> usize {
        trace!("ObjectModel.get_align_offset_when_copied");
        let rvm_type = Self::load_rvm_type(object);

        if unsafe { (rvm_type + IS_ARRAY_TYPE_FIELD_OFFSET).load::<bool>() } {
            Self::get_offset_for_alignment_array(object, rvm_type)
        } else {
            Self::get_offset_for_alignment_class(object, rvm_type)
        }
    }

    #[inline(always)]
    pub(crate) fn get_array_length(object: ObjectReference) -> usize {
        trace!("ObjectModel.get_array_length");
        let len_addr = object.to_raw_address() + Self::get_array_length_offset();
        unsafe { len_addr.load::<usize>() }
    }

    pub(crate) fn get_array_length_offset() -> isize {
        ARRAY_LENGTH_OFFSET
    }
}

impl ObjectModel<JikesRVM> for VMObjectModel {
    const GLOBAL_LOG_BIT_SPEC: VMGlobalLogBitSpec = vm_metadata::LOGGING_SIDE_METADATA_SPEC;

    const LOCAL_FORWARDING_POINTER_SPEC: VMLocalForwardingPointerSpec =
        vm_metadata::FORWARDING_POINTER_METADATA_SPEC;
    const LOCAL_FORWARDING_BITS_SPEC: VMLocalForwardingBitsSpec =
        vm_metadata::FORWARDING_BITS_METADATA_SPEC;
    const LOCAL_MARK_BIT_SPEC: VMLocalMarkBitSpec = vm_metadata::MARKING_METADATA_SPEC;
    const LOCAL_LOS_MARK_NURSERY_SPEC: VMLocalLOSMarkNurserySpec = vm_metadata::LOS_METADATA_SPEC;

    #[inline(always)]
    fn copy(
        from: ObjectReference,
        copy: CopySemantics,
        copy_context: &mut GCWorkerCopyContext<JikesRVM>,
    ) -> ObjectReference {
        trace!("ObjectModel.copy");
        let tib = Self::load_tib(from);
        let rvm_type = Self::load_rvm_type(from);

        trace!("Is it a class?");
        if Self::is_class(rvm_type) {
            trace!("... yes");
            Self::copy_scalar(from, tib, rvm_type, copy, copy_context)
        } else {
            trace!("... no");
            Self::copy_array(from, tib, rvm_type, copy, copy_context)
        }
    }

    #[inline(always)]
    fn copy_to(from: ObjectReference, to: ObjectReference, region: Address) -> Address {
        trace!("ObjectModel.copy_to");
        let rvm_type = Self::load_rvm_type(from);

        let copy = from != to;

        let bytes = if copy {
            let bytes = Self::bytes_required_when_copied(from, rvm_type);
            Self::move_object(from, MoveTarget::ToObject(to), bytes, rvm_type);
            bytes
        } else {
            Self::bytes_used(from, rvm_type)
        };

        let start = Self::object_start_ref(to);
        fill_alignment_gap::<JikesRVM>(region, start);

        start + bytes
    }

    fn get_reference_when_copied_to(from: ObjectReference, to: Address) -> ObjectReference {
        trace!("ObjectModel.get_reference_when_copied_to");
        let mut res = to;
        if ADDRESS_BASED_HASHING && !DYNAMIC_HASH_OFFSET {
            unsafe {
                let hash_state =
                    (from.to_raw_address() + STATUS_OFFSET).load::<usize>() & HASH_STATE_MASK;
                if hash_state != HASH_STATE_UNHASHED {
                    res += HASHCODE_BYTES;
                }
            }
        }

        debug_assert!(!res.is_zero());
        unsafe { ObjectReference::from_raw_address_unchecked(res + OBJECT_REF_OFFSET) }
    }

    fn get_current_size(object: ObjectReference) -> usize {
        trace!("ObjectModel.get_current_size");
        let rvm_type = Self::load_rvm_type(object);

        Self::bytes_used(object, rvm_type)
    }

    fn get_size_when_copied(object: ObjectReference) -> usize {
        let rvm_type = Self::load_rvm_type(object);

        Self::bytes_required_when_copied(object, rvm_type)
    }

    fn get_align_when_copied(object: ObjectReference) -> usize {
        let rvm_type = Self::load_rvm_type(object);

        if Self::is_class(rvm_type) {
            Self::get_alignment_class(rvm_type)
        } else {
            Self::get_alignment_array(rvm_type)
        }
    }

    fn get_align_offset_when_copied(object: ObjectReference) -> usize {
        let rvm_type = Self::load_rvm_type(object);

        if Self::is_class(rvm_type) {
            Self::get_offset_for_alignment_class(object, rvm_type)
        } else {
            Self::get_offset_for_alignment_array(object, rvm_type)
        }
    }

    fn get_type_descriptor(_reference: ObjectReference) -> &'static [i8] {
        unimplemented!()
    }

    #[inline(always)]
    fn ref_to_object_start(object: ObjectReference) -> Address {
        trace!("ObjectModel.object_start_ref");
        // Easier to read if we do not collapse if here.
        #[allow(clippy::collapsible_if)]
        if MOVES_OBJECTS {
            if ADDRESS_BASED_HASHING && !DYNAMIC_HASH_OFFSET {
                let hash_state = unsafe {
                    (object.to_raw_address() + STATUS_OFFSET).load::<usize>() & HASH_STATE_MASK
                };
                if hash_state == HASH_STATE_HASHED_AND_MOVED {
                    return object.to_raw_address()
                        + (-(OBJECT_REF_OFFSET + HASHCODE_BYTES as isize));
                }
            }
        }
        object.to_raw_address() + (-OBJECT_REF_OFFSET)
    }

    #[inline(always)]
    fn ref_to_header(object: ObjectReference) -> Address {
        object.to_raw_address()
    }

    const OBJECT_REF_OFFSET_LOWER_BOUND: isize = OBJECT_REF_OFFSET;

    const IN_OBJECT_ADDRESS_OFFSET: isize = TIB_OFFSET;

    fn dump_object(_object: ObjectReference) {
        unimplemented!()
    }
}

impl VMObjectModel {
    fn is_class(rvm_type: Address) -> bool {
        unsafe { (rvm_type + IS_CLASS_TYPE_FIELD_OFFSET).load::<bool>() }
    }

    #[inline(always)]
    fn copy_scalar(
        from: ObjectReference,
        _tib: Address,
        rvm_type: Address,
        copy: CopySemantics,
        copy_context: &mut GCWorkerCopyContext<JikesRVM>,
    ) -> ObjectReference {
        trace!("VMObjectModel.copy_scalar");
        let bytes = Self::bytes_required_when_copied_class(from, rvm_type);
        let align = Self::get_alignment_class(rvm_type);
        let offset = Self::get_offset_for_alignment_class(from, rvm_type);
        let region = copy_context.alloc_copy(from, bytes, align, offset, copy);

        let to_obj = Self::move_object(from, MoveTarget::ToAddress(region), bytes, rvm_type);
        copy_context.post_copy(to_obj, bytes, copy);
        to_obj
    }

    #[inline(always)]
    fn copy_array(
        from: ObjectReference,
        _tib: Address,
        rvm_type: Address,
        copy: CopySemantics,
        copy_context: &mut GCWorkerCopyContext<JikesRVM>,
    ) -> ObjectReference {
        trace!("VMObjectModel.copy_array");
        let bytes = Self::bytes_required_when_copied_array(from, rvm_type);
        let align = Self::get_alignment_array(rvm_type);
        let offset = Self::get_offset_for_alignment_array(from, rvm_type);
        let region = copy_context.alloc_copy(from, bytes, align, offset, copy);

        let to_obj = Self::move_object(from, MoveTarget::ToAddress(region), bytes, rvm_type);
        copy_context.post_copy(to_obj, bytes, copy);
        // XXX: Do not sync icache/dcache because we do not support PowerPC
        to_obj
    }

    #[inline(always)]
    fn bytes_required_when_copied(object: ObjectReference, rvm_type: Address) -> usize {
        trace!("VMObjectModel.bytes_required_when_copied");
        if Self::is_class(rvm_type) {
            Self::bytes_required_when_copied_class(object, rvm_type)
        } else {
            Self::bytes_required_when_copied_array(object, rvm_type)
        }
    }

    #[inline(always)]
    fn bytes_required_when_copied_class(object: ObjectReference, rvm_type: Address) -> usize {
        let mut size = unsafe { (rvm_type + INSTANCE_SIZE_FIELD_OFFSET).load::<usize>() };
        trace!("bytes_required_when_copied_class: instance size={}", size);

        if ADDRESS_BASED_HASHING {
            let hash_state = unsafe {
                (object.to_raw_address() + STATUS_OFFSET).load::<usize>() & HASH_STATE_MASK
            };
            if hash_state != HASH_STATE_UNHASHED {
                size += HASHCODE_BYTES;
            }
        }

        trace!("bytes_required_when_copied_class: returned size={}", size);
        size
    }

    #[inline(always)]
    fn bytes_required_when_copied_array(object: ObjectReference, rvm_type: Address) -> usize {
        trace!("VMObjectModel.bytes_required_when_copied_array");
        let mut size = {
            let num_elements = Self::get_array_length(object);
            unsafe {
                let log_element_size = (rvm_type + LOG_ELEMENT_SIZE_FIELD_OFFSET).load::<usize>();
                // println!(
                //     "log_element_size(0x{:x}, 0x{:x}) -> 0x{:x} << 0x{:x}",
                //     object, rvm_type, num_elements, log_element_size
                // );
                ARRAY_HEADER_SIZE + (num_elements << log_element_size)
            }
        };

        if ADDRESS_BASED_HASHING {
            let hash_state = unsafe {
                (object.to_raw_address() + STATUS_OFFSET).load::<usize>() & HASH_STATE_MASK
            };
            if hash_state != HASH_STATE_UNHASHED {
                size += HASHCODE_BYTES;
            }
        }

        conversions::raw_align_up(size, BYTES_IN_INT)
    }

    #[inline(always)]
    fn bytes_used(object: ObjectReference, rvm_type: Address) -> usize {
        trace!("VMObjectModel.bytes_used");
        unsafe {
            let is_class = Self::is_class(rvm_type);
            let mut size = if is_class {
                (rvm_type + INSTANCE_SIZE_FIELD_OFFSET).load::<usize>()
            } else {
                let num_elements = Self::get_array_length(object);
                ARRAY_HEADER_SIZE
                    + (num_elements << (rvm_type + LOG_ELEMENT_SIZE_FIELD_OFFSET).load::<usize>())
            };

            // Easier to read if we do not collapse if here.
            #[allow(clippy::collapsible_if)]
            if MOVES_OBJECTS {
                if ADDRESS_BASED_HASHING {
                    let hash_state =
                        (object.to_raw_address() + STATUS_OFFSET).load::<usize>() & HASH_STATE_MASK;
                    if hash_state == HASH_STATE_HASHED_AND_MOVED {
                        size += HASHCODE_BYTES;
                    }
                }
            }

            if is_class {
                size
            } else {
                conversions::raw_align_up(size, BYTES_IN_INT)
            }
        }
    }

    #[inline(always)]
    fn move_object(
        from_obj: ObjectReference,
        mut to: MoveTarget,
        num_bytes: usize,
        _rvm_type: Address,
    ) -> ObjectReference {
        trace!("VMObjectModel.move_object");

        // Default values
        let mut copy_bytes = num_bytes;
        let mut obj_ref_offset = OBJECT_REF_OFFSET;
        let mut status_word: usize = 0;
        let mut hash_state = HASH_STATE_UNHASHED;

        if ADDRESS_BASED_HASHING {
            unsafe {
                // Read the hash state (used below)
                status_word = (from_obj.to_raw_address() + STATUS_OFFSET).load::<usize>();
                hash_state = status_word & HASH_STATE_MASK;
                if hash_state == HASH_STATE_HASHED {
                    // We do not copy the hashcode, but we do allocate it
                    copy_bytes -= HASHCODE_BYTES;

                    if !DYNAMIC_HASH_OFFSET {
                        // The hashcode is the first word, so we copy to object one word higher
                        if let MoveTarget::ToAddress(ref mut addr) = to {
                            *addr += HASHCODE_BYTES;
                        }
                    }
                } else if !DYNAMIC_HASH_OFFSET && hash_state == HASH_STATE_HASHED_AND_MOVED {
                    // Simple operation (no hash state change), but one word larger header
                    obj_ref_offset += HASHCODE_BYTES as isize;
                }
            }
        }

        let (to_address, to_obj) = match to {
            MoveTarget::ToAddress(addr) => {
                let obj =
                    unsafe { ObjectReference::from_raw_address_unchecked(addr + obj_ref_offset) };
                (addr, obj)
            }
            MoveTarget::ToObject(obj) => {
                let addr = obj.to_raw_address() + (-obj_ref_offset);
                debug_assert!(obj.to_raw_address() == addr + obj_ref_offset);
                (addr, obj)
            }
        };

        // Low memory word of source object
        let from_address = from_obj.to_raw_address() + (-obj_ref_offset);

        // Do the copy
        unsafe {
            Self::aligned_32_copy(to_address, from_address, copy_bytes);
        }

        // Do we need to copy the hash code?
        if hash_state == HASH_STATE_HASHED {
            unsafe {
                let hash_code = from_obj.to_raw_address().as_usize() >> LOG_BYTES_IN_ADDRESS;
                if DYNAMIC_HASH_OFFSET {
                    (to_obj.to_raw_address()
                        + num_bytes
                        + (-OBJECT_REF_OFFSET)
                        + (-(HASHCODE_BYTES as isize)))
                        .store::<usize>(hash_code);
                } else {
                    (to_obj.to_raw_address() + HASHCODE_OFFSET)
                        .store::<usize>((hash_code << 1) | ALIGNMENT_MASK);
                }
                (to_obj.to_raw_address() + STATUS_OFFSET)
                    .store::<usize>(status_word | HASH_STATE_HASHED_AND_MOVED);
                if HASH_STATS {
                    HASH_TRANSITION2.fetch_add(1, Ordering::Relaxed);
                }
            }
        }

        to_obj
    }

    #[inline(always)]
    unsafe fn aligned_32_copy(dst: Address, src: Address, copy_bytes: usize) {
        trace!("VMObjectModel.aligned_32_copy");
        //debug_assert!(copy_bytes >= 0);
        debug_assert!(copy_bytes & (BYTES_IN_INT - 1) == 0);
        debug_assert!(src.is_aligned_to(BYTES_IN_INT));
        debug_assert!(src.is_aligned_to(BYTES_IN_INT));
        debug_assert!(src + copy_bytes <= dst || src >= dst + BYTES_IN_INT);

        let cnt = copy_bytes;
        let src_end = src + cnt;
        let dst_end = dst + cnt;
        let overlap = src_end > dst && dst_end > src;
        if overlap {
            memmove(dst.to_mut_ptr(), src.to_mut_ptr(), cnt);
        } else {
            memcpy(dst.to_mut_ptr(), src.to_mut_ptr(), cnt);
        }
    }

    #[inline(always)]
    fn get_alignment_array(rvm_type: Address) -> usize {
        trace!("VMObjectModel.get_alignment_array");
        unsafe { (rvm_type + RVM_ARRAY_ALIGNMENT_OFFSET).load::<usize>() }
    }

    #[inline(always)]
    fn get_alignment_class(rvm_type: Address) -> usize {
        trace!("VMObjectModel.get_alignment_class");
        if BYTES_IN_ADDRESS == BYTES_IN_DOUBLE {
            BYTES_IN_ADDRESS
        } else {
            unsafe { (rvm_type + RVM_CLASS_ALIGNMENT_OFFSET).load::<usize>() }
        }
    }

    #[inline(always)]
    fn get_offset_for_alignment_array(object: ObjectReference, _rvm_type: Address) -> usize {
        trace!("VMObjectModel.get_offset_for_alignment_array");
        let mut offset = OBJECT_REF_OFFSET as usize;

        if ADDRESS_BASED_HASHING && !DYNAMIC_HASH_OFFSET {
            let hash_state = unsafe {
                (object.to_raw_address() + STATUS_OFFSET).load::<usize>() & HASH_STATE_MASK
            };
            if hash_state != HASH_STATE_UNHASHED {
                offset += HASHCODE_BYTES;
            }
        }

        offset
    }

    #[inline(always)]
    fn get_offset_for_alignment_class(object: ObjectReference, _rvm_type: Address) -> usize {
        trace!("VMObjectModel.get_offset_for_alignment_class");
        let mut offset = SCALAR_HEADER_SIZE;

        if ADDRESS_BASED_HASHING && !DYNAMIC_HASH_OFFSET {
            let hash_state = unsafe {
                (object.to_raw_address() + STATUS_OFFSET).load::<usize>() & HASH_STATE_MASK
            };
            if hash_state != HASH_STATE_UNHASHED {
                offset += HASHCODE_BYTES
            }
        }

        offset
    }

    #[inline(always)]
    fn object_start_ref(object: ObjectReference) -> Address {
        trace!("ObjectModel.object_start_ref");
        // Easier to read if we do not collapse if here.
        #[allow(clippy::collapsible_if)]
        if MOVES_OBJECTS {
            if ADDRESS_BASED_HASHING && !DYNAMIC_HASH_OFFSET {
                let hash_state = unsafe {
                    (object.to_raw_address() + STATUS_OFFSET).load::<usize>() & HASH_STATE_MASK
                };
                if hash_state == HASH_STATE_HASHED_AND_MOVED {
                    return object.to_raw_address()
                        + (-(OBJECT_REF_OFFSET + HASHCODE_BYTES as isize));
                }
            }
        }
        object.to_raw_address() + (-OBJECT_REF_OFFSET)
    }
}
