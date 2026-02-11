use std::path::PathBuf;

use crate::context::McContext;
use crate::minecraft::eula::MinecraftEula;
use crate::utils::errors::McResult;

pub struct EulaApplyOptions {
    pub accept: bool,
    pub instance_path: PathBuf
}

pub async fn apply(context: &mut McContext, options: &EulaApplyOptions) -> McResult<()> {
    let eula_path = options.instance_path.join("eula.txt");
    let eula = MinecraftEula {
        eula: options.accept
    };

    tokio::fs::write(eula_path, eula.to_string()?).await?;

    Ok(())
}
