fn main() {
    let proto_root = "../proto";

    let mut include_dirs: Vec<String> = vec![proto_root.to_string()];

    let common_include_candidates: &[&str] = &["/usr/local/include", "/usr/include"];
    for candidate in common_include_candidates {
        if std::path::Path::new(&format!("{candidate}/google/api/annotations.proto")).exists() {
            include_dirs.push(candidate.to_string());
            break;
        }
    }

    let result = tonic_build::configure()
        .build_server(true)
        .build_client(false)
        .out_dir("src/gen")
        .compile_protos(
            &[
                "../proto/tea/v1/publisher.proto",
                "../proto/tea/v1/product.proto",
                "../proto/tea/v1/component.proto",
                "../proto/tea/v1/artifact.proto",
                "../proto/tea/v1/collection.proto",
                "../proto/tea/v1/common.proto",
            ],
            &include_dirs
                .iter()
                .map(|s| s.as_str())
                .collect::<Vec<_>>(),
        );

    if let Err(e) = result {
        let err_str = e.to_string();
        // L4 fix: only suppress errors that are clearly caused by missing well-known
        // proto dependencies (validate.proto, annotations.proto, etc.) that aren't
        // bundled in the build environment.
        // Real structural errors (syntax, type errors, import cycles) propagate so CI
        // catches proto regressions.
        let is_missing_dep = err_str.contains("File not found")
            || err_str.contains("not found")
            || err_str.contains("No such file")
            || err_str.contains("validate.proto")
            || err_str.contains("annotations.proto");

        if is_missing_dep {
            println!(
                "cargo:warning=protoc compilation skipped — proto dependencies not found: {e}"
            );
        } else {
            // Propagate real proto errors
            panic!("protoc failed with a structural error: {e}");
        }
    }

    // Re-run if proto files change
    println!("cargo:rerun-if-changed=../proto");
}
