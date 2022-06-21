#![no_std]
#![feature(custom_test_frameworks)]
#![test_runner(crate::my_runner)]
#![no_main]

use nanos_sdk::{debug_print, exit_app};

#[no_mangle]
extern "C" fn sample_main() {
    debug_print("\nEXE: ");
    debug_print(EXE_PATH);
    debug_print("\n");
    exit_app(0);
}

fn my_runner(_: &[&i32]) {
}

use core::panic::PanicInfo;
#[panic_handler]
fn panic_handler(_: &PanicInfo) -> ! {
    exit_app(0);
}

pub static EXE_PATH : &str = env!("CARGO_BIN_EXE_rust-app");


// Stub to trigger a build with ts-tests; this will actually be intercepted by speculos-wrapper to
// as a trigger to run the typescript tests.
