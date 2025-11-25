fn main() {
    let arch = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap();
    let mut env = std::env::var("CARGO_ENV");

    if env.is_err() {
        env = Ok("test".to_string());
    }
    if env.unwrap() != "test" {
        println!("cargo:rustc-link-arg=-Tlinker/{arch}.ld");
        println!("cargo:rerun-if-changed=linker/{arch}.ld");
    }
    println!("cargo:rustc-link-search=extern/liballoc/");
}
