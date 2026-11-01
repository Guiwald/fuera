//! Icon of the Windows executable.

fn main() {
    println!("cargo:rerun-if-changed=../assets/icons/fuera.rc");
    println!("cargo:rerun-if-changed=../assets/icons/fuera.ico");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let _ = embed_resource::compile("../assets/icons/fuera.rc", embed_resource::NONE);
    }
}
