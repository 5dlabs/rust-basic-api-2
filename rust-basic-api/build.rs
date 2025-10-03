#![allow(clippy::disallowed_macros)]

fn main() {
    println!("cargo::rustc-check-cfg=cfg(coverage)");
}
