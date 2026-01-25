use url::Url;

pub enum ChecksumRef {
    Remote(RemoteChecksum),
    Local(LocalChecksum)
}

#[allow(non_camel_case_types)]
pub enum LocalChecksum {
    md5([u8; 16]),
    sha1([u8; 20]),
    sha256([u8; 32])
}

#[allow(non_camel_case_types)]
pub enum RemoteChecksum {
    md5(Url),
    sha1(Url),
    sha256(Url)
}
