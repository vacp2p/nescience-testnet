use std::{env, fs, path::Path, process::Command};
use risc0_build::a;

fn main() {
    // 1️⃣ Crate root and OUT_DIR
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let out_dir = env::var("OUT_DIR").unwrap();

    // 2️⃣ Directory to write generated module
    let mod_dir = Path::new(&out_dir).join("nssa_programs");
    let mod_file = mod_dir.join("mod.rs");

    println!("cargo:rerun-if-changed=program_methods/guest");

    // 3️⃣ Build the Risc0 guest program
    let guest_manifest = Path::new(&manifest_dir)
        .join("program_methods/guest/Cargo.toml");

    let status = Command::new("cargo")
        .arg("risczero")
        .arg("build")
        .arg("--manifest-path")
        .arg(&guest_manifest)
        .status()
        .expect("failed to run risczero build");
    assert!(status.success(), "Risc0 deterministic build failed");

    // 4️⃣ Target directory where the Risc0 build produces .bin files
    let target_dir = Path::new(&manifest_dir)
        .join("program_methods/guest/target/riscv32im-risc0-zkvm-elf/docker/");

    println!("cargo:warning=Looking for binaries in {}", target_dir.display());

    // 5️⃣ Collect all .bin files
    let bins = fs::read_dir(&target_dir)
        .expect("failed to read external target dir")
        .filter_map(Result::ok)
        .filter(|e| e.path().extension().map(|ext| ext == "bin").unwrap_or(false))
        .collect::<Vec<_>>();

    if bins.is_empty() {
        panic!("No .bin files found in {:?}", target_dir);
    }

    println!("cargo:warning=Found {} binaries:", bins.len());
    for b in &bins {
        println!("cargo:warning= - {}", b.path().display());
    }

    // 6️⃣ Generate Rust module
    fs::create_dir_all(&mod_dir).unwrap();
    let mut src = String::new();
    for entry in bins {
        let path = entry.path();
        let name = path.file_stem().unwrap().to_string_lossy();
        src.push_str(&format!(
            "pub const {}_ELF: &[u8] = include_bytes!(r#\"{}\"#);\n",
            name.to_uppercase(),
            path.display()
        ));
    }

    fs::write(&mod_file, src).unwrap();
    println!("cargo:warning=Generated module at {}", mod_file.display());
}

