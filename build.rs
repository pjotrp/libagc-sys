// Run with
//
//   cargo build -vv

use std::env;
// Zuse std::fs;
// use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    // let host = env::var("HOST").unwrap();
    // let target = env::var("TARGET").unwrap();

    // let host_and_target_contain = |s| host.contains(s) && target.contains(s);
    // let mut cfg = cc::Build::new().cpp(true);
    let mut binding = cc::Build::new();
    let mut cfg = binding.cpp(true);

    // If we've gotten this far we're probably a pretty standard platform.
    // Almost all platforms here ship libz by default, but some don't have
    // pkg-config files that we would find above.
    //
    // In any case test if libagc is actually installed and if so we link to it,
    // otherwise continue below to build things.
    if libagc_installed(&mut cfg) {
        println!("cargo::warning=\"Linking against system libagc!\"");
        println!("cargo::rustc-link-lib=agc");
        return;
    }

    println!("cargo::error=\"Should not get here\"");

    // For convenience fallback to building libagc if attempting to link libagc failed
    // build_libagc(&mut cfg, &target)
}

fn libagc_installed(cfg: &mut cc::Build) -> bool {
    let guixenv =
        match env::var("GUIX_ENVIRONMENT") {
            Ok(val) => val,
            Err(_e) => "".to_string(),
        };
    let mut cmd = cfg.get_compiler().to_command();
    println!("cargo::warning=\"Looking for system libagc\"");
    cmd.arg("test/smoke.cpp")
        // .arg("-Wno-unused-parameter")
        .arg("-g0")
        .arg("-o")
        .arg("/dev/null")
        .arg(guixenv.to_string()+"/lib/mimalloc-2.1/libmimalloc.a")
        .arg("-lzstd")
        .arg(guixenv.to_string()+"/lib/libagc.a");

    println!("cargo::warning=\"running {:?}\"", cmd);
    if let Ok(status) = cmd.status() {
        if status.success() {
            return true;
        }
    }
    println!("cargo::warning=\"linking libagc.a failed!\"");
    false
}
