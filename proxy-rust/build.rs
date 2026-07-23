use std::path::Path;

fn main() {
    let path = Path::new("../index.html");
    println!("cargo:rerun-if-changed=../index.html");
    if !path.exists() {
        panic!(
            "index.html not found at {} (cwd: {:?}).\n\
             cargo run/build must be executed from the proxy-rust/ subdirectory \
             inside the project root, e.g. `cd proxy-rust && cargo build --release`.",
            path.display(),
            std::env::current_dir().unwrap_or_default()
        );
    }
}