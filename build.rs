use std::env;
use std::path::Path;
use std::process::Command;
use std::str::from_utf8;

const LIB_REPO: &str = "bls-eth-go-binary";
const LIB_DIR: &str = "bls/lib/linux/amd64";

fn main() {
    // get $OUT_DIR
    let out_dir = env::var("OUT_DIR").unwrap();

    // RUSTFLAGS=-L$OUT_DIR/bls-eth-go-binary/bls/lib/linux/amd64
    println!(
        "cargo:rustc-link-search={}/{}/{}",
        out_dir, LIB_REPO, LIB_DIR
    );

    // cd $OUT_DIR
    env::set_current_dir(&out_dir).unwrap();

    // git clone https://github.com/herumi/bls-eth-go-binary
    let ret = Command::new("git")
        .args(&["clone", &format!("https://github.com/herumi/{}", LIB_REPO)])
        .output()
        .unwrap();
    if !ret.status.success() {
        println!("{}", from_utf8(&ret.stderr).unwrap());
        return;
    }
}
