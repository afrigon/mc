use crate::env::Architecture;
use crate::env::Platform;
use crate::network::artifact::ArtifactSource;
use crate::utils::errors::McResult;

pub trait TunnelProvider {
    async fn versions(client: &reqwest::Client) -> McResult<Vec<String>>;

    async fn agent_source(
        client: &reqwest::Client,
        version: &str,
        platform: Platform,
        architecture: Architecture
    ) -> McResult<ArtifactSource>;

    fn agent_binary_name(platform: Platform) -> &'static str;
}
