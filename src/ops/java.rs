use std::path::PathBuf;

use crate::context::McContext;
use crate::env::Architecture;
use crate::env::Platform;
use crate::java::JavaDescriptor;
use crate::java::JavaVendor;
use crate::java::JavaVersion;
use crate::utils::CaseIterable;
use crate::utils::errors::McResult;

pub struct JavaInstallOptions {
    pub version: JavaDescriptor,
    pub platform: Platform,
    pub architecture: Architecture,
    pub java_directory: PathBuf
}

pub async fn install(context: &mut McContext, options: &JavaInstallOptions) -> McResult<()> {
    // TODO: check if already installed
    // TODO: add confirm, override, etc dialogs
    // TODO: add progress bar
    // JavaService::download_version(
    //     &context.http_client,
    //     version,
    //     platform.unwrap_or(Platform::current()),
    //     architecture.unwrap_or(Architecture::current())
    // )
    // .await?;
    Ok(())
}

pub struct JavaListOptions {}

pub async fn list(context: &mut McContext, options: &JavaListOptions) -> McResult<()> {
    for descriptor in JavaDescriptor::all_cases() {
        let mut shell = context.shell();
        let stdout = shell.out();

        write!(stdout, "{}", *descriptor);

        if descriptor.product == JavaVendor::graal && descriptor.version == JavaVersion::Java25 {
            write!(stdout, " (recommended)");
        }

        write!(stdout, "\n");
    }

    Ok(())
}
