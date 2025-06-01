use libbpf_cargo::SkeletonBuilder;
use std::path::{Path, PathBuf};

const BPF_SRC: &str = "./bpf";

fn main() {
    let out_dir = PathBuf::from(std::env::var_os("OUT_DIR").expect("'OUT_DIR' was not specified!"));

    let src = Path::new(BPF_SRC).join("dualsense.bpf.c");
    let header = Path::new(BPF_SRC).join("dualsense.h");

    SkeletonBuilder::new()
        .source(&src)
        .build_and_generate(out_dir.join("dualsense.skel.rs"))
        .unwrap();

    println!("cargo:rerun-if-changed={}", src.display());
    println!("cargo:rerun-if-changed={}", header.display());
}
