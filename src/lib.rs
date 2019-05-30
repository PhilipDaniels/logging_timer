use log::{log_enabled, Level, RecordBuilder};
use std::fmt;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;

/// When this struct is dropped, it logs a message stating its name and how long
/// the execution time was. Can be used to time functions or other critical areas.
pub struct LoggingTimer<'a> {
    /// The log level. Defaults to Debug.
    level: Level,
    /// Set by the file!() macro to the name of the file where the timer is instantiated.
    file: &'static str,
    /// Set by the module_path!() macro to the module where the timer is instantiated.
    module_path: &'static str,
    /// Set by the line!() macro to the line number where the timer is instantiated.
    line: u32,
    /// A flag used to suppress printing of the 'Finished' message in the drop() function
    /// It is set by the finish method.
    finished: AtomicBool,
    /// The instant, in UTC, that the timer was instantiated.
    start_time: Instant,
    /// The name of the timer. Used in messages to identify it.
    name: &'a str,
    /// Any extra information to be logged along with the name. Unfortunately, due
    /// to the lifetimes associated with a `format_args!` invocation, this currently allocates
    /// if you use it.
    extra_info: Option<String>,
}

impl<'a> LoggingTimer<'a> {
    /// Constructs a new `LoggingTimer` that prints only a 'Finished' message.
    /// This method is not usually called directly, use the `timer!` macro instead.
    pub fn new(
        file: &'static str,
        module_path: &'static str,
        line: u32,
        name: &'a str,
        extra_info: Option<String>,
    ) -> Self
    {
        LoggingTimer {
            level: Level::Debug,
            start_time: Instant::now(),
            file: file,
            module_path: module_path,
            line: line,
            name: name,
            finished: AtomicBool::new(false),
            extra_info: extra_info,
        }
    }

    /// Constructs a new `LoggingTimer` that prints a 'Starting' and a 'Finished' message.
    /// This method is not usually called directly, use the `stimer!` macro instead.
    pub fn with_start_message(
        file: &'static str,
        module_path: &'static str,
        line: u32,
        name: &'a str,
        extra_info: Option<String>,
    ) -> Self
    {
        // Determine this before calling log_impl, since logging will take time
        // itself, i.e. it is overhead that can confuse timings.
        let start_time = Instant::now();

        let tmr = LoggingTimer {
            level: Level::Debug,
            start_time: start_time,
            file: file,
            module_path: module_path,
            line: line,
            name: name,
            finished: AtomicBool::new(false),
            extra_info: extra_info,
        };

        tmr.log_impl(TimerTarget::Starting, None);

        tmr
    }

    /// Returns how long the timer has been running for.
    pub fn elapsed(&self) -> std::time::Duration {
        self.start_time.elapsed()
    }

    /// Sets the logging level.
    pub fn level(mut self, level: Level) -> Self {
        self.level = level;
        self
    }

    /// Outputs a log message showing the current elapsed time, but does not stop the timer.
    /// This method can be called multiple times until the timer is dropped.
    /// The message can include further information via a `format_args!` approach.
    /// This method is usually not called directly, it is easier to call via the `progress!`
    /// macro.
    pub fn progress(&self, args: Option<fmt::Arguments>) {
        self.log_impl(TimerTarget::Executing, args);
    }

    /// Outputs a 'Finished' log message and suppresses the normal message that is
    /// output when the timer is dropped. The message can include further `format_args!`
    /// information. This method is normally called using the `finish!` macro. Calling
    /// finish() again will have no effect.
    pub fn finish(&self, args: Option<fmt::Arguments>) {
        if !self.finished.load(Ordering::SeqCst) {
            self.finished.store(true, Ordering::SeqCst);
            self.log_impl(TimerTarget::Finished, args);
        }
    }

    fn log_impl(&self, target: TimerTarget, args: Option<fmt::Arguments>) {
        if !log_enabled!(self.level) { return; }

        match (target, self.extra_info.as_ref(), args) {
            (TimerTarget::Starting, Some(info), Some(args)) => self.log_record(target, format_args!("{}, {}, {}", self.name, info, args)),
            (TimerTarget::Starting, Some(info), None)       => self.log_record(target, format_args!("{}, {}",  self.name, info)),
            (TimerTarget::Starting, None, Some(args))       => self.log_record(target, format_args!("{}, {}", self.name, args)),
            (TimerTarget::Starting, None, None)             => self.log_record(target, format_args!("{}", self.name)),

            (_, Some(info), Some(args)) => self.log_record(target, format_args!("Elapsed={:?}, {}, {}, {}", self.elapsed(), self.name, info, args)),
            (_, Some(info), None)       => self.log_record(target, format_args!("Elapsed={:?}, {}, {}", self.elapsed(), self.name, info)),
            (_, None, Some(args))       => self.log_record(target, format_args!("Elapsed={:?}, {}, {}", self.elapsed(), self.name, args)),
            (_, None, None)             => self.log_record(target, format_args!("Elapsed={:?}, {}", self.elapsed(), self.name)),
        };
    }

    fn log_record(&self, target: TimerTarget, args: fmt::Arguments) {
        log::logger().log(
            &RecordBuilder::new()
                .level(self.level)
                .target(match target {
                    TimerTarget::Starting => "TimerStarting",
                    TimerTarget::Executing => "TimerExecuting",
                    TimerTarget::Finished => "TimerFinished",
                })
                .file(Some(self.file))
                .module_path(Some(self.module_path))
                .line(Some(self.line))
                .args(args)
                .build(),
        );
    }
}

impl<'a> Drop for LoggingTimer<'a> {
    /// Drops the timer, outputting a 'Finished' message if `finish` has not yet been called.
    fn drop(&mut self) {
        self.finish(None);
    }
}

#[derive(Debug, Copy, Clone)]
enum TimerTarget {
    Starting,
    Executing,
    Finished,
}

/// Creates a timer that does not log a starting message, only a finished one.
#[macro_export]
macro_rules! timer {
    ($name:expr) => {
        {
            $crate::LoggingTimer::new(
                file!(),
                module_path!(),
                line!(),
                $name,
                None,
                )
        }
    };

    ($name:expr, $format:tt) => {
        {
            $crate::LoggingTimer::new(
                file!(),
                module_path!(),
                line!(),
                $name,
                Some(format!($format)),
                )
        }
    };

    ($name:expr, $format:tt, $($arg:expr),*) => {
        {
            $crate::LoggingTimer::new(
                file!(),
                module_path!(),
                line!(),
                $name,
                Some(format!($format, $($arg), *)),
                )
        }
    };
}

/// Creates a timer that logs a starting mesage and a finished message.
#[macro_export]
macro_rules! stimer {
    ($name:expr) => {
        {
            $crate::LoggingTimer::with_start_message(
                file!(),
                module_path!(),
                line!(),
                $name,
                None,
                )
        }
    };

    ($name:expr, $format:tt) => {
        {
            $crate::LoggingTimer::with_start_message(
                file!(),
                module_path!(),
                line!(),
                $name,
                Some(format!($format)),
                )
        }
    };

    ($name:expr, $format:tt, $($arg:expr),*) => {
        {
            $crate::LoggingTimer::with_start_message(
                file!(),
                module_path!(),
                line!(),
                $name,
                Some(format!($format, $($arg), *)),
                )
        }
    };
}

#[macro_export]
macro_rules! finish {
    ($timer:expr) => ({
        $timer.finish(None)
    });

    ($timer:expr, $format:tt) => ({
        $timer.finish(Some(format_args!($format)))
    });

    ($timer:expr, $format:tt, $($arg:expr),*) => ({
        $timer.finish(Some(format_args!($format, $($arg), *)))
    })
}

#[macro_export]
macro_rules! progress {
    ($timer:expr) => ({
        $timer.progress(None)
    });

    ($timer:expr, $format:tt) => ({
        $timer.progress(Some(format_args!($format)))
    });

    ($timer:expr, $format:tt, $($arg:expr),*) => ({
        $timer.progress(Some(format_args!($format, $($arg), *)))
    })
}
