use anstream::{AutoStream, ColorChoice};
use anstyle::Style;
use std::fmt;
use std::io::Write;

use crate::cli::styles::{ERROR, HEADER, WARN};
use crate::utils::verbosity::Verbosity;

pub struct Shell {
    stdout: AutoStream<std::io::Stdout>,
    stderr: AutoStream<std::io::Stderr>,
    verbosity: Verbosity,
    color_choice: ColorChoice,
}

impl Shell {
    pub fn new() -> Shell {
        let color_choice = ColorChoice::Auto;

        Shell {
            stdout: AutoStream::new(std::io::stdout(), color_choice),
            stderr: AutoStream::new(std::io::stderr(), color_choice),
            verbosity: Verbosity::Regular,
            color_choice,
        }
    }

    pub fn out(&mut self) -> &mut dyn Write {
        &mut self.stdout
    }

    pub fn err(&mut self) -> &mut dyn Write {
        &mut self.stderr
    }

    pub fn set_verbosity(&mut self, verbosity: Verbosity) {
        self.verbosity = verbosity
    }

    pub fn set_color_choice(&mut self, color_choice: clap::ColorChoice) {
        self.color_choice = match color_choice {
            clap::ColorChoice::Auto => ColorChoice::Auto,
            clap::ColorChoice::Always => ColorChoice::Always,
            clap::ColorChoice::Never => ColorChoice::Never,
        };

        self.stdout = AutoStream::new(std::io::stdout(), self.color_choice);
        self.stderr = AutoStream::new(std::io::stderr(), self.color_choice);
    }

    fn output_stderr(
        &mut self,
        status: &dyn fmt::Display,
        message: Option<&dyn fmt::Display>,
        style: &Style,
        justified: bool,
    ) -> anyhow::Result<()> {
        let mut buffer = Vec::new();

        if justified {
            write!(&mut buffer, "{style}{status:>12}{style:#}")?;
        } else {
            write!(&mut buffer, "{style}{status}{style:#}:")?;
        }

        match message {
            Some(message) => writeln!(buffer, " {message}")?,
            None => write!(buffer, " ")?,
        }

        self.stderr.write_all(&buffer)?;

        Ok(())
    }

    pub fn print(
        &mut self,
        status: &dyn fmt::Display,
        message: Option<&dyn fmt::Display>,
        style: &Style,
        justified: bool,
    ) -> anyhow::Result<()> {
        match self.verbosity {
            Verbosity::Quiet => Ok(()),
            _ => self.output_stderr(status, message, style, justified),
        }
    }

    pub fn status<S, M>(&mut self, status: S, message: M) -> anyhow::Result<()>
    where
        S: fmt::Display,
        M: fmt::Display,
    {
        self.print(&status, Some(&message), &HEADER, true)
    }

    pub fn error<M>(&mut self, message: M) -> anyhow::Result<()>
    where
        M: fmt::Display,
    {
        self.output_stderr(&"error", Some(&message), &ERROR, false)
    }

    pub fn warn<M>(&mut self, message: M) -> anyhow::Result<()>
    where
        M: fmt::Display,
    {
        self.output_stderr(&"warn", Some(&message), &WARN, false)
    }
}
