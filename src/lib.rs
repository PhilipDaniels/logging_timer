//! Provides a couple of simple timers that output messages indicating the elapsed time via the
//! [log](https://crates.io/crates/log) crate when they are dropped.
//!
//! Timers have names, and the log messages are constructed in such a way that they contain
//! the module, filename and line number of the place where the timer was constructed.
//!
//! Timers are usually created using the `timer!` or `stimer!` macros. The difference is
//! that `timer!` returns a timer that logs a message only when it is dropped, while `stimer!`
//! returns a timer that logs a started message as soon as it is created, and a finished
//! message when it is dropped.
//!
//! Example - "Find Files" is the name of the timer:
//!
//! ```norun
//! use logging_timer::{timer};
//!
//! fn find_files(dir: PathBuf) -> List<PathBuf> {
//!     let _tmr = timer!("Find Files");
//!     let files = vec![];
//!
//!     // expensive operation here
//!
//!     return files;
//! } // _tmr is dropped here and a 'TimerFinished' message is logged
//!```
//!
//! You can replace `timer!` with `stimer!` to get a timer that logs a starting message as
//! well, giving you a pair of 'bracketing' log messages.
//!
//! In addition, both timer macros accept [format_args!](https://doc.rust-lang.org/std/macro.format_args.html)
//! style parameters, allowing you to include extra information in the log messages.
//!
//! ```norun
//! let tmr = timer!("Find Files", "Directory = {}", dir);
//! ```
//!
//! # Outputting Intermediate Messages
//!
//! The `executing!` macro allows you to make the timer produce a message before it is dropped.
//! You can call it as many times as you want. A pseudocode example:
//!
//! ```norun
//! use logging_timer::{timer, executing};
//!
//! fn find_files(dir: PathBuf) -> List<PathBuf> {
//!     let tmr = timer!("Find Files");
//!     let files = vec![];
//!
//!     for dir in sub_dirs(dir) {
//!         // expensive operation
//!         executing!(tmr, "Processed {}", dir);
//!     }
//!
//!     return files;
//! } // tmr is dropped here and a 'TimerFinished' message is logged
//!```
//!
//! # Controlling the Final Message
//!
//! Finally, the `finish!` macro also makes the timer log a message, but it also has the side
//! effect of suppressing the normal drop message.  `finish!` is useful when you want the final
//! message to include some information that you did not have access to until the calculation had
//! finished.
//!
//! ```norun
//! use logging_timer::{timer, executing, finish};
//!
//! fn find_files(dir: PathBuf) -> List<PathBuf> {
//!     let tmr = timer!("Find Files");
//!     let files = vec![];
//!
//!     finish!(tmr, "Found {} files", files.len());
//!     return files;
//! } // tmr is dropped here but no message is produced.
//!```
//!
//! # Example of Timer Output
//!
//! The overall format will depend on how you customize the output format of the log crate, but as an illustrative example:
//!
//! ```text
//! 2019-05-30T21:41:41.847982550Z DEBUG [TimerStarting] [dnscan/src/main.rs/63] Directory Analysis
//! 2019-05-30T21:41:41.868690703Z INFO [dnlib::configuration] [dnlib/src/configuration.rs/116] Loaded configuration from "/home/phil/.dnscan/.dnscan.json"
//! 2019-05-30T21:41:41.897609281Z DEBUG [TimerFinished] [dnlib/src/io.rs/67] Elapsed=28.835275ms, Find Files, Dir="/home/phil/mydotnetprojects", NumSolutions=1 NumCsproj=45, NumOtherFiles=12
//! 2019-05-30T21:41:41.955140835Z DEBUG [TimerFinished] [dnlib/src/analysis.rs/93] Elapsed=57.451736ms, Load And Analyze Solution files
//! 2019-05-30T21:41:42.136762196Z DEBUG [TimerFinished] [dnlib/src/analysis.rs/108] Elapsed=181.563223ms, Load And Analyze Project files, Found 43 linked projects and 2 orphaned projects
//! 2019-05-30T21:41:42.136998556Z DEBUG [TimerStarting] [dnscan/src/main.rs/87] Calculate project graphs and redundant projects
//! 2019-05-30T21:41:42.143072972Z DEBUG [TimerExecuting] [dnscan/src/main.rs/87] Elapsed=6.075205ms, Calculate project graphs and redundant projects, Individual graphs done
//! 2019-05-30T21:41:42.149218039Z DEBUG [TimerFinished] [dnscan/src/main.rs/87] Elapsed=12.219438ms, Calculate project graphs and redundant projects, Found 19 redundant project relationships
//! 2019-05-30T21:41:42.165724712Z DEBUG [TimerFinished] [dnscan/src/main.rs/108] Elapsed=16.459312ms, Write output files
//! 2019-05-30T21:41:42.166445Z INFO [TimerFinished] [dnscan/src/main.rs/63] Elapsed=318.48581ms, Directory Analysis
//! ```
//!
//! Here the `[Timer*]` blocks are the `target` field from log's [Record](https://docs.rs/log/0.4.6/log/struct.Record.html)
//! struct and `[dnscan/src/main.rs/63]` is the filename and number from `Record` - this captures the place where the timer was
//! instantiated. The module is also set, but is not shown in these examples.
//! Note how the timer named 'Directory Analysis' is the only one to log both a starting and ending message.
//!
//! By default messages are output at DEBUG level, you can control this using the `level()` builder method:
//!
//! ```norun
//! let _tmr = timer!("Directory Analysis").level(Level::Info);
//! ```
//!
//! (There is a bug, this does not work for `stimer!` yet, only `timer!`)


use log::{log_enabled, Level, RecordBuilder};
use std::fmt;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;

/// When this struct is dropped, it logs a message stating its name and how long
/// the execution time was. Can be used to time functions or other critical areas.
pub struct LoggingTimer<'name> {
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
    name: &'name str,
    /// Any extra information to be logged along with the name. Unfortunately, due
    /// to the lifetimes associated with a `format_args!` invocation, this currently allocates
    /// if you use it.
    extra_info: Option<String>,
}

impl<'name> LoggingTimer<'name> {
    /// Constructs a new `LoggingTimer` that prints only a 'TimerFinished' message.
    /// This method is not usually called directly, use the `timer!` macro instead.
    pub fn new(
        file: &'static str,
        module_path: &'static str,
        line: u32,
        name: &'name str,
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

    /// Constructs a new `LoggingTimer` that prints a 'TimerStarting' and a 'TimerFinished' message.
    /// This method is not usually called directly, use the `stimer!` macro instead.
    pub fn with_start_message(
        file: &'static str,
        module_path: &'static str,
        line: u32,
        name: &'name str,
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
    /// Note that this consumes self, so that it can be called in a one-liner like this:
    ///
    /// ```norun
    /// let tmr = timer!("foo").level(Level::Trace);
    /// ```
    pub fn level(mut self, level: Level) -> Self {
        self.level = level;
        self
    }

    /// Outputs a log message with a target of 'TimerExecuting' showing the current elapsed time, but does not
    /// stop the timer. This method can be called multiple times.
    /// The message can include further information via a `format_args!` approach.
    /// This method is usually not called directly, it is easier to use the `executing!` macro.
    pub fn executing(&self, args: Option<fmt::Arguments>) {
        self.log_impl(TimerTarget::Executing, args);
    }

    /// Outputs a log message with a target of 'TimerFinished' and suppresses the normal message
    /// that is output when the timer is dropped. The message can include further `format_args!`
    /// information. This method is normally called using the `finish!` macro. Calling
    /// `finish()` again will have no effect.
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
    /// Drops the timer, outputting a log message with a target of `TimerFinished`
    /// if the `finish` method has not yet been called.
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

/// Makes an existing timer output an 'executing' mesasge.
/// Can be called multiple times.
#[macro_export]
macro_rules! executing {
    ($timer:expr) => ({
        $timer.executing(None)
    });

    ($timer:expr, $format:tt) => ({
        $timer.executing(Some(format_args!($format)))
    });

    ($timer:expr, $format:tt, $($arg:expr),*) => ({
        $timer.executing(Some(format_args!($format, $($arg), *)))
    })
}

/// Makes an existing timer output a 'finished' mesasge and suppresses
/// the normal drop message.
/// Only the first call has any effect, subsequent calls will be ignored.
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

#[cfg(test)]
mod tests {
    use super::*;
    use log::Level;

    // Just some simple tests to check that the macros still generate
    // valid code when refactoring.

    #[test]
    pub fn construction() {
        let _ = timer!("foo");
        let _ = timer!("foo", "pat");
        let _ = timer!("foo", "pat {}", 1);

        let _ = stimer!("foo");
        let _ = stimer!("foo", "pat");
        let _ = stimer!("foo", "pat {}", 1);
    }

    #[test]
    pub fn executing() {
        let tmr = timer!("foo");
        executing!(tmr);
        executing!(tmr, "pat");
        executing!(tmr, "pat {}", 1);
    }

    #[test]
    pub fn finish() {
        let tmr = timer!("foo");
        finish!(tmr);
        finish!(tmr, "pat");
        finish!(tmr, "pat {}", 1);
    }

    #[test]
    pub fn level() {
        let tmr = timer!("foo").level(Level::Trace);
        assert_eq!(tmr.level, Level::Trace);
    }
}
