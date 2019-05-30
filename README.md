# Logging Timers for Rust

Provides a couple of simple timers that log messages indicating the elapsed time between
their creation and dropping. Messages are output via the [log](https://crates.io/crates/log)
crate.

Timers have names, and the log messages are constructed in such a way that they contain
the module, filename and line number of the place where the timer was constructed.

Timers are usually created using the `timer!` or `stimer!` macros. The difference is
that `timer!` returns a timer that logs a message only when it is dropped, while `stimer!`
returns a timer that logs a started message as soon as it is created, and a finished
message when it is dropped.

Example - "Find Files" is the name of the timer:

```rust
use logging_timer::{timer};

fn find_files(dir: PathBuf) -> List<PathBuf> {
    let _tmr = timer!("Find Files");
    let files = vec![];
    // expensive operation here
    return files;
} // _tmr is dropped here and a 'TimerFinished' message is logged
```

You can replace `timer!` with `stimer!` to get a timer that logs a starting message as
well, giving you a pair of 'bracketing' log messages.

In addition, both timer macros accept [format_args!](https://doc.rust-lang.org/std/macro.format_args.html)
style parameters, allowing you to include extra information in the log messages.

```norun
let tmr = timer!("Find Files", "Directory = {}", dir);
```

See the module documentation for more examples and an example of the output format.
