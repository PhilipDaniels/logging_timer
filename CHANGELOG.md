# v1.1.1 - 2024-03-10

## Fixed
* Issue https://github.com/PhilipDaniels/logging_timer/issues/4 (unsafeness)
* Issue https://github.com/PhilipDaniels/logging_timer/issues/3 (asyncness)
  Fixed by PR https://github.com/PhilipDaniels/logging_timer/pull/5
  contributed by S0c5.
  The macros now preserve the `unsafe` and `async` keywords when they
  appear in fn signatures. Previously they were stripped by the library.

# v1.1.0 - 2022-01-02

## Fixed

* Issue https://github.com/PhilipDaniels/logging_timer/issues/2
  Previously when using the proc macro timers `time` and `stimer`
  attributes would be stripped from the function. This included doc
  comments. These attributes are now preserved.

# v1.0.0 - 2020-02-09

## Fixed

* It was previously not possible to just use `logging_timers` on its own.
  You also had to add the `log` crate to `Cargo.toml` and add `use`
  statements for some things. This is now corrected, you can just add
  `logging_timers` to your `Cargo.toml` and start using the timers.

## Added

* More helpful doc comments for the `time` and `stime` attribute macros.


# v0.9.2 - 2019-12-29

## Fixed

* The attributes now work on methods that take `&mut self`.


# v0.9.1 - 2019-12-29

## Fixed

* Need to fully qualify the uses of `LogLevel` in `timer!` and `stimer!` macros.


# v0.9 - 2019-12-29

## Added

* Allowed the `time` and `stime` attributes to take two arguments, one
  specifying the level, and one specifying a pattern in which to substitute
  the function name. This helps to disambiguate functions when you have
  many with the same name, which is a tendency in Rust because modules are
  large compared to say, C#, where every class is in its own file.


# v0.5 - 2019-12-28

## Added

* Two new proc-macros, `time` and `stime`, which can be used to instrument a
  function with a timer of the appropriate kind.


# v0.4 - 2019-12-27

## Changed

* The `timer` and `stimer` macros now return an `Option<LoggingTimer>`, resulting
  in a factor-of-10 speed up in the case where logging is disabled for the
  specified level.


# v0.3 - 2019-12-27

## Changed

* Changed the order of the output in the macros so that the name of the timer
  is always printed at the beginning of the log line. This makes it easier to
  visually match up log lines.
* Updated docs, in particular to recommend using UPPERCASE for timer names to
  help make them stand out in the log. The re-ordering change also helps
  with this.

## Added

* This changelog!
* The `timer` and `stimer` macros can now include an optional log level as
  their first argument; to aid parsing it is separated by a semi-colon. For
  example, `let tmr = stimer!(Level::Warn; "S_TIMER_AT_WARN")`.
* Added an example which contains demonstrations of all the different usages.
