use url::Url;

#[allow(non_camel_case_types)]
pub enum ChecksumAlgorithm {
    md5,
    sha1,
    sha256
}

pub enum ChecksumRef {
    Remote(RemoteChecksum),
    Local(LocalChecksum)
}

#[allow(non_camel_case_types)]
#[derive(Clone, PartialEq, Eq)]
pub enum LocalChecksum {
    md5([u8; 16]),
    sha1([u8; 20]),
    sha256([u8; 32])
}

impl LocalChecksum {
    pub fn algorithm(&self) -> ChecksumAlgorithm {
        match self {
            LocalChecksum::md5(_) => ChecksumAlgorithm::md5,
            LocalChecksum::sha1(_) => ChecksumAlgorithm::sha1,
            LocalChecksum::sha256(_) => ChecksumAlgorithm::sha256
        }
    }

    pub fn hash(&self) -> &[u8] {
        match self {
            LocalChecksum::md5(data) => data,
            LocalChecksum::sha1(data) => data,
            LocalChecksum::sha256(data) => data
        }
    }
}

pub struct RemoteChecksum {
    pub url: Url,
    pub algorithm: ChecksumAlgorithm
}

impl RemoteChecksum {
    pub fn md5(url: Url) -> RemoteChecksum {
        RemoteChecksum {
            url,
            algorithm: ChecksumAlgorithm::md5
        }
    }

    pub fn sha1(url: Url) -> RemoteChecksum {
        RemoteChecksum {
            url,
            algorithm: ChecksumAlgorithm::sha1
        }
    }

    pub fn sha256(url: Url) -> RemoteChecksum {
        RemoteChecksum {
            url,
            algorithm: ChecksumAlgorithm::sha256
        }
    }
}
