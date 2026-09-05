use std::net::IpAddr;
use std::net::Ipv4Addr;

use anyhow::Context;
use reqwest::header::AUTHORIZATION;
use serde::Deserialize;
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Value;

use crate::crypto::checksum::ChecksumRef;
use crate::crypto::checksum::LocalChecksum;
use crate::env::Architecture;
use crate::env::Platform;
use crate::network::artifact::ArtifactKind;
use crate::network::artifact::ArtifactSource;
use crate::services::github_api;
use crate::services::github_api::GithubApiRelease;
use crate::services::tunnel_provider::TunnelProvider;
use crate::utils::errors::McResult;

const API_URL: &'static str = "https://api.playit.gg";
const RELEASES_OWNER: &'static str = "playit-cloud";
const RELEASES_REPOSITORY: &'static str = "playit-agent";

pub const CLAIM_URL: &'static str = "https://playit.gg/claim";
pub const DASHBOARD_TUNNELS_URL: &'static str = "https://playit.gg/account/tunnels";
pub const MINECRAFT_JAVA_TUNNEL_TYPE: &'static str = "minecraft-java";

pub struct PlayitApi;

fn release_version(release: &GithubApiRelease) -> Option<String> {
    if release.prerelease || release.draft {
        return None;
    }

    let version = release.tag_name.strip_prefix('v')?;
    let is_number =
        |part: &str| !part.is_empty() && part.chars().all(|character| character.is_ascii_digit());
    let parts: Vec<&str> = version.split('.').collect();

    if parts.len() == 3 && parts.iter().all(|part| is_number(part)) {
        Some(version.to_owned())
    } else {
        None
    }
}

fn asset_name(platform: Platform, architecture: Architecture) -> Option<&'static str> {
    match (platform, architecture) {
        (Platform::Linux, Architecture::x86_64) => Some("playit-linux-amd64"),
        (Platform::Linux, Architecture::aarch64) => Some("playit-linux-aarch64"),
        (Platform::Windows, Architecture::x86_64) => Some("playit-windows-x86_64.exe"),
        _ => None
    }
}

fn asset_checksum(digest: &str) -> McResult<ChecksumRef> {
    let hex_digest = digest
        .strip_prefix("sha256:")
        .with_context(|| format!("unsupported release asset digest `{}`", digest))?;
    let mut bytes = [0u8; 32];

    hex::decode_to_slice(hex_digest, &mut bytes).context("invalid release asset digest")?;

    Ok(ChecksumRef::Local(LocalChecksum::sha256(bytes)))
}

impl TunnelProvider for PlayitApi {
    async fn versions(client: &reqwest::Client) -> McResult<Vec<String>> {
        let releases =
            github_api::get_releases(client, RELEASES_OWNER, RELEASES_REPOSITORY).await?;

        Ok(releases.iter().filter_map(release_version).collect())
    }

    async fn agent_source(
        client: &reqwest::Client,
        version: &str,
        platform: Platform,
        architecture: Architecture
    ) -> McResult<ArtifactSource> {
        let name = asset_name(platform, architecture).with_context(|| {
            format!(
                "playit does not publish a build for {} {} on GitHub",
                platform, architecture
            )
        })?;

        let release = github_api::get_release(
            client,
            RELEASES_OWNER,
            RELEASES_REPOSITORY,
            &format!("v{}", version)
        )
        .await?;

        let asset = release
            .assets
            .into_iter()
            .find(|asset| asset.name == name)
            .with_context(|| format!("playit {} has no `{}` asset", version, name))?;

        let checksum = asset.digest.as_deref().map(asset_checksum).transpose()?;

        Ok(ArtifactSource {
            url: asset.browser_download_url,
            kind: ArtifactKind::Binary,
            checksum
        })
    }

    fn agent_binary_name(platform: Platform) -> &'static str {
        match platform {
            Platform::Windows => "playit.exe",
            _ => "playit"
        }
    }
}

#[derive(Deserialize)]
#[serde(tag = "status", content = "data", rename_all = "lowercase")]
enum PlayitApiResponse<T> {
    Success(T),
    Fail(Value),
    Error(Value)
}

impl<T> PlayitApiResponse<T> {
    fn into_result(self, path: &str) -> McResult<T> {
        match self {
            PlayitApiResponse::Success(data) => Ok(data),
            PlayitApiResponse::Fail(details) => {
                anyhow::bail!("playit API {} failed: {}", path, details)
            }
            PlayitApiResponse::Error(details) => {
                anyhow::bail!("playit API {} errored: {}", path, details)
            }
        }
    }
}

// The API answers with its envelope on every status code, so the body is
// decoded regardless of the HTTP status.
async fn call<Request: Serialize, Response: DeserializeOwned>(
    client: &reqwest::Client,
    path: &str,
    secret: Option<&str>,
    request: &Request
) -> McResult<PlayitApiResponse<Response>> {
    let mut builder = client.post(format!("{}{}", API_URL, path)).json(request);

    if let Some(secret) = secret {
        builder = builder.header(AUTHORIZATION, format!("Agent-Key {}", secret));
    }

    let response = builder
        .send()
        .await
        .context("could not send HTTP request")?;
    let status = response.status();
    let body = response
        .text()
        .await
        .context("could not read the playit API response")?;

    serde_json::from_str(&body)
        .with_context(|| format!("unexpected playit API response from {} ({})", path, status))
}

#[derive(Deserialize)]
pub struct PlayitApiRunData {
    pub agent_id: String,
    pub tunnels: Vec<PlayitApiTunnel>,
    pub pending: Vec<PlayitApiPendingTunnel>
}

#[derive(Deserialize)]
pub struct PlayitApiTunnel {
    pub tunnel_type: Option<String>,
    pub local_port: u16,
    pub assigned_domain: String,
    pub custom_domain: Option<String>
}

impl PlayitApiTunnel {
    pub fn address(&self) -> String {
        self.custom_domain
            .clone()
            .unwrap_or_else(|| self.assigned_domain.clone())
    }
}

#[derive(Deserialize)]
pub struct PlayitApiPendingTunnel {
    pub tunnel_type: Option<String>
}

pub async fn get_run_data(client: &reqwest::Client, secret: &str) -> McResult<PlayitApiRunData> {
    #[derive(Serialize)]
    struct Request {}

    call(client, "/agents/rundata", Some(secret), &Request {})
        .await?
        .into_result("/agents/rundata")
}

#[derive(Serialize)]
pub struct PlayitApiCreateTunnel {
    pub name: Option<String>,
    pub tunnel_type: Option<&'static str>,
    pub port_type: &'static str,
    pub port_count: u16,
    pub origin: PlayitApiTunnelOrigin,
    pub enabled: bool,
    pub alloc: Option<()>,
    pub firewall_id: Option<String>,
    pub proxy_protocol: Option<()>
}

impl PlayitApiCreateTunnel {
    pub fn minecraft_java(
        name: String,
        agent_id: String,
        local_port: u16
    ) -> PlayitApiCreateTunnel {
        PlayitApiCreateTunnel {
            name: Some(name),
            tunnel_type: Some(MINECRAFT_JAVA_TUNNEL_TYPE),
            port_type: "tcp",
            port_count: 1,
            origin: PlayitApiTunnelOrigin::Agent {
                agent_id,
                local_ip: IpAddr::V4(Ipv4Addr::LOCALHOST),
                local_port: Some(local_port)
            },
            enabled: true,
            alloc: None,
            firewall_id: None,
            proxy_protocol: None
        }
    }
}

#[derive(Serialize)]
#[serde(tag = "type", content = "details", rename_all = "lowercase")]
pub enum PlayitApiTunnelOrigin {
    Agent {
        agent_id: String,
        local_ip: IpAddr,
        local_port: Option<u16>
    }
}

pub async fn create_tunnel(
    client: &reqwest::Client,
    secret: &str,
    request: &PlayitApiCreateTunnel
) -> McResult<()> {
    call::<_, Value>(client, "/tunnels/create", Some(secret), request)
        .await?
        .into_result("/tunnels/create")
        .map(|_| ())
}

#[derive(Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum PlayitApiClaimStatus {
    WaitingForUserVisit,
    WaitingForUser,
    UserAccepted,
    UserRejected
}

pub async fn claim_setup(client: &reqwest::Client, code: &str) -> McResult<PlayitApiClaimStatus> {
    #[derive(Serialize)]
    struct Request<'a> {
        code: &'a str,
        agent_type: &'static str,
        version: String
    }

    let request = Request {
        code,
        agent_type: "self-managed",
        version: format!("mc {}", env!("CARGO_PKG_VERSION"))
    };

    call(client, "/claim/setup", None, &request)
        .await?
        .into_result("/claim/setup")
}

/// Returns the agent secret once the claim has been approved, `None` while
/// playit is still waiting on the user.
pub async fn claim_exchange(client: &reqwest::Client, code: &str) -> McResult<Option<String>> {
    #[derive(Serialize)]
    struct Request<'a> {
        code: &'a str
    }

    #[derive(Deserialize)]
    struct Response {
        secret_key: String
    }

    let response: PlayitApiResponse<Response> =
        call(client, "/claim/exchange", None, &Request { code }).await?;

    match response {
        PlayitApiResponse::Success(data) => Ok(Some(data.secret_key)),
        PlayitApiResponse::Fail(details) if details == "NotAccepted" || details == "NotSetup" => {
            Ok(None)
        }
        other => other.into_result("/claim/exchange").map(|_| None)
    }
}
