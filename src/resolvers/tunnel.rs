use anyhow::Context;

use crate::context::McContext;
use crate::services::playit_api::PlayitApi;
use crate::services::tunnel_provider::TunnelProvider;
use crate::tunnel::TunnelProviderKind;
use crate::utils::errors::McResult;
use crate::utils::product_descriptor::VersionResolver;

pub struct TunnelVersionResolver;

impl VersionResolver<TunnelProviderKind, String> for TunnelVersionResolver {
    async fn resolve(context: &McContext, version: Option<&str>) -> McResult<String> {
        let versions = PlayitApi::versions(&context.http_client).await?;

        match version.unwrap_or("latest") {
            "latest" => versions
                .into_iter()
                .next()
                .context("no playit release found on GitHub"),
            v => {
                let v = v.strip_prefix('v').unwrap_or(v);

                if versions.iter().any(|item| item == v) {
                    Ok(v.to_owned())
                } else {
                    anyhow::bail!("unknown playit version {}", v)
                }
            }
        }
    }
}
