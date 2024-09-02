extern crate libc;
extern crate mmtk;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;

use mmtk::plan::PlanConstraints;
use mmtk::util::address::Address;
use mmtk::util::ObjectReference;
use mmtk::vm::slot::Slot;
use mmtk::vm::VMBinding;
use mmtk::MMTKBuilder;
use mmtk::MMTK;

use collection::BOOT_THREAD;
use object_model::JikesObj;

pub mod active_plan;
pub mod api;
pub mod boot_image_size;
pub mod class_loader_constants;
pub mod collection;
mod entrypoint;
pub mod heap_layout_constants;
pub mod java_header;
pub mod java_header_constants;
pub mod java_size_constants;
pub mod jikesrvm_calls;
pub mod memory_manager_constants;
pub mod misc_header_constants;
pub mod object_model;
pub mod reference_glue;
pub mod scan_boot_image;
pub mod scan_sanity;
pub mod scan_statics;
pub mod scanning;
pub mod tib_layout_constants;
mod unboxed_size_constants;
pub(crate) mod vm_metadata;

pub static mut JTOC_BASE: Address = Address::ZERO;

#[derive(Default)]
pub struct JikesRVM;

/// The type of slots in JikesRVM.
///
/// Each slot holds a `JikesObj` value, which is equal to the JikesRVM-level `ObjectReference`. The
/// Java parts of the binding may insert `Address` values into native arrays of `JikesRVMSlot`
/// passed from Rust code, so this type has `repr(transparent)` to make sure it has the same layout
/// as `Address`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct JikesRVMSlot(Address);

impl JikesRVMSlot {
    pub fn from_address(address: Address) -> Self {
        Self(address)
    }

    pub fn to_address(&self) -> Address {
        self.0
    }
}

impl Slot for JikesRVMSlot {
    fn load(&self) -> Option<ObjectReference> {
        let jikes_obj = JikesObj::from_address(unsafe { self.0.load::<Address>() });
        ObjectReference::try_from(jikes_obj).ok()
    }

    fn store(&self, object: ObjectReference) {
        let jikes_obj = JikesObj::from(object);
        unsafe {
            self.0.store::<Address>(jikes_obj.to_address());
        }
    }
}

impl VMBinding for JikesRVM {
    type VMObjectModel = object_model::VMObjectModel;
    type VMScanning = scanning::VMScanning;
    type VMCollection = collection::VMCollection;
    type VMActivePlan = active_plan::VMActivePlan;
    type VMReferenceGlue = reference_glue::VMReferenceGlue;

    type VMSlot = JikesRVMSlot;
    type VMMemorySlice = mmtk::vm::slot::UnimplementedMemorySlice<JikesRVMSlot>;

    #[cfg(target_arch = "x86")]
    // On Intel we align code to 16 bytes as recommended in the optimization manual.
    const MAX_ALIGNMENT: usize = 1 << 4;

    const ALLOC_END_ALIGNMENT: usize = 4;
}

impl JikesRVM {
    #[inline(always)]
    pub fn mm_entrypoint_test(input1: usize, input2: usize, input3: usize, input4: usize) -> usize {
        let boot_thread = unsafe { BOOT_THREAD };
        jikesrvm_calls::mm_entrypoint_test(boot_thread, input1, input2, input3, input4)
    }
}

#[cfg(feature = "nogc")]
pub const SELECTED_CONSTRAINTS: PlanConstraints = mmtk::plan::NOGC_CONSTRAINTS;
#[cfg(feature = "semispace")]
pub const SELECTED_CONSTRAINTS: PlanConstraints = mmtk::plan::SS_CONSTRAINTS;
#[cfg(feature = "marksweep")]
pub const SELECTED_CONSTRAINTS: PlanConstraints = mmtk::plan::MS_CONSTRAINTS;

use std::convert::TryFrom;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::Mutex;

pub static MMTK_INITIALIZED: AtomicBool = AtomicBool::new(false);

lazy_static! {
    pub static ref BUILDER: Mutex<MMTKBuilder> = Mutex::new(MMTKBuilder::new());
    pub static ref SINGLETON: MMTK<JikesRVM> = {
        let builder = BUILDER.lock().unwrap();
        assert!(!MMTK_INITIALIZED.load(Ordering::SeqCst));
        let ret = mmtk::memory_manager::mmtk_init(&builder);
        MMTK_INITIALIZED.store(true, std::sync::atomic::Ordering::SeqCst);
        *ret
    };
}
