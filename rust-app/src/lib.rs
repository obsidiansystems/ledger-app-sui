#![no_std]
#![allow(incomplete_features)]
#![feature(const_generics)]

#![cfg_attr(all(target_os="nanos", test), no_main)]
#![cfg_attr(target_os="nanos", feature(custom_test_frameworks))]
#![reexport_test_harness_main = "test_main"]
#![cfg_attr(target_os="nanos", test_runner(nanos_sdk::sdk_test_runner))]

#[cfg(all(target_os = "nanos", test))]
#[no_mangle]
extern "C" fn sample_main() {
    use nanos_sdk::exit_app;
    test_main();
    exit_app(0);
}

pub mod interface;

#[cfg(all(target_os = "nanos"))]
pub mod implementation;
#[cfg(all(target_os = "nanos"))]
pub mod crypto_helpers;


#[cfg(all(target_os="nanos", test))]
use core::panic::PanicInfo;
/// In case of runtime problems, return an internal error and exit the app
#[cfg(all(target_os="nanos", test))]
#[inline]
#[cfg_attr(all(target_os="nanos", test), panic_handler)]
pub fn exiting_panic(info: &PanicInfo) -> ! {
    //let mut comm = io::Comm::new();
    //comm.reply(io::StatusWords::Panic);
    write!(DBG, "Panicking: {:?}\n", info);
    nanos_sdk::exit_app(1)
}

/// Custom type used to implement tests
//#[cfg(all(target_os = "nanos", test))]
//use nanos_sdk::TestType;

#[cfg(all(target_os = "nanos", speculos))]
use nanos_sdk::debug_print;

pub struct DBG;
use core;
use arrayvec::ArrayString;
#[cfg(all(target_os = "nanos", speculos))]
impl core::fmt::Write for DBG {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        // Dunno why the copy is required, might be some pic issue as this is going straight to
        // assembly.
        let mut qq = ArrayString::<128>::new();
        qq.push_str(s);
        debug_print(qq.as_str());
        Ok(())
    }
}
#[cfg(all(target_os = "nanos", not(speculos)))]
impl core::fmt::Write for DBG {
    fn write_str(&mut self, _s: &str) -> core::fmt::Result {
        Ok(())
    }
}

