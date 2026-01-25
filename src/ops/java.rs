use crate::{
    cli::{
        commands::java::{JavaInstallCommand, JavaListCommand},
        context::CliContext,
    },
    java::{JavaDistribution, JavaVendor, JavaVersion},
    utils::{CaseIterable, errors::McResult},
};

pub async fn install(context: &mut CliContext, command: &JavaInstallCommand) -> McResult<()> {
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

pub async fn list(context: &mut CliContext, command: &JavaListCommand) -> McResult<()> {
    for distribution in JavaDistribution::all_cases() {
        print!("{}", distribution);

        if distribution.vendor == JavaVendor::graal && distribution.version == JavaVersion::Java25 {
            print!(" (recommended)")
        }

        print!("\n");
    }

    Ok(())
}
