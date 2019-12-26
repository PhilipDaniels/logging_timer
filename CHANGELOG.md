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
