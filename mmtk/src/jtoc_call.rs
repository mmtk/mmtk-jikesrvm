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
    ($call_addr:expr, $tls:expr $(, $arg:ident)*) => ({
        use mmtk::util::Address;
        debug_assert!(! std::mem::transmute::<_, Address>($tls).is_zero());

        let ret: usize;
        let rvm_thread = $tls;

        $(
            llvm_asm!("push %ebx" : : "{ebx}"($arg) : "sp", "memory");
        )*

        let call_addr = $call_addr;
        jikesrvm_call_helper!(ret, rvm_thread, call_addr $(, $arg)*);

        ret
    });
}

#[cfg(target_arch = "x86")]
macro_rules! jikesrvm_call_helper {
    ($ret:ident, $rvm_thread:ident, $call_addr:ident) => (
        llvm_asm!("call *%ebx"
             : "={eax}"($ret)
             : "{esi}"($rvm_thread), "{ebx}"($call_addr)
             : "ebx", "ecx", "edx", "esi", "memory", "mm0", "mm1", "mm2", "mm3", "mm4", "mm5", "mm6", "mm7", "xmm0", "xmm1", "xmm2", "xmm3", "xmm4", "xmm5", "xmm6", "xmm7", "ymm0", "ymm1", "ymm2", "ymm3", "ymm4", "ymm5", "ymm6", "ymm7", "zmm0", "zmm1", "zmm2", "zmm3", "zmm4", "zmm5", "zmm6", "zmm7"
             : "volatile");
    );

    ($ret:ident, $rvm_thread:ident, $call_addr:ident, $arg1:ident) => (
        llvm_asm!("call *%ebx"
             : "={eax}"($ret)
             : "{esi}"($rvm_thread), "{ebx}"($call_addr), "{eax}"($arg1)
             : "ebx", "ecx", "edx", "esi", "memory", "mm0", "mm1", "mm2", "mm3", "mm4", "mm5", "mm6", "mm7", "xmm0", "xmm1", "xmm2", "xmm3", "xmm4", "xmm5", "xmm6", "xmm7", "ymm0", "ymm1", "ymm2", "ymm3", "ymm4", "ymm5", "ymm6", "ymm7", "zmm0", "zmm1", "zmm2", "zmm3", "zmm4", "zmm5", "zmm6", "zmm7"
             : "volatile");
    );

    ($ret:ident, $rvm_thread:ident, $call_addr:ident, $arg1:ident, $arg2:ident $(, $arg:ident)*) => (
        llvm_asm!("call *%ebx"
             : "={eax}"($ret)
             : "{esi}"($rvm_thread), "{ebx}"($call_addr), "{eax}"($arg1), "{edx}"($arg2)
             : "ebx", "ecx", "edx", "esi", "memory", "mm0", "mm1", "mm2", "mm3", "mm4", "mm5", "mm6", "mm7", "xmm0", "xmm1", "xmm2", "xmm3", "xmm4", "xmm5", "xmm6", "xmm7", "ymm0", "ymm1", "ymm2", "ymm3", "ymm4", "ymm5", "ymm6", "ymm7", "zmm0", "zmm1", "zmm2", "zmm3", "zmm4", "zmm5", "zmm6", "zmm7"
             : "volatile");
    );
}
