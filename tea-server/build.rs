use std::{
    fs,
    path::{Path, PathBuf},
};

fn main() {
    let proto_root = "../proto";
    let buf_export_dir = format!("{proto_root}/.buf-deps");

    if !Path::new(&buf_export_dir).exists() {
        panic!(
            "missing exported Buf dependencies at {buf_export_dir}; run `make -C ../proto export-deps` before building tea-server"
        );
    }

    // Compile the full TEA surface so Rust bindings stay in lockstep with Buf.
    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .compile_protos(
            &[
                "../proto/tea/v1/common.proto",
                "../proto/tea/v1/product.proto",
                "../proto/tea/v1/component.proto",
                "../proto/tea/v1/artifact.proto",
                "../proto/tea/v1/collection.proto",
                "../proto/tea/v1/discovery.proto",
                "../proto/tea/v1/consumer.proto",
                "../proto/tea/v1/publisher.proto",
                "../proto/tea/v1/insights.proto",
            ],
            &[proto_root, &buf_export_dir],
        )
        .unwrap_or_else(|e| panic!("failed to compile TEA protobuf definitions: {e}"));

    let out_dir = PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR must be set"));
    for generated in ["buf.validate.rs", "google.api.rs"] {
        strip_generated_docs(&out_dir.join(generated));
    }
    normalize_generated_docs(&out_dir.join("tea.v1.rs"));

    println!("cargo:rerun-if-changed=../proto/buf.yaml");
    println!("cargo:rerun-if-changed=../proto/buf.gen.yaml");
    println!("cargo:rerun-if-changed=../proto/buf.lock");
    println!("cargo:rerun-if-changed=../proto/tea");
}

fn normalize_generated_docs(path: &Path) {
    let Ok(contents) = fs::read_to_string(path) else {
        return;
    };

    let mut normalized = String::with_capacity(contents.len());
    let mut in_fence = false;
    let mut changed = false;

    for line in contents.lines() {
        if line.trim_end() == "/// ```" {
            if in_fence {
                normalized.push_str("/// ```\n");
            } else {
                // Generated dependency docs often contain proto or grammar examples,
                // which rustdoc otherwise tries to compile as Rust doctests.
                normalized.push_str("/// ```text\n");
            }
            in_fence = !in_fence;
            changed = true;
        } else {
            normalized.push_str(line);
            normalized.push('\n');
        }
    }

    if changed {
        fs::write(path, normalized).unwrap_or_else(|e| {
            panic!(
                "failed to normalize generated docs for {}: {e}",
                path.display()
            )
        });
    }
}

fn strip_generated_docs(path: &Path) {
    let Ok(contents) = fs::read_to_string(path) else {
        return;
    };

    let mut stripped = String::with_capacity(contents.len());
    let mut changed = false;

    for line in contents.lines() {
        if line.trim_start().starts_with("///") {
            changed = true;
            continue;
        }
        stripped.push_str(line);
        stripped.push('\n');
    }

    if changed {
        fs::write(path, stripped).unwrap_or_else(|e| {
            panic!("failed to strip generated docs for {}: {e}", path.display())
        });
    }
}
