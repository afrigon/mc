use url::Url;

use crate::crypto::checksum::ChecksumAlgorithm;
use crate::crypto::checksum::ChecksumRef;
use crate::crypto::checksum::LocalChecksum;
use crate::utils::errors::McResult;

pub enum ArtifactKind {
    Jar,
    Zip,
    TarGz
}

pub struct ArtifactSource {
    pub url: Url,
    pub kind: ArtifactKind,
    pub checksum: Option<ChecksumRef>
}

impl ArtifactSource {
    pub async fn checksum(&self, client: &reqwest::Client) -> McResult<Option<LocalChecksum>> {
        if let Some(ref checksum) = self.checksum {
            match checksum {
                ChecksumRef::Local(local) => Ok(Some(local.clone())),
                ChecksumRef::Remote(remote) => {
                    let data = client
                        .get(remote.url.clone())
                        .send()
                        .await?
                        .error_for_status()?
                        .text()
                        .await?;

                    match remote.algorithm {
                        ChecksumAlgorithm::md5 => {
                            let mut digest = [0u8; 16];
                            hex::decode_to_slice(data, &mut digest)?;
                            Ok(Some(LocalChecksum::md5(digest)))
                        }
                        ChecksumAlgorithm::sha1 => {
                            let mut digest = [0u8; 20];
                            hex::decode_to_slice(data, &mut digest)?;
                            Ok(Some(LocalChecksum::sha1(digest)))
                        }
                        ChecksumAlgorithm::sha256 => {
                            let mut digest = [0u8; 32];
                            hex::decode_to_slice(data, &mut digest)?;
                            Ok(Some(LocalChecksum::sha256(digest)))
                        }
                    }
                }
            }
        } else {
            Ok(None)
        }
    }
}
