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
