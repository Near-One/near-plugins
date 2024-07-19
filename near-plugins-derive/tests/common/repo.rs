use std::path::{Path, PathBuf};
use std::process::Output;

async fn read_toolchain(project_path: &Path) -> anyhow::Result<String> {
    let content = tokio::fs::read_to_string(project_path.join("rust-toolchain")).await?;
    let value: toml::Value = toml::from_str(&content)?;
    let result = value
        .as_table()
        .and_then(|t| t.get("toolchain"))
        .and_then(|v| v.as_table())
        .and_then(|t| t.get("channel"))
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::Error::msg("Failed to parse rust-toolchain toml"))?
        .to_string();
    Ok(result)
}

pub fn require_success(output: &Output) -> Result<(), anyhow::Error> {
    if output.status.success() {
        Ok(())
    } else {
        Err(anyhow::Error::msg(format!("Command failed: {output:?}")))
    }
}

async fn add_wasm_target(project_path: &Path, toolchain: &str) -> anyhow::Result<()> {
    let output = tokio::process::Command::new("rustup")
        .env("RUSTUP_TOOLCHAIN", toolchain)
        .current_dir(project_path)
        .args(["target", "add", "wasm32-unknown-unknown"])
        .output()
        .await?;
    require_success(&output)?;
    Ok(())
}

pub async fn compile_project(project_path: &Path, package_name: &str) -> anyhow::Result<Vec<u8>> {
    let toolchain = read_toolchain(project_path).await?;
    add_wasm_target(project_path, &toolchain).await?;
    let output = tokio::process::Command::new("cargo")
        .env("RUSTUP_TOOLCHAIN", &toolchain)
        .current_dir(project_path)
        .args([
            "build",
            "--target",
            "wasm32-unknown-unknown",
            "--release",
            "--no-default-features",
            "-p",
            package_name,
        ])
        .output()
        .await?;

    require_success(&output)?;
    let binary_path = project_path.join(
        [
            "target",
            "wasm32-unknown-unknown",
            "release",
            format!("{package_name}.wasm").as_str(),
        ]
        .iter()
        .collect::<PathBuf>(),
    );
    Ok(tokio::fs::read(binary_path).await?)
}
