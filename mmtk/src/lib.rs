#![feature(llvm_asm)]
#![feature(vec_into_raw_parts)]
extern crate libc;
extern crate mmtk;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;

use mmtk::plan::PlanConstraints;
use mmtk::util::address::Address;
use mmtk::vm::VMBinding;
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

pub static mut JTOC_BASE: Address = Address::ZERO;

#[derive(Default)]
pub struct JikesRVM;

impl VMBinding for JikesRVM {
    type VMObjectModel = object_model::VMObjectModel;
    type VMScanning = scanning::VMScanning;
    type VMCollection = collection::VMCollection;
    type VMActivePlan = active_plan::VMActivePlan;
    type VMReferenceGlue = reference_glue::VMReferenceGlue;

    const ALLOC_END_ALIGNMENT: usize = 4;
}

impl JikesRVM {
    #[inline(always)]
    pub fn test(input: usize) -> usize {
        unsafe { jtoc_call!(TEST_METHOD_OFFSET, BOOT_THREAD, input) }
    }

    #[inline(always)]
    pub fn test1() -> usize {
        unsafe { jtoc_call!(TEST1_METHOD_OFFSET, BOOT_THREAD) }
    }

    #[inline(always)]
    pub fn test2(input1: usize, input2: usize) -> usize {
        unsafe { jtoc_call!(TEST2_METHOD_OFFSET, BOOT_THREAD, input1, input2) }
    }

    #[inline(always)]
    pub fn test3(input1: usize, input2: usize, input3: usize, input4: usize) -> usize {
        unsafe {
            jtoc_call!(
                TEST3_METHOD_OFFSET,
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

lazy_static! {
    pub static ref SINGLETON: MMTK<JikesRVM> = {
        #[cfg(feature = "nogc")]
        std::env::set_var("MMTK_PLAN", "NoGC");
        #[cfg(feature = "semispace")]
        std::env::set_var("MMTK_PLAN", "SemiSpace");
        #[cfg(feature = "marksweep")]
        std::env::set_var("MMTK_PLAN", "MarkSweep");

        MMTK::new()
    };
}
