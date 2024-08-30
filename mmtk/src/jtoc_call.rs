use mmtk::util::{Address, OpaquePointer, VMMutatorThread, VMThread, VMWorkerThread};

#[cfg(target_arch = "x86")]
#[macro_export]
macro_rules! jtoc_call {
    ($offset:ident, $tls:expr $(, $arg:ident)*) => ({
        use JTOC_BASE;
        let call_addr = (JTOC_BASE + $offset).load::<fn()>();
        jikesrvm_call!(call_addr, $tls $(, $arg)*)
    });
}

#[cfg(target_arch = "x86")]
#[macro_export]
macro_rules! jikesrvm_instance_call {
    ($obj:expr, $offset:expr, $tls:expr $(, $arg:ident)*) => ({
        use java_header::TIB_OFFSET;
        let tib = ($obj + TIB_OFFSET).load::<Address>();
        let call_addr = (tib + $offset).load::<fn()>();
        jikesrvm_call!(call_addr, $tls $(, $arg)*)
    });
}

#[cfg(target_arch = "x86")]
#[macro_export]
macro_rules! jikesrvm_call {
    // Note: `$arg` must be identifiers (i.e. they must have already been evaluated before calling this macro).
    // Otherwise the instructions for evaluating arguments will be intermingled with the `push {}` instructions.
    ($call_addr:expr, $tls:expr $(, $arg:ident)*) => ({
        // ret is mut, as asm! will write to it.
        let mut ret: usize;

        // Evaluate the call address.
        let call_addr = $call_addr;

        // Cast $tls from opaque pointer to a primitive type so we can use it in asm!
        let rvm_thread = $crate::jtoc_call::ToRvmThreadArg::to_rvm_thread_arg($tls);
        debug_assert_ne!(rvm_thread, 0);

        $(
            std::arch::asm!(
                "push {}",
                in(reg) $arg,
            );
        )*

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
    // * esi: new asm! does not allow use esi as operand (https://github.com/rust-lang/rust/blob/master/compiler/rustc_target/src/asm/x86.rs#L182), we have to manually save/restore it (following the pattern here: https://github.com/rust-lang/rust/pull/84658/files#diff-d7283132d97a993fad4e2d491ac883dbce4e17fe248cdf37fa3f9334e2a5a115)
    // * memory: llvm_asm! allows to specify "memory" as clobber if the asm modifies memory.
    //   asm! has a option nomem to specify the asm does not modify memory, otherwise the compiler assumes the asm will modify memory.
    // * xmm/ymm: Those are lower parts of zmm. We only need to mark zmm as clobbers.

    ($ret:ident, $rvm_thread:ident, $call_addr:ident) => (
        std::arch::asm!(
            // Exchange the value of TLS and esi (so esi now holds the TLS, and the TLS temp holds the esi value)
            "xchg {0}, esi",
            // Call $call_addr which is in ebx
            "call ebx",
            // Restore esi from the TLS
            "mov esi, {0}",

            // TLS. We will manually exchange it with esi.
            inout(reg) $rvm_thread => _,
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
        );
    );

    ($ret:ident, $rvm_thread:ident, $call_addr:ident, $arg1:expr) => (
        std::arch::asm!(
            // Exchange the value of TLS and esi (so esi now holds the TLS, and the TLS temp holds the esi value)
            "xchg {0}, esi",
            // Call $call_addr which is in ebx
            "call ebx",
            // Restore esi from the TLS
            "mov esi, {0}",

            // TLS. We will manually exchange it with esi.
            inout(reg) $rvm_thread => _,
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
        );
    );

    ($ret:ident, $rvm_thread:ident, $call_addr:ident, $arg1:expr, $arg2:expr $(, $arg:expr)*) => (
        std::arch::asm!(
            // Exchange the value of TLS and esi (so esi now holds the TLS, and the TLS temp holds the esi value)
            "xchg {0}, esi",
            // Call $call_addr which is in ebx
            "call ebx",
            // Restore esi from the TLS
            "mov esi, {0}",

            // TLS. We will manually exchange it with esi.
            inout(reg) $rvm_thread => _,
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
        );
    );
}

/// Types that can be used as the `rvm_thread` argument.
pub trait ToRvmThreadArg {
    fn to_rvm_thread_arg(self) -> usize;
}

impl ToRvmThreadArg for VMMutatorThread {
    fn to_rvm_thread_arg(self) -> usize {
        unsafe { std::mem::transmute::<_, usize>(self) }
    }
}

impl ToRvmThreadArg for VMWorkerThread {
    fn to_rvm_thread_arg(self) -> usize {
        unsafe { std::mem::transmute::<_, usize>(self) }
    }
}

impl ToRvmThreadArg for VMThread {
    fn to_rvm_thread_arg(self) -> usize {
        unsafe { std::mem::transmute::<_, usize>(self) }
    }
}

impl ToRvmThreadArg for OpaquePointer {
    fn to_rvm_thread_arg(self) -> usize {
        unsafe { std::mem::transmute::<_, usize>(self) }
    }
}

/// Types that can be used as arguments of `jtoc_call!` after conversion.
pub trait ToAsmArg {
    type ResultType;
    const ALLOW_RVM_THREAD_ARG: bool = false;
    fn to_jtoc_call_arg(self) -> Self::ResultType;
}

/// Types that can be converted from the result of `jtoc_call!`, that is, `usize`.
pub trait FromAsmResult {
    fn from_asm_result(value: usize) -> Self;
}

/// All the opaque pointer types above can be used as asm args.
impl<T: ToRvmThreadArg> ToAsmArg for T {
    type ResultType = usize;

    fn to_jtoc_call_arg(self) -> Self::ResultType {
        self.to_rvm_thread_arg()
    }
}

/// JikesRVM uses a 32-bit register to hold boolean values.
impl ToAsmArg for bool {
    type ResultType = i32;

    fn to_jtoc_call_arg(self) -> Self::ResultType {
        self as i32
    }
}

/// Interpret the whole 32-bit register as the result of a boolean return value.
impl FromAsmResult for bool {
    fn from_asm_result(value: usize) -> Self {
        value != 0
    }
}

/// Address type is just `usize`.
impl ToAsmArg for Address {
    type ResultType = usize;

    fn to_jtoc_call_arg(self) -> Self::ResultType {
        self.as_usize()
    }
}
