use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    let profile = std::env::var("PROFILE").unwrap();
    Command::new("sh")
        .arg("-c")
        .arg("./build_filters.sh")
        .arg(profile.as_str())
        .status()
        .expect("build_filters.sh failed");

    println!("cargo:rustc-link-search=target/filters_build");
    println!("cargo:rustc-link-lib=blocked_bloom_filter");

    // The bindgen::Builder is the main entry point
    // to bindgen, and lets you build up options for
    // the resulting bindings.
    let bindings = bindgen::Builder::default()
        // The input header we would like to generate
        // bindings for.
        .header("filters/include/blocked_bloom_filter.h")
        // Tell cargo to invalidate the built crate whenever any of the
        // included header files changed.
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bbf_bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bbf_bindings.rs"))
        .expect("Couldn't write bindings!");
}