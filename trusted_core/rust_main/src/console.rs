#[macro_export]
macro_rules! print
{
    ($($args:tt)+) => ({
        use core::fmt::Write;
        let _ = write!(crate::uart::Uart::new(0x1000_0000), $($args)+);
    });
}
#[macro_export]
macro_rules! println
{
    () => ({
        $crate::print!("\r\n")
    });
    ($fmt:expr) => ({
        $crate::print!(concat!($fmt, "\r\n"))
    });
    ($fmt:expr, $($args:tt)+) => ({
        $crate::print!(concat!($fmt, "\r\n"), $($args)+)
    });
}

#[macro_export]
macro_rules! kprintln
{
    ($fmt:expr) => ({
        $crate::print!(concat!("\x1b[0;32m[kernel] \x1b[0m ", $fmt, "\r\n"))
    });
    ($fmt:expr, $($args:tt)+) => ({
        $crate::print!(concat!("\x1b[0;32m[kernel] \x1b[0m", $fmt, "\r\n"), $($args)+)
    });
}

