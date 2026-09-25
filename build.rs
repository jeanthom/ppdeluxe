fn main() {
    println!("cargo:rerun-if-changed=src/asm");

    // specify the linker.ld script
    println!("cargo:rustc-link-arg=-Tsrc/asm/linker.ld");
}
