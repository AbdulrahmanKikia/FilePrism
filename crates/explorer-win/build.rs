fn main() {
    #[cfg(all(windows, target_env = "msvc"))]
    {
        let def = std::path::Path::new("exports.def");
        if def.exists() {
            let abs = std::fs::canonicalize(def).expect("canonicalize exports.def");
            println!("cargo:rustc-cdylib-link-arg=/DEF:{}", abs.display());
        }
    }
}
