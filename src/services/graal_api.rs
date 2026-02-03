use url::Url;

use crate::crypto::checksum::ChecksumRef;
use crate::crypto::checksum::RemoteChecksum;
use crate::env::Architecture;
use crate::env::Platform;
use crate::java::JavaVersion;
use crate::network::artifact::ArtifactKind;
use crate::network::artifact::ArtifactSource;
use crate::services::java_provider::JavaProvider;

pub struct GraalApi;

impl JavaProvider for GraalApi {
    fn jdk_source(
        java_version: JavaVersion,
        platform: Platform,
        architecture: Architecture
    ) -> ArtifactSource {
        let version = java_version.value();

        let extension = match platform {
            Platform::Windows => "zip",
            _ => "tar.gz"
        };

        let url = Url::parse(&format!(
            "https://download.oracle.com/graalvm/{}/latest/graalvm-jdk-{}_{}-{}_bin.{}",
            version, version, platform, architecture, extension
        ))
        .unwrap();

        let kind = match platform {
            Platform::Windows => ArtifactKind::Zip,
            _ => ArtifactKind::TarGz
        };

        let checksum_url = Url::parse(&format!("{}.sha256", url.as_str())).unwrap();
        let checksum = Some(ChecksumRef::Remote(RemoteChecksum::sha256(checksum_url)));

        ArtifactSource {
            url,
            kind,
            checksum
        }
    }
}
