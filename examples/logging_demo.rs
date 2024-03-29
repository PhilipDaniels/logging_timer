#![allow(dead_code)]
#![allow(unused_imports)]

use chrono::{DateTime, Utc};
use env_logger::Builder;
use logging_timer::{executing, finish, stime, stimer, time, timer};
use std::{default, io::Write, time::Duration};
use tokio::*;

/// Demonstrates the various timer macros.
///
/// To run in Linux, do:
///     RUST_LOG=debug cargo run --example logging_demo
///
/// To run in PowerShell, do:
///     $env:RUST_LOG="debug"
///     cargo run --example logging_demo
#[tokio::main]
async fn main() {
    configure_logging();

    let _main_tmr = stimer!(log::Level::Error; "MAIN");

    // For my info only.
    // println!("Size(_main_tmr.level)       = {}", std::mem::size_of_val(&_main_tmr.level));
    // println!("Size(_main_tmr.file)        = {}", std::mem::size_of_val(&_main_tmr.file));
    // println!("Size(_main_tmr.module_path) = {}", std::mem::size_of_val(&_main_tmr.module_path));
    // println!("Size(_main_tmr.line)        = {}", std::mem::size_of_val(&_main_tmr.line));
    // println!("Size(_main_tmr.finished)    = {}", std::mem::size_of_val(&_main_tmr.finished));
    // println!("Size(_main_tmr.start_time)  = {}", std::mem::size_of_val(&_main_tmr.start_time));
    // println!("Size(_main_tmr.name)        = {}", std::mem::size_of_val(&_main_tmr.name));
    // println!("Size(_main_tmr.extra_info)  = {}", std::mem::size_of_val(&_main_tmr.extra_info));
    //println!("Size(_main_tmr)             = {}", std::mem::size_of_val(&_main_tmr));
    //println!("Size(x)             = {}", std::mem::size_of_val(&x));

    // Create and drop a lot of timers for performance comparisons.
    // for _ in 0..1_000_000 {
    //      let _tmr = stimer!("TEMP");
    // }

    test_time_macro();
    println!("");

    test_stime_macro();
    println!("");

    let mut x = 3;
    test_time_macro_with_mut_parameters(&mut x, 12);
    println!("");

    test_mut_self();
    println!("");

    test_hygiene();
    println!("");

    test_stime_macro_with_level_and_pattern();
    println!("");

    test_stime_macro_with_pattern();
    println!("");

    test_stime_macro_with_no_brackets_pattern();
    println!("");

    test_stime_macro_with_never();
    println!("");

    timer_with_name_only();
    println!("");

    stimer_with_name_only();
    println!("");

    stimer_with_intermediate_messages_and_final_message();
    println!("");

    stimer_with_intermediate_messages_and_no_automatic_final_message();
    println!("");

    timer_with_inline_log_level();
    println!("");

    stimer_with_inline_log_level();
    println!("");

    stimer_with_args();
    println!("");

    executing_with_args();
    println!("");

    finish_with_args();
    println!("");

    execute_and_finish_without_args();
    println!("");

    executed_by_async().await;
    println!("");

    unsafe {
        unsafe_fn();
    }
    println!("");

    async_trait_example().await;
}

struct Foo {
    x: i32,
}

#[time]
pub unsafe fn unsafe_fn() {
    println!("hello world");
}

#[async_trait::async_trait]
trait Walker {
    async fn walk(&self) -> bool;
}

#[derive(Default)]
struct Animal;

#[async_trait::async_trait]
impl Walker for Animal {
    #[time]
    async fn walk(&self) -> bool {
        time::sleep(Duration::from_secs(2)).await;
        false
    }
}

async fn async_trait_example() {
    let dog = Animal::default();
    dog.walk().await;
}

impl Foo {
    // v0.9 of the library had a bug where this would fail to compile.
    #[stime]
    fn bar(&mut self) {
        self.x = 12;
    }
}

// Section 0. The attribute-based timers.
#[time]
fn test_time_macro() {}

#[time]
fn test_time_macro_with_mut_parameters(_a: &mut i32, mut _x: i32) {}

fn test_mut_self() {
    let mut f = Foo { x: 4 };
    f.bar();
}

#[time]
fn test_hygiene() {
    let _tmr = 3;
}

#[stime("warn")]
fn test_stime_macro() {}

#[stime("warn", "FirstStruct::{}")]
fn test_stime_macro_with_level_and_pattern() {}

#[stime("SecondStruct::{}/blah")]
fn test_stime_macro_with_pattern() {}

#[stime("NOBRACKETS")]
fn test_stime_macro_with_no_brackets_pattern() {}

#[stime("never", "ComplexPattern::{}::I Don't Want To Delete Yet")]
fn test_stime_macro_with_never() {
    // Nothing should be logged
}

// Section 1. Basic operation of all macros.
fn timer_with_name_only() {
    let _tmr = timer!("NAMED_TIMER");
}

fn stimer_with_name_only() {
    let _tmr = stimer!("NAMED_S_TIMER");
}

fn stimer_with_intermediate_messages_and_final_message() {
    let tmr = stimer!("S_TIMER_INTER_FINAL");
    executing!(tmr, "Stuff is happening");
    executing!(tmr, "More stuff is happening");
}

fn stimer_with_intermediate_messages_and_no_automatic_final_message() {
    let tmr = stimer!("S_TIMER_INTER_NOFINAL");
    executing!(tmr, "Stuff is happening");
    executing!(tmr, "More stuff is happening");
    finish!(tmr, "All done. Frobbed 5 wuidgets.");
}

// Section 2. Changing the log level.
fn timer_with_inline_log_level() {
    let _tmr1 = timer!(log::Level::Info; "TIMER_AT_INFO", "Got {} widgets", 5);
    let _tmr2 = timer!(log::Level::Warn; "TIMER_AT_WARN");
    let _tmr3 = timer!(log::Level::Error; "TIMER_AT_ERROR", "more info");
}

fn stimer_with_inline_log_level() {
    let _tmr1 = stimer!(log::Level::Info; "S_TIMER_AT_INFO", "Got {} widgets", 5);
    let _tmr2 = stimer!(log::Level::Warn; "S_TIMER_AT_WARN");
    let _tmr3 = stimer!(log::Level::Error; "S_TIMER_AT_ERROR", "more info");
}

// Section 3. Using format args.
fn stimer_with_args() {
    let _tmr = stimer!("FORMATTED_S_TIMER", "extra info");
    let _tmr2 = stimer!("FORMATTED_S_TIMER2", "extra info: {} widgets", 5);
}

fn executing_with_args() {
    let tmr = stimer!("EXEC_WITH_ARGS", "Expecting to process {} widgets", 20);
    executing!(tmr);
    executing!(tmr, "More info: Processed {} widgets", 5);
    executing!(tmr, "More info: Processed {} widgets", 10);
}

fn finish_with_args() {
    let tmr = stimer!("FINISH_WITH_ARGS", "Expecting to process {} widgets", 20);
    executing!(tmr, "More info: Processed {} widgets", 10);
    executing!(tmr, "More info: Processed {} widgets", 20);
    finish!(tmr, "Done. Processed {} widgets", 20);
}

fn execute_and_finish_without_args() {
    let tmr = stimer!("WITHOUT_ARGS", "Expecting to process {} widgets", 20);
    executing!(tmr);
    finish!(tmr);
}

trait AsyncFoo {
    async fn foo(&self);
}

#[derive(Default)]
struct AsyncOof {}

impl AsyncFoo for AsyncOof {
    #[time("AsyncFoo::{}")]
    async fn foo(&self) {
        time::sleep(Duration::from_millis(10000)).await;
    }
}

async fn executed_by_async() {
    let foo_async = AsyncOof::default();
    foo_async.foo().await;
}

// Just configures logging in such a way that we can see everything.
fn configure_logging() {
    let mut builder = Builder::from_default_env();
    builder.format(|buf, record| {
        let utc: DateTime<Utc> = Utc::now();

        write!(
            buf,
            "{:?} {} [{}] ",
            //utc.format("%Y-%m-%dT%H:%M:%S.%fZ"),
            utc, // same, probably faster?
            record.level(),
            record.target()
        )?;

        match (record.file(), record.line()) {
            (Some(file), Some(line)) => write!(buf, "[{}/{}] ", file, line),
            (Some(file), None) => write!(buf, "[{}] ", file),
            (None, Some(_line)) => write!(buf, " "),
            (None, None) => write!(buf, " "),
        }?;

        writeln!(buf, "{}", record.args())
    });

    builder.init();
}
