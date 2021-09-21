#[cfg(target_arch = "x86")]
#[macro_export]
macro_rules! jtoc_call {
    ($offset:ident, $tls:expr $(, $arg:expr)*) => ({
        use JTOC_BASE;
        let call_addr = (JTOC_BASE + $offset).load::<fn()>();
        jikesrvm_call!(call_addr, $tls $(, $arg)*)
    });
}

#[cfg(target_arch = "x86")]
#[macro_export]
macro_rules! jikesrvm_instance_call {
    ($obj:expr, $offset:expr, $tls:expr $(, $arg:expr)*) => ({
        use java_header::TIB_OFFSET;
        let tib = ($obj + TIB_OFFSET).load::<Address>();
        let call_addr = (tib + $offset).load::<fn()>();
        jikesrvm_call!(call_addr, $tls $(, $arg)*)
    });
}

#[cfg(target_arch = "x86")]
#[macro_export]
macro_rules! jikesrvm_call {
    ($call_addr:expr, $tls:expr $(, $arg:expr)*) => ({
        use mmtk::util::Address;
        debug_assert!(! std::mem::transmute::<_, Address>($tls).is_zero());

        // ret is mut, as asm! will write to it.
        let mut ret: usize;
        // Cast $tls from opaque pointer to a primitive type so we can use it in asm!
        let rvm_thread: usize = std::mem::transmute::<_, usize>($tls);

        $(
            // llvm_asm!("push %ebx" : : "{ebx}"($arg) : "sp", "memory");
            asm!(
                "push {}",
                in(reg) $arg,
            );
        )*

        let call_addr = $call_addr;
        jikesrvm_call_helper!(ret, rvm_thread, call_addr $(, $arg)*);

        ret
    });
}

#[cfg(target_arch = "x86")]
macro_rules! jikesrvm_call_helper {
    // The old llvm_asm! for the calls is like this:
    // llvm_asm!("call *%ebx"
    //      : "={eax}"($ret)
    //      : "{esi}"($rvm_thread), "{ebx}"($call_addr)
    //      : "ebx", "ecx", "edx", "esi", "memory", "mm0", "mm1", "mm2", "mm3", "mm4", "mm5", "mm6", "mm7", "xmm0", "xmm1", "xmm2", "xmm3", "xmm4", "xmm5", "xmm6", "xmm7", "ymm0", "ymm1", "ymm2", "ymm3", "ymm4", "ymm5", "ymm6", "ymm7", "zmm0", "zmm1", "zmm2", "zmm3", "zmm4", "zmm5", "zmm6", "zmm7"
    //      : "volatile");

    // The clobber list of the old llvm_asm! is:
    // "ebx", "ecx", "edx", "esi", "memory", "mm0", "mm1", "mm2", "mm3", "mm4", "mm5", "mm6", "mm7", "xmm0", "xmm1", "xmm2", "xmm3", "xmm4", "xmm5", "xmm6", "xmm7", "ymm0", "ymm1", "ymm2", "ymm3", "ymm4", "ymm5", "ymm6", "ymm7", "zmm0", "zmm1", "zmm2", "zmm3", "zmm4", "zmm5", "zmm6", "zmm7"
    // When we move to the new asm!, most of the registers are remained as clobbered, except:
    // * esi: new asm! does not allow use esi as operand, we have to manually save/restore it: https://github.com/rust-lang/rust/blob/master/compiler/rustc_target/src/asm/x86.rs#L182
    // * memory: llvm_asm! allows to specify "memory" as clobber if the asm modifies memory.
    //   asm! has a option nomem to specify the asm does not modify memory, otherwise the compiler assumes the asm will modify memory.
    // * xmm/ymm: Those are lower parts of zmm. We only need to mark zmm as clobbers.

    ($ret:ident, $rvm_thread:ident, $call_addr:ident) => (
        let esi_val: usize = 0;
        asm!(
            // Save the value in esi
            "mov {0}, esi",
            // Move TLS to esi
            "mov esi, {1}",
            // Call $call_addr which is in ebx
            "call ebx",
            // Restore the esi_val to esi
            "mov esi, {0}",

            // We just use esi_val as a scratch register.
            inout(reg) esi_val => _,
            // TLS. We will manually move it to esi. It would be best if we can use inout("esi") $rvm_thread => _, but asm! does not allow us to use esi as operand.
            in(reg) $rvm_thread,
            // Call address. We use ebx in asm.
            inout("ebx") $call_addr => _,
            // Return value in eax.
            out("eax") $ret,
            // Rest of the clobber list.
            out("ecx") _,
            out("edx") _,
            out("mm0") _,
            out("mm1") _,
            out("mm2") _,
            out("mm3") _,
            out("mm4") _,
            out("mm5") _,
            out("mm6") _,
            out("mm7") _,
            out("zmm0") _,
            out("zmm1") _,
            out("zmm2") _,
            out("zmm3") _,
            out("zmm4") _,
            out("zmm5") _,
            out("zmm6") _,
            out("zmm7") _,
            // equivalent of "volatile" in llvm_asm!
            options(nostack)
        );
    );

    ($ret:ident, $rvm_thread:ident, $call_addr:ident, $arg1:expr) => (
        let esi_val: usize = 0;
        asm!(
            // Save the value in esi
            "mov {0}, esi",
            // Move TLS to esi
            "mov esi, {1}",
            // Call $call_addr which is in ebx
            "call ebx",
            // Restore the esi_val to esi
            "mov esi, {0}",

            // We just use esi_val as a scratch register.
            inout(reg) esi_val => _,
            // TLS. We will manually move it to esi. It would be best if we can use inout("esi") $rvm_thread => _, but asm! does not allow us to use esi as operand.
            in(reg) $rvm_thread,
            // Call address. We use ebx in asm.
            inout("ebx") $call_addr => _,
            // arg1 in eax, also return value in eax.
            inout("eax") $arg1 => $ret,
            // Rest of the clobber list.
            out("ecx") _,
            out("edx") _,
            out("mm0") _,
            out("mm1") _,
            out("mm2") _,
            out("mm3") _,
            out("mm4") _,
            out("mm5") _,
            out("mm6") _,
            out("mm7") _,
            out("zmm0") _,
            out("zmm1") _,
            out("zmm2") _,
            out("zmm3") _,
            out("zmm4") _,
            out("zmm5") _,
            out("zmm6") _,
            out("zmm7") _,
            // equivalent of "volatile" in llvm_asm!
            options(nostack)
        );
    );

    ($ret:ident, $rvm_thread:ident, $call_addr:ident, $arg1:expr, $arg2:expr $(, $arg:expr)*) => (
        let esi_val: usize = 0;
        asm!(
            // Save the value in esi
            "mov {0}, esi",
            // Move TLS to esi
            "mov esi, {1}",
            // Call $call_addr which is in ebx
            "call ebx",
            // Restore the esi_val to esi
            "mov esi, {0}",

            // We just use esi_val as a scratch register.
            inout(reg) esi_val => _,
            // TLS. We will manually move it to esi. It would be best if we can use inout("esi") $rvm_thread => _, but asm! does not allow us to use esi as operand.
            in(reg) $rvm_thread,
            // Call address. We use ebx in asm.
            inout("ebx") $call_addr => _,
            // arg1 in eax, also return value in eax.
            inout("eax") $arg1 => $ret,
            // arg2 in edx, but edx will be clobbered.
            inout("edx") $arg2 => _,
            // Rest of the clobber list.
            out("ecx") _,
            out("mm0") _,
            out("mm1") _,
            out("mm2") _,
            out("mm3") _,
            out("mm4") _,
            out("mm5") _,
            out("mm6") _,
            out("mm7") _,
            out("zmm0") _,
            out("zmm1") _,
            out("zmm2") _,
            out("zmm3") _,
            out("zmm4") _,
            out("zmm5") _,
            out("zmm6") _,
            out("zmm7") _,
            // equivalent of "volatile" in llvm_asm!
            options(nostack)
        );
    );
}
