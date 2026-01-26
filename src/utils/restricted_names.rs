type Result<T> = std::result::Result<T, NameValidationError>;

/// Error validating names in mc.
#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub struct NameValidationError(#[from] ErrorKind);

/// Non-public error kind for [`NameValidationError`].
#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
enum ErrorKind {
    #[error("{0} cannot be empty")]
    Empty(&'static str),

    #[error("invalid character `{ch}` in {what}: `{name}`, {reason}")]
    InvalidCharacter {
        ch: char,
        what: &'static str,
        name: String,
        reason: &'static str,
    },
}

pub fn validate_server_name(name: &str) -> Result<()> {
    validate_name(name, "server name")
}

fn validate_name(name: &str, what: &'static str) -> Result<()> {
    if name.is_empty() {
        return Err(ErrorKind::Empty(what).into());
    }

    let mut chars = name.chars();

    if let Some(ch) = chars.next() {
        if ch.is_digit(10) {
            // A specific error for a potentially common case.
            return Err(ErrorKind::InvalidCharacter {
                ch,
                what,
                name: name.into(),
                reason: "the name cannot start with a digit",
            }
            .into());
        }

        if !(unicode_ident::is_xid_start(ch) || ch == '_') {
            return Err(ErrorKind::InvalidCharacter {
                ch,
                what,
                name: name.into(),
                reason: "the first character must be a Unicode XID start character \
                 (most letters or `_`)",
            }
            .into());
        }
    }

    for ch in chars {
        if !(unicode_ident::is_xid_continue(ch) || ch == '-') {
            return Err(ErrorKind::InvalidCharacter {
                ch,
                what,
                name: name.into(),
                reason: "characters must be Unicode XID characters \
                 (numbers, `-`, `_`, or most letters)",
            }
            .into());
        }
    }

    Ok(())
}
