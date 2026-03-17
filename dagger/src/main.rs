use dagger_sdk::{Container, Dagger};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Dagger::new().await?;

    let source = client.host().directory(".").id().await?;

    // Build the TEA server
    let rust = client
        .container()
        .from("rust:1.75")
        .with_directory("/src", source)
        .with_workdir("/src/tea-server");

    let built = rust.exec(["cargo", "build", "--release"]).await?;
    println!("Build completed");

    // Run tests
    let tested = rust.exec(["cargo", "test"]).await?;
    println!("Tests completed");

    // Generate SBOM
    let syft = client
        .container()
        .from("anchore/syft:latest")
        .with_directory("/src", source)
        .with_workdir("/src")
        .exec(["syft", "packages", "tea-server/target/release/tea-server", "-o", "cyclonedx-json", "--file", "sbom.json"])
        .await?;
    println!("SBOM generated");

    // Sign artifact with cosign
    let cosign = client
        .container()
        .from("gcr.io/projectsigstore/cosign:latest")
        .with_directory("/src", source)
        .with_workdir("/src")
        .exec(["cosign", "sign-blob", "--key", "cosign.key", "tea-server/target/release/tea-server"])
        .await?;
    println!("Artifact signed");

    // Export results
    client.host().directory("/src").export(".").await?;

    Ok(())
}
