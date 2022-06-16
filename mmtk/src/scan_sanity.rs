use entrypoint::*;
use mmtk::util::OpaquePointer;
use std::arch::asm;

pub fn scan_boot_image_sanity(tls: OpaquePointer) {
    trace!("scan_boot_image_sanity");
    let boot_image_roots: [usize; 10000] = [0; 10000];
    let addr = &boot_image_roots as *const usize;

    unsafe {
        jtoc_call!(SCAN_BOOT_IMAGE_METHOD_OFFSET, tls, addr);
    }

    for slot in boot_image_roots.iter() {
        if *slot == 0 {
            break;
        }
        print!("0x{:X} ", slot);
    }
    println!();
}
