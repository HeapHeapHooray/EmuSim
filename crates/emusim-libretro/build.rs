fn main() {
    cc::Build::new()
        .file("c_src/logger.c")
        .compile("emusim_libretro_c");
    println!("cargo:rerun-if-changed=c_src/logger.c");
}
