use std::path::Path;

fn main() {
    // Re-run build script if the force-build env changes
    println!("cargo:rerun-if-env-changed=ZK_GUEST_FORCE_BUILD");
    // Optional override to force building the guest even if a prebuilt artifact exists.
    // Set ZK_GUEST_FORCE_BUILD=1 to rebuild the ELF from source.
    let force_build = std::env::var("ZK_GUEST_FORCE_BUILD").map_or(false, |v| v == "1");

    // Check if pre-built ELF exists (for Docker builds without SP1 toolchain)
    let prebuilt_elf = Path::new("../.artifacts/zk-guest-sp1-guest");

    if prebuilt_elf.exists() && !force_build {
        println!("cargo:warning=Using pre-built ELF from .artifacts directory");
        println!("cargo:rerun-if-changed=../.artifacts/zk-guest-sp1-guest");

        // Copy pre-built ELF to the expected location
        let target_dir = std::env::var("OUT_DIR").unwrap();
        let target_elf_dir =
            Path::new(&target_dir).join("elf-compilation/riscv32im-succinct-zkvm-elf/release");
        std::fs::create_dir_all(&target_elf_dir).expect("Failed to create ELF directory");

        let target_elf = target_elf_dir.join("zk-guest-sp1-guest");
        std::fs::copy(prebuilt_elf, &target_elf).expect("Failed to copy pre-built ELF");

        // Make it executable
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&target_elf).unwrap().permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(&target_elf, perms).unwrap();
        }

        // Set the environment variable that include_elf! expects
        // This tells the macro where to find the ELF at compile time
        println!(
            "cargo:rustc-env=SP1_ELF_zk-guest-sp1-guest={}",
            target_elf.display()
        );
        println!(
            "cargo:warning=Pre-built ELF copied to: {}",
            target_elf.display()
        );
        return;
    }

    // Either prebuilt not found or force-build requested
    #[cfg(feature = "build-guest")]
    {
        if force_build {
            println!("cargo:warning=ZK_GUEST_FORCE_BUILD=1 set; rebuilding guest ELF from source");
        } else {
            println!(
                "cargo:warning=Pre-built ELF not found, building guest program with SP1 toolchain"
            );
        }
        println!("cargo:rerun-if-changed=../guest");
        sp1_build::build_program("../guest");
    }

    #[cfg(not(feature = "build-guest"))]
    {
        panic!("Pre-built ELF not found at {} and build-guest feature is disabled. Please build the ELF or enable the build-guest feature.", prebuilt_elf.display());
    }
}
