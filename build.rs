use std::process::Command;

fn main() {
    make_libbootc();
}

fn make_libbootc() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    // this path relates to where build.rs locates
    let build_dir = std::path::absolute("build/boot").unwrap();
    Command::new("make")
        .arg(format!("BUILD_DIR={}", build_dir.to_str().unwrap()))
        .args(["--directory=src/boot", "all"])
        .output()
        .expect("fail to make libbootc");
    // for rustc configure, this path relates to the workspace root
    println!(
        "cargo::rustc-link-search=native={}",
        build_dir.to_str().unwrap()
    );
    println!("cargo::rustc-link-lib=static=bootc");
    println!("cargo::rustc-link-arg=-T{manifest_dir}/src/boot/kernel.ld");
}
