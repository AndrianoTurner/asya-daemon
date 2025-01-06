fn main() {
    // Указание на библиотеку без префикса "lib" и суффикса ".so"
    println!("cargo:rustc-link-lib=plugin_system");

    // Указание на путь, где находится библиотека
    println!("cargo:rustc-link-search=native=/home/eblan/Projects/asya-daemon/target/debug");
}
