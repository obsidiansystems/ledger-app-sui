fn main() {
    println!("cargo:rerun-if-changed=script.ld");
    let profile = std::env::var("PROFILE").unwrap();
    let debug_print = std::env::var("CARGO_FEATURE_SPECULOS").is_ok();
    let extra_debug_print = std::env::var("CARGO_FEATURE_EXTRA_DEBUG").is_ok();
    let reloc_size = match (profile.as_str(), debug_print, extra_debug_print) {
        ("release", false, false) => 2864,
        (_, _, true) => 1024 * 10,
        _ => 1024 * 7,
    };
    println!("cargo:rustc-link-arg=--defsym=_reloc_size={reloc_size}");
}
