use libc::c_void;
use libc::c_char;
use std::ffi::CStr;
use mmtk::memory_manager;
use mmtk::util::{Address, OpaquePointer, ObjectReference};
use mmtk::AllocationSemantics;
use mmtk::SelectedPlan;
use mmtk::Mutator;
use mmtk::Plan;
use mmtk::MMTK;
use mmtk::scheduler::*;
use JikesRVM;
use JTOC_BASE;
use SINGLETON;
use collection::BOOT_THREAD;
use collection::VMCollection;
use crate::scanning::PROCESS_EDGES_WORK_SIZE;

#[no_mangle]
pub extern "C" fn release_buffer(ptr: *mut Address) {
    let _vec = unsafe { Vec::<Address>::from_raw_parts(ptr, 0, PROCESS_EDGES_WORK_SIZE) };
}

#[no_mangle]
pub extern "C" fn jikesrvm_gc_init(jtoc: *mut c_void, heap_size: usize) {
    unsafe {
        JTOC_BASE = Address::from_mut_ptr(jtoc);
        BOOT_THREAD
            = OpaquePointer::from_address(VMCollection::thread_from_id(1));
    }
    let singleton_mut = unsafe { &mut *(&*SINGLETON as *const MMTK<JikesRVM> as *mut MMTK<JikesRVM>) };
    memory_manager::gc_init(singleton_mut, heap_size);
    debug_assert!(54 == JikesRVM::test(44));
    debug_assert!(112 == JikesRVM::test2(45, 67));
    debug_assert!(731 == JikesRVM::test3(21, 34, 9, 8));
}

#[no_mangle]
pub extern "C" fn start_control_collector(tls: OpaquePointer) {
    memory_manager::start_control_collector(&SINGLETON, tls);
}

#[no_mangle]
pub extern "C" fn bind_mutator(tls: OpaquePointer) -> *mut Mutator<SelectedPlan<JikesRVM>> {
    let box_mutator = memory_manager::bind_mutator(&SINGLETON, tls);
    Box::into_raw(box_mutator)
}

#[no_mangle]
pub extern "C" fn destroy_mutator(mutator: *mut Mutator<SelectedPlan<JikesRVM>>) {
    memory_manager::destroy_mutator(unsafe { Box::from_raw(mutator) })
}

#[no_mangle]
pub extern "C" fn alloc(mutator: *mut Mutator<SelectedPlan<JikesRVM>>, size: usize,
                           align: usize, offset: isize, allocator: AllocationSemantics) -> Address {
    memory_manager::alloc::<JikesRVM>(unsafe { &mut *mutator }, size, align, offset, allocator)
}

#[no_mangle]
pub extern "C" fn post_alloc(mutator: *mut Mutator<SelectedPlan<JikesRVM>>, refer: ObjectReference, type_refer: ObjectReference,
                                bytes: usize, allocator: AllocationSemantics) {
    memory_manager::post_alloc::<JikesRVM>(unsafe { &mut *mutator }, refer, type_refer, bytes, allocator)
}

#[no_mangle]
// For a syscall that returns bool, we have to return a i32 instead. See https://github.com/mmtk/mmtk-jikesrvm/issues/20
pub extern "C" fn will_never_move(object: ObjectReference) -> i32 {
    !object.is_movable() as i32
}

#[no_mangle]
pub extern "C" fn start_worker(tls: OpaquePointer, worker: *mut GCWorker<JikesRVM>) {
    memory_manager::start_worker::<JikesRVM>(tls, unsafe { worker.as_mut().unwrap() }, &SINGLETON)
}

#[no_mangle]
pub extern "C" fn enable_collection(tls: OpaquePointer) {
    memory_manager::enable_collection(&SINGLETON, tls)
}

#[no_mangle]
pub extern "C" fn used_bytes() -> usize {
    memory_manager::used_bytes(&SINGLETON)
}

#[no_mangle]
pub extern "C" fn free_bytes() -> usize {
    memory_manager::free_bytes(&SINGLETON)
}

#[no_mangle]
pub extern "C" fn total_bytes() -> usize {
    memory_manager::total_bytes(&SINGLETON)
}

#[no_mangle]
#[cfg(feature = "sanity")]
pub extern "C" fn scan_region() {
    memory_manager::scan_region(&SINGLETON)
}

#[no_mangle]
pub extern "C" fn handle_user_collection_request(tls: OpaquePointer) {
    memory_manager::handle_user_collection_request::<JikesRVM>(&SINGLETON, tls);
}

#[no_mangle]
// For a syscall that returns bool, we have to return a i32 instead. See https://github.com/mmtk/mmtk-jikesrvm/issues/20
pub extern "C" fn is_live_object(object: ObjectReference) -> i32 {
    object.is_live() as i32
}

#[no_mangle]
// For a syscall that returns bool, we have to return a i32 instead. See https://github.com/mmtk/mmtk-jikesrvm/issues/20
pub extern "C" fn is_mapped_object(object: ObjectReference) -> i32 {
    object.is_mapped() as i32
}

#[no_mangle]
// For a syscall that returns bool, we have to return a i32 instead. See https://github.com/mmtk/mmtk-jikesrvm/issues/20
pub extern "C" fn is_mapped_address(address: Address) -> i32 {
    address.is_mapped() as i32
}

#[no_mangle]
pub extern "C" fn modify_check(object: ObjectReference) {
    memory_manager::modify_check(&SINGLETON, object)
}

#[no_mangle]
pub extern "C" fn add_weak_candidate(reff: ObjectReference, referent: ObjectReference) {
    memory_manager::add_weak_candidate(&SINGLETON, reff, referent)
}

#[no_mangle]
pub extern "C" fn add_soft_candidate(reff: ObjectReference, referent: ObjectReference) {
    memory_manager::add_soft_candidate(&SINGLETON, reff, referent)
}

#[no_mangle]
pub extern "C" fn add_phantom_candidate(reff: ObjectReference, referent: ObjectReference) {
    memory_manager::add_phantom_candidate(&SINGLETON, reff, referent)
}

#[no_mangle]
pub extern "C" fn harness_begin(tls: OpaquePointer) {
    memory_manager::harness_begin(&SINGLETON, tls)
}

#[no_mangle]
pub extern "C" fn harness_end(tls: OpaquePointer) {
    memory_manager::harness_end(&SINGLETON)
}

#[no_mangle]
// For a syscall that returns bool, we have to return a i32 instead. See https://github.com/mmtk/mmtk-jikesrvm/issues/20
pub extern "C" fn process(name: *const c_char, value: *const c_char) -> i32 {
    let name_str: &CStr = unsafe { CStr::from_ptr(name) };
    let value_str: &CStr = unsafe { CStr::from_ptr(value) };
    memory_manager::process(&SINGLETON, name_str.to_str().unwrap(), value_str.to_str().unwrap()) as i32
}

#[no_mangle]
pub extern "C" fn starting_heap_address() -> Address {
    memory_manager::starting_heap_address()
}

#[no_mangle]
pub extern "C" fn last_heap_address() -> Address {
    memory_manager::last_heap_address()
}

// Allocation slow path

use mmtk::util::alloc::{BumpAllocator, LargeObjectAllocator};
use mmtk::util::alloc::Allocator as IAllocator;
use mmtk::util::heap::MonotonePageResource;

#[no_mangle]
pub extern "C" fn alloc_slow_bump_monotone_immortal(allocator: *mut c_void, size: usize, align: usize, offset:isize) -> Address {
    use mmtk::policy::immortalspace::ImmortalSpace;
    unsafe { &mut *(allocator as *mut BumpAllocator<JikesRVM>) }.alloc_slow(size, align, offset)
}

// For plans that do not include copy space, use the other implementation
// FIXME: after we remove plan as build-time option, we should remove this conditional compilation as well.

#[no_mangle]
#[cfg(any(feature = "semispace"))]
pub extern "C" fn alloc_slow_bump_monotone_copy(allocator: *mut c_void, size: usize, align: usize, offset:isize) -> Address {
    use mmtk::policy::copyspace::CopySpace;
    unsafe { &mut *(allocator as *mut BumpAllocator<JikesRVM>) }.alloc_slow(size, align, offset)
}
#[no_mangle]
#[cfg(not(any(feature = "semispace")))]
pub extern "C" fn alloc_slow_bump_monotone_copy(allocator: *mut c_void, size: usize, align: usize, offset:isize) -> Address {
    unimplemented!()
}

#[no_mangle]
pub extern "C" fn alloc_slow_largeobject(allocator: *mut c_void, size: usize, align: usize, offset:isize) -> Address {
    unsafe { &mut *(allocator as *mut LargeObjectAllocator<JikesRVM>) }.alloc_slow(size, align, offset)
}

// Test
// TODO: we should remove this?

#[no_mangle]
pub extern "C" fn test_stack_alignment() {
    info!("Entering stack alignment test with no args passed");
    unsafe {
        llvm_asm!("movaps %xmm1, (%esp)" : : : "sp", "%xmm1", "memory");
    }
    info!("Exiting stack alignment test");
}

#[no_mangle]
pub extern "C" fn test_stack_alignment1(a: usize, b: usize, c: usize, d: usize, e: usize) -> usize {
    info!("Entering stack alignment test");
    info!("a:{}, b:{}, c:{}, d:{}, e:{}",
          a, b, c, d, e);
    unsafe {
        llvm_asm!("movaps %xmm1, (%esp)" : : : "sp", "%xmm1", "memory");
    }
    let result = a + b * 2 + c * 3  + d * 4 + e * 5;
    info!("Exiting stack alignment test");
    result
}
