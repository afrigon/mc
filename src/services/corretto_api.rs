use url::Url;

use crate::crypto::checksum::ChecksumRef;
use crate::crypto::checksum::RemoteChecksum;
use crate::env::Architecture;
use crate::env::Platform;
use crate::java::JavaVersion;
use crate::network::artifact::ArtifactSource;
use crate::services::java_provider::JavaProvider;

pub struct CorrettoApi {}

impl JavaProvider for CorrettoApi {
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
            "https://corretto.aws/downloads/latest/amazon-corretto-{}-{}-{}-jdk.{}",
            version, architecture, platform, extension
        ))
        .unwrap();

        let checksum_url = Url::parse(&format!(
            "https://corretto.aws/downloads/latest_sha256/amazon-corretto-{}-{}-{}-jdk.{}",
            version, architecture, platform, extension
        ))
        .unwrap();

        let checksum = ChecksumRef::Remote(RemoteChecksum::sha256(checksum_url));

        ArtifactSource { url, checksum }
    }
}
