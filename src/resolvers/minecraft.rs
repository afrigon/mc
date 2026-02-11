use crate::context::McContext;
use crate::services;
use crate::utils::errors::McResult;
use crate::utils::product_descriptor::VersionResolver;

pub struct MinecraftVersionResolver;

impl VersionResolver for MinecraftVersionResolver {
    async fn resolve(context: &McContext, version: Option<&str>) -> McResult<String> {
        let version = version.unwrap_or("latest");

        let manifest = services::minecraft_api::get_manifest(&context.http_client).await?;

        match version {
            "latest" => Ok(manifest.latest.release),
            "latest-snapshot" => Ok(manifest.latest.snapshot.to_owned()),
            v => {
                if manifest.versions.iter().any(|item| item.id == v) {
                    Ok(v.to_owned())
                } else {
                    anyhow::bail!("unknown Minecraft version {}", v)
                }
            }
        }
    }
}
