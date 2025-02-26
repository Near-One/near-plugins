use std::process::Output;
use std::{path::Path, str::FromStr};

pub fn require_success(output: Output) -> Result<(), anyhow::Error> {
    if output.status.success() {
        Ok(())
    } else {
        Err(anyhow::Error::msg(format!("Command failed: {:?}", output)))
    }
}

pub async fn compile_project(project_path: &Path, package_name: &str) -> anyhow::Result<Vec<u8>> {
    let path = project_path.join("Cargo.toml");
    let artifact = cargo_near_build::build(cargo_near_build::BuildOpts {
        manifest_path: Some(
            cargo_near_build::camino::Utf8PathBuf::from_str(path.to_str().unwrap())
                .expect("camino PathBuf from str"),
        ),
        no_abi: true, // TODO remove this flag when we fix ABI generation
        ..Default::default()
    })
    .unwrap_or_else(|_| panic!("building contract {package_name} from {project_path:?}"));

    Ok(tokio::fs::read(&artifact.path).await?)
}
