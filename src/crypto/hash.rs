use std::io;
use std::pin;
use std::task;

use digest::Digest;
use digest::DynDigest;
use md5::Md5;
use sha1::Sha1;
use sha2::Sha256;
use tokio::io::AsyncRead;
use tokio::io::ReadBuf;

use crate::crypto::checksum::ChecksumAlgorithm;

pub struct Hasher<R> {
    reader: R,
    digest: Box<dyn DynDigest>
}

impl<R> Hasher<R> {
    pub fn new(reader: R, checksum: ChecksumAlgorithm) -> Self {
        let digest: Box<dyn DynDigest> = match checksum {
            ChecksumAlgorithm::md5 => Box::new(Md5::new()),
            ChecksumAlgorithm::sha1 => Box::new(Sha1::new()),
            ChecksumAlgorithm::sha256 => Box::new(Sha256::new())
        };

        Hasher { reader, digest }
    }

    pub fn hash(&self) -> Box<[u8]> {
        self.digest.box_clone().finalize()
    }
}

impl<R: io::Read> io::Read for Hasher<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let n = self.reader.read(buf)?;

        if n != 0 {
            self.digest.update(&buf[..n]);
        }

        Ok(n)
    }
}

impl<R: AsyncRead + Unpin> AsyncRead for Hasher<R> {
    fn poll_read(
        mut self: pin::Pin<&mut Self>,
        cx: &mut task::Context<'_>,
        buf: &mut ReadBuf<'_>
    ) -> task::Poll<io::Result<()>> {
        let start = buf.filled().len();
        let poll = pin::Pin::new(&mut self.reader).poll_read(cx, buf);

        if let task::Poll::Ready(Ok(())) = &poll {
            let end = buf.filled().len();

            if end > start {
                self.digest.update(&buf.filled()[start..end]);
            }
        }

        poll
    }
}
