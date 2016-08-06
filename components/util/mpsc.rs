pub use schedule_recv::oneshot_ms;

#[macro_export]
macro_rules! lselect_timeout {
    ($time_ms:expr => $code:expr, $($args:tt)*) => ({
        let timeout_rx = $crate::lselect::oneshot_ms($time_ms);
        lselect!(
            _ = timeout_rx => $code,
            $($args)*
        )
    });
}

#[macro_export]
macro_rules! lselect {
    (@internal
        [$($parsed:tt)*]
        $name:pat = $handle:expr => $code:expr, 
        $($rest:tt)*
    ) => ({
        lselect!(@internal [$($parsed)* rx rx2 $name = $handle => $code,]
                         $($rest)*)
    });
    
    (@internal
        [$($parsed:tt)*]
        $name:pat = $handle:expr => $code:expr
    ) => ({
        lselect!([$($parsed)* rx rx2 $name = $handle => $code,])
    });

    (@internal
        [$($rx:ident $output:ident $name:pat = $handle:expr => $code:expr,)*]
    ) => ({
        $( let mut $output = None; )+
        {
            use std::sync::mpsc::Select;
            let sel = Select::new();
            $( let mut $rx = sel.handle(&$handle); )+
            unsafe {
                $( $rx.add(); )+
            }
            let ret = sel.wait();
            $( if ret == $rx.id() { $output = Some($rx.recv()); } )+ 
        }
        $( if let Some($name) = $output { $code } else )+
        { unreachable!() }
    });
    
    ($($args:tt)*) => ( lselect!(@internal [] $($args)* ) );
}

#[derive(Debug)]
pub enum ChanError {
    SendError(&'static str),
    RecvError(&'static str),
}

#[macro_export]
macro_rules! send {
    ($sender:expr, $path:path => ( $($arg:expr),* )) => {{
        let data = $path($($arg),*);
        $sender.send(data)
            .map_err(|e| {
                $crate::ChanError::SendError(stringify!($path))
            })
    }}
}

#[macro_export]
macro_rules! req_rep {
    ($sender:expr, $path:path => ( $($arg:expr),* )) => {{
        let (tx, rx) = channel();
        let data = $path(tx, $($arg),*);
        let path_str = stringify!($path);
        let ret = $sender.send(data)
            .map_err(|_e| {
                $crate::ChanError::SendError(stringify!($path))
            });
        let finalres: ::std::result::Result<_, $crate::ChanError>;
        if ret.is_err() {
            finalres = Err(ret.unwrap_err());
        } else {
            finalres = rx.recv()
            .map_err(|_e| {
                $crate::ChanError::RecvError(stringify!($path))
            });
        }
        finalres
    }}
}
