use libc::*;
use std::convert::TryFrom;
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

/// This type represents a JikesRVM-level `ObjectReference`.
///
/// Currently, it has the same value as the MMTk-level `mmtk::util::address::ObjectReference`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct JikesObj(Address);

impl From<ObjectReference> for JikesObj {
    fn from(value: ObjectReference) -> Self {
        Self(value.to_raw_address())
    }
}

impl TryFrom<JikesObj> for ObjectReference {
    type Error = NullRefError;

    fn try_from(value: JikesObj) -> Result<Self, Self::Error> {
        ObjectReference::from_raw_address(value.0).ok_or(NullRefError)
    }
}

/// Error when trying to convert a null `JikesObj` to MMTk-level `ObjectReference` which cannot be
/// null.
#[derive(Debug)]
pub struct NullRefError;

impl std::fmt::Display for NullRefError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Attempt to convert null JikesObj to ObjectReference")
    }
}

impl std::error::Error for NullRefError {}

impl JikesObj {
    pub const NULL: Self = Self(Address::ZERO);

    /// Query the hashcode overhead of the current object.
    ///
    /// *   If `WHEN_COPIED` is true, return the overhead after copying;
    ///     otherwise return the current overhead.
    /// *   If `FRONT` is true, return the overhead at the start of the object;
    ///     otherwise return the overhead regardless of the location of the hash field.
    #[inline(always)]
    fn hashcode_overhead<const WHEN_COPIED: bool, const FRONT: bool>(&self) -> usize {
        if !MOVES_OBJECTS || !ADDRESS_BASED_HASHING || FRONT && DYNAMIC_HASH_OFFSET {
            // If the GC never moves object, just use the address as hashcode.
            // If not using address-based hashing, JikesRVM uses a 10-bit in-header hash field.
            // DYNAMIC_HASH_OFFSET puts hash field in the end.
            return 0;
        }

        let hash_state = self.get_status() & HASH_STATE_MASK;
        let has_hashcode = if WHEN_COPIED {
            // As long as it is hashed, it will have a hash field after moved.
            hash_state != HASH_STATE_UNHASHED
        } else {
            // An object has a hash field if and only if it is hashed and moved.
            hash_state == HASH_STATE_HASHED_AND_MOVED
        };

        if has_hashcode {
            HASHCODE_BYTES
        } else {
            0
        }
    }

    #[inline(always)]
    pub fn from_address(addr: Address) -> Self {
        JikesObj(addr)
    }

    #[inline(always)]
    pub fn to_address(&self) -> Address {
        self.0
    }

    #[inline(always)]
    pub fn from_objref_nullable(value: Option<ObjectReference>) -> Self {
        value.map_or(Self::NULL, Self::from)
    }

    #[inline(always)]
    pub fn is_null(&self) -> bool {
        self.0.is_zero()
    }

    #[inline(always)]
    pub fn load_tib(&self) -> TIB {
        TIB(unsafe { (self.0 + TIB_OFFSET).load::<Address>() })
    }

    #[inline(always)]
    pub fn load_rvm_type(&self) -> RVMType {
        let tib = self.load_tib();
        tib.load_rvm_type()
    }

    #[inline(always)]
    fn get_status(&self) -> usize {
        unsafe { (self.0 + STATUS_OFFSET).load::<usize>() }
    }

    #[inline(always)]
    fn set_status(&self, value: usize) {
        unsafe { (self.0 + STATUS_OFFSET).store::<usize>(value) }
    }

    #[inline(always)]
    pub fn get_current_size(&self) -> usize {
        let rvm_type = self.load_rvm_type();
        self.bytes_used(rvm_type)
    }

    #[inline(always)]
    fn get_size_when_copied(&self) -> usize {
        let rvm_type = self.load_rvm_type();
        self.bytes_required_when_copied(rvm_type)
    }

    #[inline(always)]
    pub fn get_align_when_copied(&self) -> usize {
        let rvm_type = self.load_rvm_type();

        if rvm_type.is_class() {
            rvm_type.get_alignment_class()
        } else {
            rvm_type.get_alignment_array()
        }
    }

    #[inline(always)]
    pub fn get_align_offset_when_copied(&self) -> usize {
        let rvm_type = self.load_rvm_type();

        if rvm_type.is_class() {
            self.get_offset_for_alignment_class()
        } else {
            self.get_offset_for_alignment_array()
        }
    }

    #[inline(always)]
    pub fn object_start(&self) -> Address {
        let start_to_objref = OBJECT_REF_OFFSET + self.hashcode_overhead::<false, true>() as isize;
        self.0.offset(-start_to_objref)
    }

    #[inline(always)]
    pub(crate) fn get_array_length(&self) -> usize {
        trace!("ObjectModel.get_array_length");
        unsafe { (self.0 + ARRAY_LENGTH_OFFSET).load::<usize>() }
    }

    #[inline(always)]
    fn bytes_required_when_copied(&self, rvm_type: RVMType) -> usize {
        trace!("VMObjectModel.bytes_required_when_copied");
        if rvm_type.is_class() {
            self.bytes_required_when_copied_class(rvm_type)
        } else {
            self.bytes_required_when_copied_array(rvm_type)
        }
    }

    #[inline(always)]
    fn bytes_required_when_copied_class(&self, rvm_type: RVMType) -> usize {
        let mut size = rvm_type.instance_size();
        trace!("bytes_required_when_copied_class: instance size={}", size);

        size += self.hashcode_overhead::<true, false>();

        trace!("bytes_required_when_copied_class: returned size={}", size);
        size
    }

    #[inline(always)]
    fn bytes_required_when_copied_array(&self, rvm_type: RVMType) -> usize {
        trace!("VMObjectModel.bytes_required_when_copied_array");
        let mut size = {
            let num_elements = self.get_array_length();
            let log_element_size = rvm_type.log_element_size();
            ARRAY_HEADER_SIZE + (num_elements << log_element_size)
        };

        size += self.hashcode_overhead::<true, false>();

        conversions::raw_align_up(size, BYTES_IN_INT)
    }

    #[inline(always)]
    fn bytes_used(&self, rvm_type: RVMType) -> usize {
        trace!("VMObjectModel.bytes_used");
        let is_class = rvm_type.is_class();
        let mut size = if is_class {
            rvm_type.instance_size()
        } else {
            let num_elements = self.get_array_length();
            let log_element_size = rvm_type.log_element_size();
            ARRAY_HEADER_SIZE + (num_elements << log_element_size)
        };

        size += self.hashcode_overhead::<false, false>();

        if is_class {
            size
        } else {
            conversions::raw_align_up(size, BYTES_IN_INT)
        }
    }

    #[inline(always)]
    fn get_offset_for_alignment_array(&self) -> usize {
        trace!("VMObjectModel.get_offset_for_alignment_array");
        ARRAY_HEADER_SIZE + self.hashcode_overhead::<true, true>()
    }

    #[inline(always)]
    fn get_offset_for_alignment_class(&self) -> usize {
        trace!("VMObjectModel.get_offset_for_alignment_class");
        SCALAR_HEADER_SIZE + self.hashcode_overhead::<true, true>()
    }

    #[inline(always)]
    pub fn get_referent(&self) -> JikesObj {
        unsafe { (self.0 + REFERENCE_REFERENT_FIELD_OFFSET).load::<JikesObj>() }
    }

    #[inline(always)]
    pub fn set_referent(&self, referent: JikesObj) {
        unsafe {
            (self.0 + REFERENCE_REFERENT_FIELD_OFFSET).store(referent);
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TIB(Address);

impl TIB {
    #[inline(always)]
    pub fn load_rvm_type(&self) -> RVMType {
        RVMType(unsafe { (self.0 + TIB_TYPE_INDEX * BYTES_IN_ADDRESS).load::<Address>() })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RVMType(Address);

impl RVMType {
    #[inline(always)]
    fn is_class(&self) -> bool {
        unsafe { (self.0 + IS_CLASS_TYPE_FIELD_OFFSET).load::<bool>() }
    }

    #[allow(dead_code)]
    #[inline(always)]
    fn is_array_type(&self) -> bool {
        unsafe { (self.0 + IS_ARRAY_TYPE_FIELD_OFFSET).load::<bool>() }
    }

    #[inline(always)]
    fn get_alignment_array(&self) -> usize {
        trace!("VMObjectModel.get_alignment_array");
        unsafe { (self.0 + RVM_ARRAY_ALIGNMENT_OFFSET).load::<usize>() }
    }

    #[inline(always)]
    fn get_alignment_class(&self) -> usize {
        trace!("VMObjectModel.get_alignment_class");
        if BYTES_IN_ADDRESS == BYTES_IN_DOUBLE {
            BYTES_IN_ADDRESS
        } else {
            unsafe { (self.0 + RVM_CLASS_ALIGNMENT_OFFSET).load::<usize>() }
        }
    }

    #[inline(always)]
    pub(crate) fn reference_offsets(&self) -> usize {
        unsafe { (self.0 + REFERENCE_OFFSETS_FIELD_OFFSET).load::<usize>() }
    }

    #[inline(always)]
    fn instance_size(&self) -> usize {
        unsafe { (self.0 + INSTANCE_SIZE_FIELD_OFFSET).load::<usize>() }
    }

    #[inline(always)]
    fn log_element_size(&self) -> usize {
        unsafe { (self.0 + LOG_ELEMENT_SIZE_FIELD_OFFSET).load::<usize>() }
    }
}

/// Used as a parameter of `move_object` to specify where to move an object to.
enum MoveTarget {
    /// Move an object to the address returned from `alloc_copy`.
    ToAddress(Address),
    /// Move an object to an `JikesObj` pointing to an object previously computed from
    /// `get_reference_when_copied_to`.
    ToObject(JikesObj),
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
        semantics: CopySemantics,
        copy_context: &mut GCWorkerCopyContext<JikesRVM>,
    ) -> ObjectReference {
        trace!("ObjectModel.copy");
        let jikes_from = JikesObj::from(from);
        let tib = jikes_from.load_tib();
        let rvm_type = tib.load_rvm_type();

        trace!("Is it a class?");
        let (bytes, align, offset) = if rvm_type.is_class() {
            trace!("... yes");
            let bytes = jikes_from.bytes_required_when_copied_class(rvm_type);
            let align = rvm_type.get_alignment_class();
            let offset = jikes_from.get_offset_for_alignment_class();
            (bytes, align, offset)
        } else {
            trace!("... no");
            let bytes = jikes_from.bytes_required_when_copied_array(rvm_type);
            let align = rvm_type.get_alignment_array();
            let offset = jikes_from.get_offset_for_alignment_array();
            (bytes, align, offset)
        };

        let addr = copy_context.alloc_copy(from, bytes, align, offset, semantics);
        debug_assert!(!addr.is_zero());

        let jikes_to_obj =
            Self::move_object(jikes_from, MoveTarget::ToAddress(addr), bytes, rvm_type);
        // jikes_to_obj must not be null because we gave it a non-zero `addr`.
        let to_obj = ObjectReference::try_from(jikes_to_obj).unwrap();

        copy_context.post_copy(to_obj, bytes, semantics);
        to_obj
    }

    #[inline(always)]
    fn copy_to(from: ObjectReference, to: ObjectReference, region: Address) -> Address {
        trace!("ObjectModel.copy_to");
        let jikes_from = JikesObj::from(from);
        let rvm_type = jikes_from.load_rvm_type();

        let copy = from != to;

        let bytes = if copy {
            let jikes_to = JikesObj::from(to);
            let bytes = jikes_from.bytes_required_when_copied(rvm_type);
            Self::move_object(jikes_from, MoveTarget::ToObject(jikes_to), bytes, rvm_type);
            bytes
        } else {
            jikes_from.bytes_used(rvm_type)
        };

        let start = Self::ref_to_object_start(to);
        fill_alignment_gap::<JikesRVM>(region, start);

        start + bytes
    }

    fn get_reference_when_copied_to(from: ObjectReference, to: Address) -> ObjectReference {
        trace!("ObjectModel.get_reference_when_copied_to");
        debug_assert!(!to.is_zero());

        let res_addr =
            to + OBJECT_REF_OFFSET + JikesObj::from(from).hashcode_overhead::<true, true>();
        debug_assert!(!res_addr.is_zero());
        let res_jikes = JikesObj(res_addr);
        // res cannot be null as long as res_addr is not zero.
        let res = ObjectReference::try_from(res_jikes).unwrap();
        res
    }

    fn get_current_size(object: ObjectReference) -> usize {
        trace!("ObjectModel.get_current_size");
        JikesObj::from(object).get_current_size()
    }

    fn get_size_when_copied(object: ObjectReference) -> usize {
        JikesObj::from(object).get_size_when_copied()
    }

    fn get_align_when_copied(object: ObjectReference) -> usize {
        trace!("ObjectModel.get_align_when_copied");
        JikesObj::from(object).get_align_when_copied()
    }

    fn get_align_offset_when_copied(object: ObjectReference) -> usize {
        trace!("ObjectModel.get_align_offset_when_copied");
        JikesObj::from(object).get_align_offset_when_copied()
    }

    fn get_type_descriptor(_reference: ObjectReference) -> &'static [i8] {
        unimplemented!()
    }

    #[inline(always)]
    fn ref_to_object_start(object: ObjectReference) -> Address {
        trace!("ObjectModel.object_start_ref");
        JikesObj::from(object).object_start()
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
    #[inline(always)]
    fn move_object(
        from_obj: JikesObj,
        mut to: MoveTarget,
        num_bytes: usize,
        _rvm_type: RVMType,
    ) -> JikesObj {
        trace!("VMObjectModel.move_object");

        // Default values
        let mut copy_bytes = num_bytes;
        let mut obj_ref_offset = OBJECT_REF_OFFSET;
        let mut status_word: usize = 0;
        let mut hash_state = HASH_STATE_UNHASHED;

        if ADDRESS_BASED_HASHING {
            // Read the hash state (used below)
            status_word = JikesObj::from(from_obj).get_status();
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

        let (to_address, to_obj) = match to {
            MoveTarget::ToAddress(addr) => {
                let obj = JikesObj(addr + obj_ref_offset);
                (addr, obj)
            }
            MoveTarget::ToObject(obj) => {
                let addr = obj.to_address() + (-obj_ref_offset);
                debug_assert!(obj.to_address() == addr + obj_ref_offset);
                (addr, obj)
            }
        };

        // Low memory word of source object
        let from_address = from_obj.to_address() + (-obj_ref_offset);

        // Do the copy// The hashcode is the first word, so we copy to object one word higher
        if let MoveTarget::ToAddress(ref mut addr) = to {
            *addr += HASHCODE_BYTES;
        }
        unsafe {
            Self::aligned_32_copy(to_address, from_address, copy_bytes);
        }

        // Do we need to copy the hash code?
        if hash_state == HASH_STATE_HASHED {
            unsafe {
                let hash_code = from_obj.to_address().as_usize() >> LOG_BYTES_IN_ADDRESS;
                if DYNAMIC_HASH_OFFSET {
                    (to_obj.to_address()
                        + num_bytes
                        + (-OBJECT_REF_OFFSET)
                        + (-(HASHCODE_BYTES as isize)))
                        .store::<usize>(hash_code);
                } else {
                    (to_obj.to_address() + HASHCODE_OFFSET)
                        .store::<usize>((hash_code << 1) | ALIGNMENT_MASK);
                }
                JikesObj::from(to_obj).set_status(status_word | HASH_STATE_HASHED_AND_MOVED);
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
        debug_assert!(copy_bytes & (BYTES_IN_INT - 1) == 0);
        debug_assert!(src.is_aligned_to(BYTES_IN_INT));
        debug_assert!(dst.is_aligned_to(BYTES_IN_INT));
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
}
