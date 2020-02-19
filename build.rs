use std::env;
use std::path::Path;
use std::process::Command;
use std::str::from_utf8;

fn main() {
    // cd $OUT_DIR
    let out_dir = env::var("OUT_DIR").unwrap();
    env::set_current_dir(&out_dir).unwrap();

    println!("cargo:rustc-link-search={}/bls/lib", out_dir);

    // git clone https://github.com/herumi/mcl
    let ret = Command::new("git")
        .args(&["clone", "https://github.com/herumi/mcl"])
        .output()
        .unwrap();
    if !ret.status.success() {
        println!("{}", from_utf8(&ret.stderr).unwrap());
        return;
    }

    // git clone https://github.com/herumi/bls
    let ret = Command::new("git")
        .args(&["clone", "https://github.com/herumi/bls"])
        .output()
        .unwrap();
    if !ret.status.success() {
        println!("{}", from_utf8(&ret.stderr).unwrap());
        return;
    }

    // cd $OUT_DIR/bls
    env::set_current_dir(Path::new(&out_dir).join("bls")).unwrap();

    // make CXX=clang++
    let ret = Command::new("make").arg("CXX=clang++").output().unwrap();
    if !ret.status.success() {
        println!("{}", from_utf8(&ret.stderr).unwrap());
        return;
    }
}
