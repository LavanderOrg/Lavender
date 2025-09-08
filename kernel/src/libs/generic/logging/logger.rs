use crate::libs::drivers::logs::sinks::Sink;

pub struct Logger<'a> {
    // TODO: Manage multiple sinks
    pub sink: &'a mut dyn crate::drivers::logs::sinks::Sink,
}

impl<'a> Logger<'a> {
    pub fn new(sink: &'a mut dyn Sink) -> Self {
        Logger { sink }
    }
}

impl<'a> core::fmt::Write for Logger<'a> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.sink.putstr(s);
        Ok(())
    }
}

#[macro_export]
macro_rules! _log {
    ($prefix:expr, $($arg:tt)*) => ({
        use core::fmt::Write;
        use crate::KERNEL_CONTEXT;

        unsafe {
            match &mut KERNEL_CONTEXT.logger {
                Some(logger) => {
                    write!(logger, $prefix).unwrap();
                    writeln!(logger, $($arg)*).unwrap()
                },
                None => panic!(),
            }
        }
    });
}

#[macro_export]
macro_rules! info {
    ($($args:tt)*) => {
        $crate::_log!("[Info] ", $($args)*)
    };
}

#[macro_export]
macro_rules! warning {
    ($($args:tt)*) => {
        $crate::_log!("[Warning] ", $($args)*)
    };
}

#[macro_export]
macro_rules! debug {
    ($($args:tt)*) => {
        $crate::_log!("[Debug] ", $($args)*)
    };
}

#[macro_export]
macro_rules! kpanic {
    ($($args:tt)*) => {
        $crate::_log!("[Panic !] ", $($args)*)
    };
}
