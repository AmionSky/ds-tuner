use libbpf_cargo::SkeletonBuilder;
use std::io::Error as IoError;
use std::path::{Path, PathBuf};

const BPF_SRC: &str = "./bpf";

fn main() {
    let src = Path::new(BPF_SRC).join("dualsense.bpf.c");
    let header = Path::new(BPF_SRC).join("dualsense.h");

    link_vmlinux().expect("Failed to link 'vmlinux.h'");

    SkeletonBuilder::new()
        .source(&src)
        .build_and_generate(out_file("dualsense.skel.rs"))
        .unwrap();

    println!("cargo:rerun-if-changed={}", src.display());
    println!("cargo:rerun-if-changed={}", header.display());
}

fn out_file(name: &str) -> PathBuf {
    let out_dir = std::env::var_os("OUT_DIR").expect("'OUT_DIR' was not specified!");
    Path::new(&out_dir).join(name)
}

fn link_vmlinux() -> Result<(), IoError> {
    const NAME: &str = "vmlinux.h";

    // Select vmlinux.h from the first directory in modules
    let source = std::fs::read_dir("/usr/lib/modules/")?
        .filter_map(|res| res.ok())
        .map(|entry| entry.path())
        .next()
        .ok_or(IoError::other("/usr/lib/modules directory is empty"))?
        .join("build")
        .join(NAME);

    let target = Path::new(BPF_SRC).join(NAME);
    if target.is_symlink() {
        std::fs::remove_file(&target)?;
    }
    std::os::unix::fs::symlink(source, target)?;

    Ok(())
}
