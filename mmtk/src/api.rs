use crate::scanning::EDGES_BUFFER_CAPACITY;
use collection::VMCollection;
use collection::BOOT_THREAD;
use libc::c_char;
use libc::c_void;
use mmtk::memory_manager;
use mmtk::scheduler::*;
use mmtk::util::opaque_pointer::*;
use mmtk::util::{Address, ObjectReference};
use mmtk::vm::{ReferenceGlue, VMBinding};
use mmtk::AllocationSemantics;
use mmtk::Mutator;
use std::ffi::CStr;
use std::sync::atomic::Ordering;
use JikesRVM;
use BUILDER;
use JTOC_BASE;
use SINGLETON;

/// # Safety
/// Caller needs to make sure the ptr is a valid vector pointer.
#[no_mangle]
pub unsafe extern "C" fn release_buffer(ptr: *mut Address) {
    let _vec = Vec::<Address>::from_raw_parts(ptr, 0, EDGES_BUFFER_CAPACITY);
}

#[no_mangle]
pub extern "C" fn jikesrvm_gc_init(jtoc: *mut c_void, heap_size: usize) {
    unsafe {
        JTOC_BASE = Address::from_mut_ptr(jtoc);
        BOOT_THREAD = OpaquePointer::from_address(VMCollection::thread_from_id(1));
    }

    {
        use mmtk::util::options::PlanSelector;
        // set heap size
        let mut builder = BUILDER.lock().unwrap();
        let success =
            builder
                .options
                .gc_trigger
                .set(mmtk::util::options::GCTriggerSelector::FixedHeapSize(
                    heap_size,
                ));
        assert!(success, "Failed to set heap size to {}", heap_size);

        // set plan based on features.
        let plan = if cfg!(feature = "nogc") {
            PlanSelector::NoGC
        } else if cfg!(feature = "semispace") {
            PlanSelector::SemiSpace
        } else if cfg!(feature = "marksweep") {
            PlanSelector::MarkSweep
        } else {
            panic!("No plan feature is enabled for JikesRVM. JikesRVM requires one plan feature to build.")
        };
        let success = builder.options.plan.set(plan);
        assert!(success, "Failed to set plan to {:?}", plan);

        // set vm space
        builder
            .options
            .vm_space_start
            .set(unsafe { Address::from_usize(0x6000_0000) });
        builder.options.vm_space_size.set(0x800_0000);
    }

    // Make sure that we haven't initialized MMTk (by accident) yet
    assert!(!crate::MMTK_INITIALIZED.load(Ordering::Relaxed));
    // Make sure we initialize MMTk here
    lazy_static::initialize(&SINGLETON);
}

#[no_mangle]
pub extern "C" fn bind_mutator(tls: VMMutatorThread) -> *mut Mutator<JikesRVM> {
    let box_mutator = memory_manager::bind_mutator(&SINGLETON, tls);
    Box::into_raw(box_mutator)
}

#[no_mangle]
// It is fine we turn the pointer back to box, as we turned a boxed value to the raw pointer in bind_mutator()
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn destroy_mutator(mutator: *mut Mutator<JikesRVM>) {
    memory_manager::destroy_mutator(unsafe { &mut *mutator })
}

#[no_mangle]
// We trust the mutator pointer is valid.
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn alloc(
    mutator: *mut Mutator<JikesRVM>,
    size: usize,
    align: usize,
    offset: usize,
    allocator: AllocationSemantics,
) -> Address {
    memory_manager::alloc::<JikesRVM>(unsafe { &mut *mutator }, size, align, offset, allocator)
}

#[no_mangle]
// We trust the mutator pointer is valid.
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn post_alloc(
    mutator: *mut Mutator<JikesRVM>,
    refer: ObjectReference,
    _type_refer: ObjectReference,
    bytes: usize,
    allocator: AllocationSemantics,
) {
    memory_manager::post_alloc::<JikesRVM>(unsafe { &mut *mutator }, refer, bytes, allocator)
}

#[no_mangle]
// For a syscall that returns bool, we have to return a i32 instead. See https://github.com/mmtk/mmtk-jikesrvm/issues/20
pub extern "C" fn will_never_move(object: ObjectReference) -> i32 {
    !object.is_movable() as i32
}

#[no_mangle]
// We trust the gc_collector pointer is valid.
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn start_control_collector(
    tls: VMWorkerThread,
    gc_controller: *mut GCController<JikesRVM>,
) {
    let mut gc_controller = unsafe { Box::from_raw(gc_controller) };
    let cstr = std::ffi::CString::new("MMTkController").unwrap();
    unsafe {
        libc::pthread_setname_np(libc::pthread_self(), cstr.as_ptr());
    }

    memory_manager::start_control_collector(&SINGLETON, tls, &mut gc_controller);
}

#[no_mangle]
// We trust the worker pointer is valid.
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn start_worker(tls: VMWorkerThread, worker: *mut GCWorker<JikesRVM>) {
    let mut worker = unsafe { Box::from_raw(worker) };
    let cstr = std::ffi::CString::new(format!("MMTkWorker{}", worker.ordinal)).unwrap();
    unsafe {
        libc::pthread_setname_np(libc::pthread_self(), cstr.as_ptr());
    }
    memory_manager::start_worker::<JikesRVM>(&SINGLETON, tls, &mut worker)
}

#[no_mangle]
pub extern "C" fn enable_collection(tls: VMThread) {
    // MMTk core renamed enable_collection() to initialize_collection(). The JikesRVM binding
    // never uses the new enable_collection() API so we just expose this as enable_collection().
    // Also this is used by JikesRVM for third party heaps in places where it uses JavaMMTK's enableCollection().
    memory_manager::initialize_collection(&SINGLETON, tls)
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
pub extern "C" fn handle_user_collection_request(tls: VMMutatorThread) {
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
    memory_manager::is_in_mmtk_spaces::<JikesRVM>(object) as i32
}

#[no_mangle]
// For a syscall that returns bool, we have to return a i32 instead. See https://github.com/mmtk/mmtk-jikesrvm/issues/20
pub extern "C" fn is_mapped_address(address: Address) -> i32 {
    memory_manager::is_mapped_address(address) as i32
}

#[no_mangle]
pub extern "C" fn modify_check(object: ObjectReference) {
    memory_manager::modify_check(&SINGLETON, object)
}

#[no_mangle]
pub extern "C" fn get_forwarded_object(object: ObjectReference) -> ObjectReference{
    match object.get_forwarded_object() {
        Some(ref_obj) => ref_obj,
        None => ObjectReference::NULL,
    }
}

#[no_mangle]
pub extern "C" fn is_reachable(object: ObjectReference) -> i32 {
    object.is_reachable() as i32
}


#[no_mangle]
// We trust the name/value pointer is valid.
#[allow(clippy::not_unsafe_ptr_arg_deref)]
// For a syscall that returns bool, we have to return a i32 instead. See https://github.com/mmtk/mmtk-jikesrvm/issues/20
pub extern "C" fn get_boolean_option(option: *const c_char) -> i32 {
    let option_str: &CStr = unsafe { CStr::from_ptr(option) };
    match option_str.to_str() {
        Ok(s) => {
            if s == "noReferenceTypes" {
                *SINGLETON.get_options().no_reference_types as i32
            } else {
                unimplemented!()
            }
        }
        Err(e) => {
            panic!("Invalid boolean option {:?}: {:?}", option_str, e);
        }
    }
}

#[no_mangle]
pub extern "C" fn harness_begin(tls: VMMutatorThread) {
    memory_manager::harness_begin(&SINGLETON, tls)
}

#[no_mangle]
pub extern "C" fn harness_end(_tls: OpaquePointer) {
    memory_manager::harness_end(&SINGLETON)
}

#[no_mangle]
// We trust the name/value pointer is valid.
#[allow(clippy::not_unsafe_ptr_arg_deref)]
// For a syscall that returns bool, we have to return a i32 instead. See https://github.com/mmtk/mmtk-jikesrvm/issues/20
pub extern "C" fn process(name: *const c_char, value: *const c_char) -> i32 {
    let name_str: &CStr = unsafe { CStr::from_ptr(name) };
    let value_str: &CStr = unsafe { CStr::from_ptr(value) };
    let mut builder = BUILDER.lock().unwrap();
    memory_manager::process(
        &mut builder,
        name_str.to_str().unwrap(),
        value_str.to_str().unwrap(),
    ) as i32
}

#[no_mangle]
pub extern "C" fn starting_heap_address() -> Address {
    memory_manager::starting_heap_address()
}

#[no_mangle]
pub extern "C" fn last_heap_address() -> Address {
    memory_manager::last_heap_address()
}

#[no_mangle]
pub extern "C" fn get_finalized_object() -> ObjectReference {
    match memory_manager::get_finalized_object(&SINGLETON) {
        Some(obj) => obj,
        None => ObjectReference::NULL,
    }
}

// Allocation slow path

use mmtk::util::alloc::Allocator as IAllocator;
use mmtk::util::alloc::{BumpAllocator, LargeObjectAllocator};

#[no_mangle]
pub extern "C" fn alloc_slow_bump_monotone_immortal(
    allocator: *mut c_void,
    size: usize,
    align: usize,
    offset: usize,
) -> Address {
    unsafe { &mut *(allocator as *mut BumpAllocator<JikesRVM>) }.alloc_slow(size, align, offset)
}

// For plans that do not include copy space, use the other implementation
// FIXME: after we remove plan as build-time option, we should remove this conditional compilation as well.

#[no_mangle]
#[cfg(feature = "semispace")]
pub extern "C" fn alloc_slow_bump_monotone_copy(
    allocator: *mut c_void,
    size: usize,
    align: usize,
    offset: usize,
) -> Address {
    unsafe { &mut *(allocator as *mut BumpAllocator<JikesRVM>) }.alloc_slow(size, align, offset)
}
#[no_mangle]
#[cfg(not(feature = "semispace"))]
pub extern "C" fn alloc_slow_bump_monotone_copy(
    _allocator: *mut c_void,
    _size: usize,
    _align: usize,
    _offset: usize,
) -> Address {
    unimplemented!()
}

#[no_mangle]
pub extern "C" fn alloc_slow_largeobject(
    allocator: *mut c_void,
    size: usize,
    align: usize,
    offset: usize,
) -> Address {
    unsafe { &mut *(allocator as *mut LargeObjectAllocator<JikesRVM>) }
        .alloc_slow(size, align, offset)
}

// Test
// TODO: we should remove this?

#[no_mangle]
pub extern "C" fn test_stack_alignment() {
    use std::arch::asm;
    info!("Entering stack alignment test with no args passed");
    unsafe {
        let _xmm: f32;
        asm!("movaps {}, [esp]", lateout(xmm_reg) _xmm);
    }
    info!("Exiting stack alignment test");
}

#[allow(clippy::many_single_char_names)]
#[no_mangle]
pub extern "C" fn test_stack_alignment1(a: usize, b: usize, c: usize, d: usize, e: usize) -> usize {
    use std::arch::asm;
    info!("Entering stack alignment test");
    info!("a:{}, b:{}, c:{}, d:{}, e:{}", a, b, c, d, e);
    unsafe {
        let _xmm: f32;
        asm!("movaps {}, [esp]", lateout(xmm_reg) _xmm);
    }
    let result = a + b * 2 + c * 3 + d * 4 + e * 5;
    info!("Exiting stack alignment test");
    result
}
