fn main() {
    // Write config.toml next to the compiled binary if it isn't already there,
    // so the binary has a config ready without needing a first run to generate one.
    let out_dir = std::env::var("OUT_DIR").unwrap();
    // OUT_DIR = target/{profile}/build/{pkg}-{hash}/out — three levels up is target/{profile}/
    let target_dir = std::path::PathBuf::from(&out_dir)
        .ancestors()
        .nth(3)
        .unwrap()
        .to_path_buf();

    let dest = target_dir.join("config.toml");
    if !dest.exists() {
        let toml = std::fs::read_to_string("src/default_config.toml").unwrap();
        std::fs::write(&dest, toml).ok();
    }

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src/default_config.toml");
}
