use std::path::PathBuf;

use crate::context::McContext;
use crate::env::Architecture;
use crate::env::Platform;
use crate::java::JavaDescriptor;
use crate::java::JavaVendor;
use crate::java::JavaVersion;
use crate::network;
use crate::services::corretto_api;
use crate::services::graal_api;
use crate::services::java_provider::JavaProvider;
use crate::utils::CaseIterable;
use crate::utils::errors::McResult;

pub struct JavaInstallOptions {
    pub version: JavaDescriptor,
    pub platform: Platform,
    pub architecture: Architecture,
    pub java_directory: PathBuf
}

pub async fn install(context: &mut McContext, options: &JavaInstallOptions) -> McResult<()> {
    let name = options.version.to_string();
    let path = options.java_directory.join(&name);

    if path.exists() {
        anyhow::bail!("{} is already installed", name);
    }

    // TODO: add progress bar

    _ = context.shell().status("Installing", name);

    tokio::fs::create_dir_all(&path).await?;

    let source = match options.version.product {
        JavaVendor::correto => corretto_api::CorrettoApi::jdk_source(
            options.version.version,
            options.platform,
            options.architecture
        ),
        JavaVendor::graal => graal_api::GraalApi::jdk_source(
            options.version.version,
            options.platform,
            options.architecture
        )
    };

    network::stream_artifact(&context.http_client, source, &path).await
}

pub struct JavaListOptions {}

pub async fn list(context: &mut McContext, options: &JavaListOptions) -> McResult<()> {
    for descriptor in JavaDescriptor::all_cases() {
        let mut shell = context.shell();
        let stdout = shell.out();

        _ = write!(stdout, "{}", *descriptor);

        if descriptor.product == JavaVendor::graal && descriptor.version == JavaVersion::Java25 {
            _ = write!(stdout, " (recommended)");
        }

        _ = write!(stdout, "\n");
    }

    Ok(())
}
