// use std::env;
// Zuse std::fs;
// use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    // let host = env::var("HOST").unwrap();
    // let target = env::var("TARGET").unwrap();

    // let host_and_target_contain = |s| host.contains(s) && target.contains(s);
    let mut cfg = cc::Build::new();

    // If we've gotten this far we're probably a pretty standard platform.
    // Almost all platforms here ship libz by default, but some don't have
    // pkg-config files that we would find above.
    //
    // In any case test if libagc is actually installed and if so we link to it,
    // otherwise continue below to build things.
    if libagc_installed(&mut cfg) {
        println!("cargo:rustc-link-lib=agc");
        println!("cargo:warning=\"TEST3\"");
        return;
    }

    println!("cargo:warning=\"TEST1a\"");

    // For convenience fallback to building libagc if attempting to link libagc failed
    // build_libagc(&mut cfg, &target)
}

fn libagc_installed(cfg: &mut cc::Build) -> bool {
    let mut cmd = cfg.get_compiler().to_command();
    println!("cargo:warning=\"TEST2\"");
    cmd.arg("test/smoke.c")
        // .arg("-Wno-unused-parameter")
        .arg("-g0")
        .arg("-o")
        .arg("/dev/null")
        .arg("-lzstd")
        .arg("-llibagc");

    println!("running {:?}", cmd);
    if let Ok(status) = cmd.status() {
        if status.success() {
            return true;
        }
    }
    println!("cargo:warning=\"TEST4\"");
    false
}
