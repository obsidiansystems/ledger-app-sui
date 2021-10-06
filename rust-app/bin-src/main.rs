#![cfg_attr(target_os = "nanos", no_std)]
#![cfg_attr(target_os = "nanos", no_main)]

#[cfg(not(target_os = "nanos"))]
fn main() {}

#[cfg(target_os = "nanos")]
mod main_nanos;
#[cfg(target_os = "nanos")]
pub use main_nanos::*;
