use std::path::Path;

use dagger_sdk::connect;

const RUST_IMAGE: &str = "rust:1.75";
const SYFT_IMAGE: &str = "anchore/syft:v1.29.0";
const COSIGN_IMAGE: &str = "gcr.io/projectsigstore/cosign:v2.4.1";
const ARTIFACT_DIR: &str = "artifacts/dagger";
const BINARY_NAME: &str = "tea-server";
const COSIGN_KEY_ENV: &str = "COSIGN_PRIVATE_KEY";

#[tokio::main]
async fn main() -> Result<(), dagger_sdk::errors::ConnectError> {
    let workspace = std::env::current_dir().expect("workspace path should resolve");
    let artifact_dir = workspace.join(ARTIFACT_DIR);
    std::fs::create_dir_all(&artifact_dir).expect("artifact directory should be creatable");
    let cosign_key = std::env::var(COSIGN_KEY_ENV).expect("COSIGN_PRIVATE_KEY must be set");

    connect(move |client| {
        let workspace = workspace.clone();
        let artifact_dir = artifact_dir.clone();
        let cosign_key = cosign_key.clone();

        async move {
            let source = client.host().directory(".");
            let cargo_registry = client.cache_volume("tea-server-cargo-registry");
            let cargo_git = client.cache_volume("tea-server-cargo-git");
            let cargo_target = client.cache_volume("tea-server-cargo-target");

            let rust = client
                .container()
                .from(RUST_IMAGE)
                .with_directory("/src", source)
                .with_workdir("/src/tea-server")
                .with_mounted_cache("/usr/local/cargo/registry", cargo_registry)
                .with_mounted_cache("/usr/local/cargo/git", cargo_git)
                .with_mounted_cache("/src/tea-server/target", cargo_target)
                .with_env_variable("CARGO_NET_RETRY", "5")
                .with_env_variable("CARGO_TERM_COLOR", "always");

            let built = rust.with_exec(vec!["cargo", "build", "--release", "--locked"]);
            built.sync().await?;
            println!("Build completed");

            rust.with_exec(vec!["cargo", "test", "--locked"])
                .sync()
                .await?;
            println!("Tests completed");

            let artifact = built.file(format!("/src/tea-server/target/release/{BINARY_NAME}"));
            artifact
                .export(artifact_dir.join(BINARY_NAME).display().to_string())
                .await?;
            println!("Artifact exported");

            let sbom = client
                .container()
                .from(SYFT_IMAGE)
                .with_mounted_file("/tmp/tea-server", artifact.clone())
                .with_exec(vec![
                    "sh",
                    "-lc",
                    "syft packages /tmp/tea-server -o cyclonedx-json",
                ])
                .stdout()
                .await?;
            write_output(&artifact_dir, "sbom.json", &sbom)?;
            println!("SBOM generated");

            let cosign_secret = client.set_secret("cosign-private-key", cosign_key);
            let signature = client
                .container()
                .from(COSIGN_IMAGE)
                .with_mounted_file("/tmp/tea-server", artifact)
                .with_secret_variable("COSIGN_PASSWORD", client.set_secret("cosign-password", ""))
                .with_secret_variable("COSIGN_PRIVATE_KEY", cosign_secret)
                .with_exec(vec![
                    "sh",
                    "-lc",
                    "printf '%s' \"$COSIGN_PRIVATE_KEY\" > /tmp/cosign.key && cosign sign-blob --key /tmp/cosign.key /tmp/tea-server",
                ])
                .stdout()
                .await?;
            write_output(&artifact_dir, "tea-server.sig", &signature)?;
            println!("Artifact signed");

            write_output(
                &artifact_dir,
                "build-metadata.txt",
                &format!(
                    "binary={BINARY_NAME}\nrust_image={RUST_IMAGE}\nsyft_image={SYFT_IMAGE}\ncosign_image={COSIGN_IMAGE}\nworkspace={}\n",
                    workspace.display()
                ),
            )?;

            Ok(())
        }
    })
    .await
}

fn write_output(dir: &Path, file_name: &str, contents: &str) -> eyre::Result<()> {
    let path = dir.join(file_name);
    std::fs::write(path, contents)?;
    Ok(())
}
