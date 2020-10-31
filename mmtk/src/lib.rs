#![feature(llvm_asm)]
#[macro_use]
extern crate mmtk;
extern crate libc;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;

use mmtk::util::address::Address;
use mmtk::util::OpaquePointer;
use mmtk::TraceLocal;
use mmtk::vm::VMBinding;
use mmtk::MMTK;

use entrypoint::*;
use collection::BOOT_THREAD;

mod entrypoint;
#[macro_use]
mod jtoc_call;
pub mod scanning;
pub mod collection;
pub mod object_model;
pub mod java_header;
pub mod java_size_constants;
pub mod java_header_constants;
pub mod memory_manager_constants;
pub mod misc_header_constants;
pub mod tib_layout_constants;
pub mod class_loader_constants;
pub mod scan_statics;
pub mod scan_boot_image;
pub mod active_plan;
pub mod heap_layout_constants;
pub mod boot_image_size;
pub mod scan_sanity;
pub mod reference_glue;
pub mod api;

pub static mut JTOC_BASE: Address = Address::ZERO;

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
        unsafe {
            jtoc_call!(TEST_METHOD_OFFSET, BOOT_THREAD, input)
        }
    }

    #[inline(always)]
    pub fn test1() -> usize {
        unsafe {
            jtoc_call!(TEST1_METHOD_OFFSET, BOOT_THREAD)
        }
    }

    #[inline(always)]
    pub fn test2(input1: usize, input2: usize) -> usize {
        unsafe {
            jtoc_call!(TEST2_METHOD_OFFSET, BOOT_THREAD, input1, input2)
        }
    }

    #[inline(always)]
    pub fn test3(input1: usize, input2: usize, input3: usize, input4: usize) -> usize {
        unsafe {
            jtoc_call!(TEST3_METHOD_OFFSET, BOOT_THREAD, input1, input2, input3, input4)
        }
    }

    pub fn enter_vm(tls: OpaquePointer) -> i32 {
        unsafe {
            jtoc_call!(ENTER_VM_METHOD_OFFSET, tls) as i32
        }
    }

    pub fn leave_vm(tls: OpaquePointer, status: i32) {
        unsafe {
            jtoc_call!(LEAVE_VM_METHOD_OFFSET, tls, status);
        }
    }
}

lazy_static! {
    pub static ref SINGLETON: MMTK<JikesRVM> = MMTK::new();
}