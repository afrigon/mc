use url::Url;

use crate::crypto::checksum::ChecksumRef;

pub struct ArtifactSource {
    pub url: Url,
    pub checksum: ChecksumRef
}
