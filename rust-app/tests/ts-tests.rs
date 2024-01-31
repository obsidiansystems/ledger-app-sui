#![no_std]
#![feature(custom_test_frameworks)]
#![test_runner(crate::my_runner)]
#![no_main]

use ledger_device_sdk::exit_app;

use alamgu_example::main_nanos::*;

#[no_mangle]
extern "C" fn sample_main() {
    app_main()
}

fn my_runner(_: &[&i32]) {}

use core::panic::PanicInfo;
#[panic_handler]
fn panic_handler(_: &PanicInfo) -> ! {
    exit_app(0);
}
