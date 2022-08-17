#![cfg_attr(target_family = "bolos", no_std)]
#![cfg_attr(target_family = "bolos", no_main)]

#[cfg(not(target_family = "bolos"))]
fn main() {}

use rust_app::main_nanos::*;

nanos_sdk::set_panic!(nanos_sdk::exiting_panic);

#[no_mangle]
extern "C" fn sample_main() {
    app_main()
}
