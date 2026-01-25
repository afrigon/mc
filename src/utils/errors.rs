use std::io;

pub type McResult<T> = anyhow::Result<T>;
pub type CliResult = Result<(), CliError>;

#[derive(Debug)]
pub struct CliError {
    pub error: Option<anyhow::Error>,
    pub exit_code: i32,
}

impl CliError {
    pub fn new(error: anyhow::Error, code: i32) -> CliError {
        CliError {
            error: Some(error),
            exit_code: code,
        }
    }

    pub fn code(code: i32) -> CliError {
        CliError {
            error: None,
            exit_code: code,
        }
    }
}

impl From<anyhow::Error> for CliError {
    fn from(value: anyhow::Error) -> Self {
        CliError::new(value, 101)
    }
}

impl From<clap::Error> for CliError {
    fn from(value: clap::Error) -> Self {
        let code = if value.use_stderr() { 1 } else { 0 };
        CliError::new(value.into(), code)
    }
}

impl From<io::Error> for CliError {
    fn from(value: io::Error) -> Self {
        CliError::new(value.into(), 1)
    }
}
