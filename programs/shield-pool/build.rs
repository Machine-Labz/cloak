use std::fs;
use std::path::Path;

fn main() {
    // Look for vkey_hash.txt in multiple locations
    let vkey_files = [
        "target/vkey_hash.txt",
        "../../target/vkey_hash.txt",
        "../../../target/vkey_hash.txt",
        "packages/vkey-generator/target/vkey_hash.txt",
        "vkey_hash.txt",       // fallback for old location
        "../../vkey_hash.txt", // fallback for old location
    ];

    let vkey_hash = vkey_files
        .iter()
        .find_map(|path| {
            if Path::new(path).exists() {
                match fs::read_to_string(path) {
                    Ok(content) => {
                        let trimmed = content.trim();
                        if !trimmed.is_empty() {
                            println!("cargo:warning=Using VKey hash from: {}", path);
                            Some(trimmed.to_string())
                        } else {
                            None
                        }
                    }
                    Err(_) => None,
                }
            } else {
                None
            }
        })
        .unwrap_or_else(|| {
            // Fallback to hardcoded VKey if no file found
            println!("cargo:warning=No vkey_hash.txt found, using fallback VKey hash");
            "0x0064c7b959bfd20407b69859a8126b8efaa6df25191373b91cb78eb03a0bd12f".to_string()
        });

    // Set the VKey hash as an environment variable for the program
    println!("cargo:rustc-env=VKEY_HASH={}", vkey_hash);

    // Rebuild if vkey_hash.txt changes
    for file in &vkey_files {
        if Path::new(file).exists() {
            println!("cargo:rerun-if-changed={}", file);
        }
    }
}
