# Logging Timers for Rust

A Rust timer that logs (via the [log](https://crates.io/crates/log) crate)
the elapsed time when it is dropped. Timers have names, and the log record
includes the module, file and line number of the place that they were
instantiated, making it easy to filter logs for particular timer messages.

Timers are most easily created using the `timer!` or `stimer` macros.
