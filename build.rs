fn main() {
    let dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    
    println!("cargo:rerun-if-changed=build.rs");
    
    // Link tree-sitter-typescript
    println!("cargo:rustc-link-search=native={}/target/debug/build", dir);
}
