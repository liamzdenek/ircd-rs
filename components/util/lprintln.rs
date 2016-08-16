pub use std::thread;

#[macro_export]
macro_rules! lprintln {
        ($fmt:expr) => {
            let thread = $crate::lprintln::thread::current();
            let threadname = thread.name().unwrap_or("unknown thread");
            println!(concat!("[{:20}]\t", $fmt), threadname);
        };
        /*($fmt:expr, $($arg:tt)*) => {
            lprintln!($fmt, $($arg)*);
        };*/
        ($fmt:expr, $key:ident=$val:expr) => {
            let thread = $crate::lprintln::thread::current();
            let threadname = thread.name().unwrap_or("unknown thread");
            println!(concat!("[{thread_name:20}]\t", $fmt), thread_name=threadname, $key=$val);
        };
        ($fmt:expr, $key:ident=$val:expr, $($arg:tt)*) => {
            let thread = $crate::lprintln::thread::current();
            let threadname = thread.name().unwrap_or("unknown thread");
            println!(concat!("[{thread_name:20}]\t", $fmt), thread_name=threadname, $key=$val, $($arg)*);
        };
        ($fmt:expr $(, $arg:expr)*) => {
            let thread = $crate::lprintln::thread::current();
            let threadname = thread.name().unwrap_or("unknown thread");
            println!(concat!("[{:20}]\t", $fmt), threadname $(, $arg)*);
        };
}
