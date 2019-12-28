# Logging Timers for Rust

This crate provides a couple of simple timers that log messages indicating the elapsed
time between their creation and dropping. Messages are output via the
[log](https://crates.io/crates/log) crate.

Timers have names, and the log messages are constructed in such a way that they contain
the module, filename and line number of the place where the timer was constructed.


# Using the Timer Attributes

The simplest way to get started is to use one of the two attributes `time` or `stime` to
instrument a function, the name of the function is used as the name of the timer:

```rs
use logging_timer::{time, stime};

#[time]
fn find_files(dir: PathBuf) -> Vec<PathBuf> {
    let files = vec![];
    // expensive operation here
    return files;
} // TimerFinished' message is logged here
```

Both attributes accept an optional string specifying the log level, which defaults
to 'debug', e.g. `#[time("info")]`.


# Using the Inline Timers

More flexibility, including logging extra information, is provided by the two function-like
macro forms, `timer!` and `stimer!`. The difference is that `timer!` returns a timer that
logs a message only when it is dropped, while `stimer!` returns a timer that logs a started
message as soon as it is created, and a finished message when it is dropped. There are also
two corresponding proc-macros called `time` and `stimer` which can be used to instrument
functions with a timer.

In this example "FIND_FILES" is the name of the timer (using all UPPERCASE for the timer
name is optional but helps make the name stand out in the log)


```rs
use logging_timer::{timer};

fn find_files(dir: PathBuf) -> Vec<PathBuf> {
    let _tmr = timer!("FIND_FILES");
    let files = vec![];

    // expensive operation here

    return files;
} // _tmr is dropped here and a 'TimerFinished' message is logged
```

You can replace `timer!` with `stimer!` to get a timer that logs a starting message as
well, giving you a pair of 'bracketing' log messages.

In addition, both timer macros accept [format_args!](https://doc.rust-lang.org/std/macro.format_args.html)
style parameters, allowing you to include extra information in the log messages.

```rs
let _tmr = timer!("FIND_FILES", "Directory = {}", dir);
```


# Outputting Intermediate Messages

The `executing!` macro allows you to make the timer produce a message before it is dropped.
You can call it as many times as you want. A pseudocode example:

```rs
use logging_timer::{timer, executing};

fn find_files(dir: PathBuf) -> Vec<PathBuf> {
    let tmr = timer!("FIND_FILES");
    let files = vec![];

    for dir in sub_dirs(dir) {
        // expensive operation
        executing!(tmr, "Processed {}", dir);
    }

    return files;
} // tmr is dropped here and a 'TimerFinished' message is logged
```


# Controlling the Final Message

The `finish!` macro also makes the timer log a message, but it also has the side
effect of suppressing the normal drop message.  `finish!` is useful when you want the final
message to include some information that you did not have access to until the calculation had
finished.

```rs
use logging_timer::{timer, executing, finish};

fn find_files(dir: PathBuf) -> Vec<PathBuf> {
    let tmr = timer!("FIND_FILES");
    let files = vec![];

    finish!(tmr, "Found {} files", files.len());
    return files;
} // tmr is dropped here but no message is produced.
```

# Setting the log level

By default both `timer` and `stimer` log at `Debug` level. An optional first parameter to
these macros allows you to set the log level. **To aid parsing of the macro arguments this
first parameter is terminated by a semi-colon.** For example:

```rs
let tmr1 = timer!(Level::Warn; "TIMER_AT_WARN");
let tmr2 = stimer!(Level::Info; "TIMER_AT_INFO");
```
# Example of Timer Output

The overall format will depend on how you customize the output format of the log crate, but as an illustrative example:

```text
2019-05-30T21:41:41.847982550Z DEBUG [TimerStarting] [dnscan/src/main.rs/63] DIRECTORY_ANALYSIS
2019-05-30T21:41:41.868690703Z INFO [dnlib::configuration] [dnlib/src/configuration.rs/116] Loaded configuration from "/home/phil/.dnscan/.dnscan.json"
2019-05-30T21:41:41.897609281Z DEBUG [TimerFinished] [dnlib/src/io.rs/67] FIND_FILES, Elapsed=28.835275ms, Dir="/home/phil/mydotnetprojects", NumSolutions=1 NumCsproj=45, NumOtherFiles=12
2019-05-30T21:41:41.955140835Z DEBUG [TimerFinished] [dnlib/src/analysis.rs/93] LOAD_SOLUTIONS, Elapsed=57.451736ms
2019-05-30T21:41:42.136762196Z DEBUG [TimerFinished] [dnlib/src/analysis.rs/108] LOAD_PROJECTS, Elapsed=181.563223ms, Found 43 linked projects and 2 orphaned projects
2019-05-30T21:41:42.136998556Z DEBUG [TimerStarting] [dnscan/src/main.rs/87] CALCULATE_PROJECT_GRAPH
2019-05-30T21:41:42.143072972Z DEBUG [TimerExecuting] [dnscan/src/main.rs/87] CALCULATE_PROJECT_GRAPH, Elapsed=6.075205ms, Individual graphs done
2019-05-30T21:41:42.149218039Z DEBUG [TimerFinished] [dnscan/src/main.rs/87] CALCULATE_PROJECT_GRAPH, Elapsed=12.219438ms, Found 19 redundant project relationships
2019-05-30T21:41:42.165724712Z DEBUG [TimerFinished] [dnscan/src/main.rs/108] WRITE_OUTPUT_FILES, Elapsed=16.459312ms
2019-05-30T21:41:42.166445Z INFO [TimerFinished] [dnscan/src/main.rs/63] DIRECTORY_ANALYSIS, Elapsed=318.48581ms
```

Here the `[Timer*]` blocks are the `target` field from log's [Record](https://docs.rs/log/0.4.6/log/struct.Record.html)
struct and `[dnscan/src/main.rs/63]` is the filename and number from `Record` - this captures the place where the timer was
instantiated. The module is also set, but is not shown in these examples.

# Code Examples

There is also an example program in the examples folder which demonstrates all the
different usages. To run, clone the repository and in Linux do

```sh
RUST_LOG=debug cargo run --example logging_demo
```

If you're on Windows, in PowerShell you can do

```ps
$env:RUST_LOG="debug"
cargo run --example logging_demo
```

# History

See the [CHANGELOG](CHANGELOG.md).

# Performance

The `timer` and `stimer` macros return an `Option<LoggingTimer>`. The method
[log_enabled](https://doc.rust-lang.org/1.1.0/log/macro.log_enabled!.html) is
used to check whether logging is enabled at the requested level. If logging is
not enabled then `None` is returned. This avoids most calculation in the case
where the timer would be a no-op, such that the following loop will create and
drop 1 million timers in about 4ms on my 2012-era Intel i7.

```rs
for _ in 0..1_000_000 {
    let _tmr = stimer!("TEMP");
}
```

In comparison, v0.3 of the library would always return a `LoggingTimer`, and
the loop took ten times longer.

An `Option<LoggingTimer>` is 104 bytes in size on 64-bit Linux.
