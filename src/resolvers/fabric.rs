use crate::context::McContext;
use crate::mods::loader::LoaderKind;
use crate::services;
use crate::utils::errors::McResult;
use crate::utils::product_descriptor::VersionResolver;

pub struct FabricVersionResolver;

impl VersionResolver<LoaderKind> for FabricVersionResolver {
    async fn resolve(context: &McContext, version: Option<String>) -> McResult<String> {
        let version = version.unwrap_or_else(|| "latest".to_owned());

        let versions = services::fabric_api::get_versions(&context.http_client).await?;

        if versions.is_empty() {
            anyhow::bail!("failed to fetch fabric versions")
        }

        match version.as_str() {
            "latest" => {
                if let Some(first) = versions.first() {
                    Ok(first.version.to_owned())
                } else {
                    anyhow::bail!("failed to fetch fabric versions");
                }
            }
            v => {
                if versions.iter().any(|item| item.version == v) {
                    Ok(v.to_owned())
                } else {
                    anyhow::bail!("unknown fabric version {}", v)
                }
            }
        }
    }
}
