fn main() {
    // unrar_sys (UnRAR) needs these Windows system libs; not always pulled in by the crate.
    #[cfg(target_os = "windows")]
    {
        println!("cargo:rustc-link-lib=advapi32");
        println!("cargo:rustc-link-lib=crypt32");
    }
}
