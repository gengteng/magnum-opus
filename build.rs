use std::{
    env,
    path::{Path, PathBuf},
};

fn find_package(name: &str) -> Vec<PathBuf> {
    let library = vcpkg::find_package(name).expect("Failed to find package");
    println!("cargo:info={}", library.vcpkg_triplet); //TODO
    let lib_name = name.trim_start_matches("lib").to_string();
    println!("{}", format!("cargo:rustc-link-lib=static={}", lib_name));

    match (library.link_paths.as_slice(), library.include_paths.as_slice()) {
        ([link_search, ..], [include, ..]) => {
            println!(
                "{}",
                format!("cargo:rustc-link-search={}", link_search.display())
            );
            println!("{}", format!("cargo:include={}", include.display()));
        }
        _ => {
            panic!(
                "{}",
                if library.link_paths.is_empty() {
                    "link path not found"
                } else {
                    "include path not found"
                }
            )
        }
    }

    library.include_paths
}

fn generate_bindings(ffi_header: &Path, include_paths: &[PathBuf], ffi_rs: &Path) {
    #[derive(Debug)]
    struct ParseCallbacks;
    impl bindgen::callbacks::ParseCallbacks for ParseCallbacks {
        fn int_macro(&self, name: &str, _value: i64) -> Option<bindgen::callbacks::IntKind> {
            if name.starts_with("OPUS") {
                Some(bindgen::callbacks::IntKind::Int)
            } else {
                None
            }
        }
    }
    let mut b = bindgen::Builder::default()
        .header(ffi_header.to_str().unwrap())
        .parse_callbacks(Box::new(ParseCallbacks))
        .generate_comments(false);

    for dir in include_paths {
        b = b.clang_arg(format!("-I{}", dir.display()));
    }

    b.generate().unwrap().write_to_file(ffi_rs).unwrap();
}

fn gen_opus() {
    let includes = find_package("opus");
    let src_dir = env::var_os("CARGO_MANIFEST_DIR").unwrap();
    let src_dir = Path::new(&src_dir);
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let out_dir = Path::new(&out_dir);

    let ffi_header = src_dir.join("opus_ffi.h");
    println!("rerun-if-changed={}", ffi_header.display());
    for dir in &includes {
        println!("rerun-if-changed={}", dir.display());
    }

    let ffi_rs = out_dir.join("opus_ffi.rs");
    generate_bindings(&ffi_header, &includes, &ffi_rs);
}

fn main() {
    gen_opus()
}
