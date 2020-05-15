fn main() {
    dotenv::dotenv().ok();
    let version = rustc_version::version().unwrap();
    println!("cargo:rustc-env=RUSTC_VERSION={}", version);
    println!("cargo:rustc-env=BRS_TWITCH_CLIENT_ID={}", std::env::var("BRS_TWITCH_CLIENT_ID").unwrap());
    if let Ok(secret) = std::env::var("BRS_TWITCH_CLIENT_SECRET") {
        println!("cargo:rustc-env=BRS_TWITCH_CLIENT_SECRET={}", secret);
    }
}
