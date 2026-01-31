use std::io;

use digest::DynDigest;

struct Hasher<R, D> {
    reader: R,
    digest: Box<D>
}

impl<R: io::Read, D: DynDigest> Hasher<R, D> {
    pub fn hash(self) {
        self.digest.finalize();
    }
}

impl<R: io::Read, D: DynDigest> io::Read for Hasher<R, D> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let n = self.reader.read(buf)?;

        if n != 0 {
            self.digest.update(&buf[..n]);
        }

        Ok(n)
    }
}
