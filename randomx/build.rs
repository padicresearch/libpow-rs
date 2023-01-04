use std::env;
use std::error::Error;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn Error>> {
    let build_path = cmake::Config::new("randomx")
        .build_target("randomx")
        .build();

    println!(
        "cargo:rustc-link-search=native={}/build",
        build_path.display()
    );
    println!("cargo:rustc-link-lib=static=randomx");
    let target = env::var("TARGET")?;
    if target.contains("apple") {
        println!("cargo:rustc-link-lib=dylib=c++");
    } else if target.contains("linux") {
        println!("cargo:rustc-link-lib=dylib=stdc++");
    } else if target.contains("windows") {
        println!("cargo:rustc-link-lib=dylib=c++");
    } else {
        unimplemented!();
    }
    let bindings = bindgen::Builder::default()
        .header("randomx/src/randomx.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()?;

    let out_dir = PathBuf::from(env::var("OUT_DIR")?);
    bindings
        .write_to_file(out_dir.join("bindings.rs"))
        .map_err(|e| e.into())
}
