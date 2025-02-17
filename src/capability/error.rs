#[derive(Debug, PartialEq, Eq)]
#[repr(isize)]
pub enum OsError {
    UnknownSyscall,
    TooManyHandles,
    HandleNotFound,
    HandleNotMovable,
    UnexpectedHandleType,
    InvalidSyscallReturnValue,
    NoPeer,
    InvalidArg,
    TooLarge,
    NotSupported,
    WouldBlock,
}
