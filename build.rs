fn main() {
    let version = rustc_version::version().unwrap();
    println!("cargo:rust-env=RUSTC_VERSION={}", version);
}
