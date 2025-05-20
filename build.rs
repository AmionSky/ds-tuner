use libbpf_cargo::SkeletonBuilder;
use std::io::Error as IoError;
use std::path::{Path, PathBuf};

const SRC: &str = "src/bpf/dualsense.bpf.c";

fn main() {
    link("vmlinux.h").expect("Failed to link 'vmlinux.h'");

    SkeletonBuilder::new()
        .source(SRC)
        .build_and_generate(out_file("dualsense.skel.rs"))
        .unwrap();

    println!("cargo:rerun-if-changed={}", SRC);
}

fn out_file(name: &str) -> PathBuf {
    let out_dir = std::env::var_os("OUT_DIR").unwrap();
    Path::new(&out_dir).join(name)
}

fn link<P: AsRef<Path>>(file: P) -> Result<(), IoError> {
    let name = file
        .as_ref()
        .file_name()
        .expect("Link target isn't a file!");

    let source = std::fs::read_dir("/usr/lib/modules/")?
        .filter_map(|res| res.ok())
        .map(|entry| entry.path())
        .next()
        .ok_or(IoError::other("modules directory is empty"))?
        .join("build")
        .join(file.as_ref());

    let target = Path::new("src/bpf").join(name);
    if target.exists() {
        std::fs::remove_file(&target)?;
    }
    std::os::unix::fs::symlink(source, target)?;

    Ok(())
}
