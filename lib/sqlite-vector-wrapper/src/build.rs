extern crate cmake;
use cmake::Config;

fn main() {
    let dst = Config::new("sqlite-vector")
        .build_target("sqlite-vector")
        .build();

    println!("cargo:rustc-link-search=native={}", dst.display());
    // println!("cargo:rustc-link-lib=static=foo");
}
