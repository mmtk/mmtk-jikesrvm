extern crate libc;
extern crate mmtk;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;

use mmtk::plan::PlanConstraints;
use mmtk::util::address::Address;
use mmtk::vm::VMBinding;
use mmtk::MMTKBuilder;
use mmtk::MMTK;

use collection::BOOT_THREAD;
use entrypoint::*;

mod entrypoint;
mod unboxed_size_constants;
#[macro_use]
mod jtoc_call;
pub mod active_plan;
pub mod api;
pub mod boot_image_size;
pub mod class_loader_constants;
pub mod collection;
pub mod heap_layout_constants;
pub mod java_header;
pub mod java_header_constants;
pub mod java_size_constants;
pub mod memory_manager_constants;
pub mod misc_header_constants;
pub mod object_model;
pub mod reference_glue;
pub mod scan_boot_image;
pub mod scan_sanity;
pub mod scan_statics;
pub mod scanning;
pub mod tib_layout_constants;
pub(crate) mod vm_metadata;

pub static mut JTOC_BASE: Address = Address::ZERO;

#[derive(Default)]
pub struct JikesRVM;

/// The type of edges in JikesRVM.
///
/// TODO: We start with Address to ease the transition.
/// We should switch to the equivalent `mmtk::vm::edge_shape::SimpleEdge` later.
pub type JikesRVMEdge = Address;

impl VMBinding for JikesRVM {
    type VMObjectModel = object_model::VMObjectModel;
    type VMScanning = scanning::VMScanning;
    type VMCollection = collection::VMCollection;
    type VMActivePlan = active_plan::VMActivePlan;
    type VMReferenceGlue = reference_glue::VMReferenceGlue;

    type VMEdge = JikesRVMEdge;
    type VMMemorySlice = mmtk::vm::edge_shape::UnimplementedMemorySlice<JikesRVMEdge>;

    #[cfg(target_arch = "x86")]
    // On Intel we align code to 16 bytes as recommended in the optimization manual.
    const MAX_ALIGNMENT: usize = 1 << 4;

    const ALLOC_END_ALIGNMENT: usize = 4;
}

impl JikesRVM {
    #[inline(always)]
    pub fn mm_entrypoint_test(input1: usize, input2: usize, input3: usize, input4: usize) -> usize {
        use std::arch::asm;
        unsafe {
            jtoc_call!(
                MM_ENTRYPOINT_TEST_METHOD_OFFSET,
                BOOT_THREAD,
                input1,
                input2,
                input3,
                input4
            )
        }
    }
}

#[cfg(feature = "nogc")]
pub const SELECTED_CONSTRAINTS: PlanConstraints = mmtk::plan::NOGC_CONSTRAINTS;
#[cfg(feature = "semispace")]
pub const SELECTED_CONSTRAINTS: PlanConstraints = mmtk::plan::SS_CONSTRAINTS;
#[cfg(feature = "marksweep")]
pub const SELECTED_CONSTRAINTS: PlanConstraints = mmtk::plan::MS_CONSTRAINTS;

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
