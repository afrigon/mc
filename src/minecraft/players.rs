use crate::utils::errors::McResult;

/// Accepts every name the server does: legacy accounts predate the current
/// rules, so only what cannot be a name at all is rejected.
pub fn validate_name(name: &str) -> McResult<()> {
    if name.is_empty() {
        anyhow::bail!("a player name cannot be empty");
    }

    if name.chars().any(|c| c.is_whitespace() || c.is_control()) {
        anyhow::bail!(
            "`{}` is not a player name; names contain no spaces or control characters",
            name
        );
    }

    Ok(())
}
