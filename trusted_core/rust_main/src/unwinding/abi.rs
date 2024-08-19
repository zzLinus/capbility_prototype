use super::util::*;
use core::ffi::c_void;
use core::fmt;
use core::ops;

pub use super::unwinder::*;

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct UnwindReasonCode(pub c_int);

#[allow(unused)]
impl UnwindReasonCode {
    pub const NO_REASON: Self = Self(0);
    pub const FOREIGN_EXCEPTION_CAUGHT: Self = Self(1);
    pub const FATAL_PHASE2_ERROR: Self = Self(2);
    pub const FATAL_PHASE1_ERROR: Self = Self(3);
    pub const NORMAL_STOP: Self = Self(4);
    pub const END_OF_STACK: Self = Self(5);
    pub const HANDLER_FOUND: Self = Self(6);
    pub const INSTALL_CONTEXT: Self = Self(7);
    pub const CONTINUE_UNWIND: Self = Self(8);
}

impl fmt::Debug for UnwindReasonCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let description = match self.0 {
            0 => "NO_REASON",
            1 => "FOREIGN_EXCEPTION_CAUGHT",
            2 => "FATAL_PHASE2_ERROR",
            3 => "FATAL_PHASE1_ERROR",
            4 => "NORMAL_STOP",
            5 => "END_OF_STACK",
            6 => "HANDLER_FOUND",
            7 => "INSTALL_CONTEXT",
            8 => "CONTINUE_UNWIND",
            _ => "UNKNOWN_REASON",
        };
        f.debug_struct("UnwindReasonCode")
            .field("code", &self.0)
            .field("description", &description)
            .finish()
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct UnwindAction(pub c_int);

impl UnwindAction {
    pub const SEARCH_PHASE: Self = Self(1);
    pub const CLEANUP_PHASE: Self = Self(2);
    pub const HANDLER_FRAME: Self = Self(4);
    pub const FORCE_UNWIND: Self = Self(8);
    pub const END_OF_STACK: Self = Self(16);
}

impl ops::BitOr for UnwindAction {
    type Output = Self;

    #[inline]
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

impl UnwindAction {
    #[inline]
    pub const fn empty() -> Self {
        Self(0)
    }

    #[inline]
    pub const fn contains(&self, other: Self) -> bool {
        self.0 & other.0 != 0
    }
}

pub type UnwindExceptionCleanupFn = unsafe extern "C" fn(UnwindReasonCode, *mut UnwindException);

pub type UnwindStopFn = unsafe extern "C" fn(
    c_int,
    UnwindAction,
    u64,
    *mut UnwindException,
    &mut UnwindContext<'_>,
    *mut c_void,
) -> UnwindReasonCode;

pub type UnwindTraceFn =
    extern "C" fn(ctx: &UnwindContext<'_>, arg: *mut c_void) -> UnwindReasonCode;

pub type PersonalityRoutine = unsafe extern "C" fn(
    c_int,
    UnwindAction,
    u64,
    *mut UnwindException,
    &mut UnwindContext<'_>,
) -> UnwindReasonCode;

macro_rules! binding {
    () => {};
    (unsafe extern $abi: literal fn $name: ident ($($arg: ident : $arg_ty: ty),*$(,)?) $(-> $ret: ty)?; $($rest: tt)*) => {
        const _: unsafe extern $abi fn($($arg_ty),*) $(-> $ret)? = $name;
    };

    (extern $abi: literal fn $name: ident ($($arg: ident : $arg_ty: ty),*$(,)?) $(-> $ret: ty)?; $($rest: tt)*) => {
        const _: extern $abi fn($($arg_ty),*) $(-> $ret)? = $name;
    };
}

binding! {
    extern "C" fn _Unwind_GetGR(unwind_ctx: &UnwindContext<'_>, index: c_int) -> usize;
    extern "C" fn _Unwind_GetCFA(unwind_ctx: &UnwindContext<'_>) -> usize;
    extern "C" fn _Unwind_SetGR(
        unwind_ctx: &mut UnwindContext<'_>,
        index: c_int,
        value: usize,
    );
    extern "C" fn _Unwind_GetIP(unwind_ctx: &UnwindContext<'_>) -> usize;
    extern "C" fn _Unwind_GetIPInfo(
        unwind_ctx: &UnwindContext<'_>,
        ip_before_insn: &mut c_int,
    ) -> usize;
    extern "C" fn _Unwind_SetIP(
        unwind_ctx: &mut UnwindContext<'_>,
        value: usize,
    );
    extern "C" fn _Unwind_GetLanguageSpecificData(unwind_ctx: &UnwindContext<'_>) -> *mut c_void;
    extern "C" fn _Unwind_GetRegionStart(unwind_ctx: &UnwindContext<'_>) -> usize;
    extern "C" fn _Unwind_GetTextRelBase(unwind_ctx: &UnwindContext<'_>) -> usize;
    extern "C" fn _Unwind_GetDataRelBase(unwind_ctx: &UnwindContext<'_>) -> usize;
    extern "C" fn _Unwind_FindEnclosingFunction(pc: *mut c_void) -> *mut c_void;
    unsafe extern "C-unwind" fn _Unwind_RaiseException(
        exception: *mut UnwindException,
    ) -> UnwindReasonCode;
    unsafe extern "C-unwind" fn _Unwind_ForcedUnwind(
        exception: *mut UnwindException,
        stop: UnwindStopFn,
        stop_arg: *mut c_void,
    ) -> UnwindReasonCode;
    unsafe extern "C-unwind" fn _Unwind_Resume(exception: *mut UnwindException) -> !;
    unsafe extern "C-unwind" fn _Unwind_Resume_or_Rethrow(
        exception: *mut UnwindException,
    ) -> UnwindReasonCode;
    unsafe extern "C" fn _Unwind_DeleteException(exception: *mut UnwindException);
    extern "C-unwind" fn _Unwind_Backtrace(
        trace: UnwindTraceFn,
        trace_argument: *mut c_void,
    ) -> UnwindReasonCode;
}
