use crate::libs::drivers::logs::sinks::Sink;
extern crate alloc;
use alloc::vec::Vec;
use alloc::vec;
use alloc::boxed::Box;

pub struct Logger<'a> {
    // TODO: Manage multiple sinks
    pub default_sink: &'a mut dyn Sink,
    pub sinks: Option<Vec<Box<dyn Sink + 'a>>>,
}

impl<'a> Logger<'a> {
    pub fn new(sink: &'a mut dyn Sink) -> Self {
        Logger { default_sink: sink, sinks: None }
    }

    pub fn add_sink(&mut self, sink: Box<dyn Sink + 'a>) {
        if let Some(sinks) = &mut self.sinks {
            sinks.push(sink);
        } else {
            self.sinks = Some(vec![sink]);
        }
    }

    pub fn remove_sink(&mut self, logger: &dyn Sink) {
        if let Some(sinks) = &mut self.sinks {
            sinks.retain(|s| !core::ptr::eq(&**s, logger));
        }
    }
}

impl<'a> core::fmt::Write for Logger<'a> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        match &mut self.sinks {
            Some(sinks) => {
                for sink in sinks.iter_mut() {
                    sink.putstr(s);
                }
            }
            None => {
                self.default_sink.putstr(s);
            }
        }
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
                None => panic!("Tried to log message but logger is not initialized !"),
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
