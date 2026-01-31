use std::fs::File;
use std::io::Read;
use std::path::Path;

use crate::utils::errors::McResult;

pub async fn deflate<R: Read>(reader: R, output: &Path) -> McResult<()> {
    Ok(())
}

pub async fn deflate_tar() -> McResult<()> {
    Ok(())
}

pub async fn deflate_zip(file: File, output: &Path) -> McResult<()> {
    let mut archive = zip::ZipArchive::new(file)?;

    archive.extract_unwrapped_root_dir(output, zip::read::root_dir_common_filter)?;

    Ok(())
}
