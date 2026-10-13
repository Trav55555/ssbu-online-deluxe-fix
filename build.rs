fn main() {
    let proj_dir = env!("CARGO_MANIFEST_DIR");

    println!("cargo:rerun-if-changed=SSBU_VERSION");
    println!(
        "cargo:rustc-env=SSBU_VERSION={}",
        include_str!("SSBU_VERSION").trim()
    );
    println!("cargo:rerun-if-changed={}/lib/", proj_dir);
    println!("cargo:rustc-link-search={}/lib/", proj_dir);
}
