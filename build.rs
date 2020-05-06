fn main() {
    dotenv::dotenv().ok();
    let version = rustc_version::version().unwrap();
    println!("cargo:rustc-env=RUSTC_VERSION={}", version);
}
