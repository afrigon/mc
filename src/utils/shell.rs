use std::fmt;
use std::io::Write;

use annotate_snippets::Renderer;
use annotate_snippets::Report;
use annotate_snippets::renderer::DecorStyle;
use anstream::AutoStream;
use anstream::ColorChoice;
use anstyle::Style;

use crate::cli::styles::ERROR;
use crate::cli::styles::HEADER;
use crate::cli::styles::WARN;
use crate::utils::errors::McResult;
use crate::utils::verbosity::Verbosity;

pub struct Shell {
    stdout: AutoStream<std::io::Stdout>,
    stderr: AutoStream<std::io::Stderr>,
    verbosity: Verbosity,
    color_choice: ColorChoice
}

impl Shell {
    pub fn new() -> Shell {
        let color_choice = ColorChoice::Auto;

        Shell {
            stdout: AutoStream::new(std::io::stdout(), color_choice),
            stderr: AutoStream::new(std::io::stderr(), color_choice),
            verbosity: Verbosity::Regular,
            color_choice
        }
    }

    pub fn out(&mut self) -> &mut dyn Write {
        &mut self.stdout
    }

    pub fn err(&mut self) -> &mut dyn Write {
        &mut self.stderr
    }

    pub fn err_width(&self) -> Option<usize> {
        platform_shell::stderr_width()
    }

    pub fn set_verbosity(&mut self, verbosity: Verbosity) {
        self.verbosity = verbosity
    }

    pub fn set_color_choice(&mut self, color_choice: clap::ColorChoice) {
        self.color_choice = match color_choice {
            clap::ColorChoice::Auto => ColorChoice::Auto,
            clap::ColorChoice::Always => ColorChoice::Always,
            clap::ColorChoice::Never => ColorChoice::Never
        };

        self.stdout = AutoStream::new(std::io::stdout(), self.color_choice);
        self.stderr = AutoStream::new(std::io::stderr(), self.color_choice);
    }

    fn output_stderr(
        &mut self,
        status: &dyn fmt::Display,
        message: Option<&dyn fmt::Display>,
        style: &Style,
        justified: bool
    ) -> anyhow::Result<()> {
        let mut buffer = Vec::new();

        if justified {
            write!(&mut buffer, "{style}{status:>12}{style:#}")?;
        } else {
            write!(&mut buffer, "{style}{status}{style:#}:")?;
        }

        match message {
            Some(message) => writeln!(buffer, " {message}")?,
            None => write!(buffer, " ")?
        }

        self.stderr.write_all(&buffer)?;

        Ok(())
    }

    pub fn print(
        &mut self,
        status: &dyn fmt::Display,
        message: Option<&dyn fmt::Display>,
        style: &Style,
        justified: bool
    ) -> anyhow::Result<()> {
        match self.verbosity {
            Verbosity::Quiet => Ok(()),
            _ => self.output_stderr(status, message, style, justified)
        }
    }

    pub fn status<S, M>(&mut self, status: S, message: M) -> anyhow::Result<()>
    where
        S: fmt::Display,
        M: fmt::Display
    {
        self.print(&status, Some(&message), &HEADER, true)
    }

    pub fn error<M>(&mut self, message: M) -> anyhow::Result<()>
    where
        M: fmt::Display
    {
        self.output_stderr(&"error", Some(&message), &ERROR, false)
    }

    pub fn warn<M>(&mut self, message: M) -> anyhow::Result<()>
    where
        M: fmt::Display
    {
        self.output_stderr(&"warning", Some(&message), &WARN, false)
    }

    pub fn note<M>(&mut self, message: M) -> anyhow::Result<()>
    where
        M: fmt::Display
    {
        let report = &[annotate_snippets::Group::with_title(
            annotate_snippets::Level::NOTE.secondary_title(message.to_string())
        )];

        self.print_report(report, false)
    }

    pub fn print_report(&mut self, report: Report<'_>, force: bool) -> McResult<()> {
        if !force && self.verbosity == Verbosity::Quiet {
            return Ok(());
        }

        let term_width = self
            .err_width()
            .unwrap_or(annotate_snippets::renderer::DEFAULT_TERM_WIDTH);

        let rendered = Renderer::styled()
            .term_width(term_width)
            .decor_style(DecorStyle::Ascii)
            .render(report);

        self.stderr.write_all(rendered.as_bytes())?;
        self.stderr.write_all(b"\n")?;

        Ok(())
    }
}

#[cfg(unix)]
mod platform_shell {
    use std::mem;

    pub fn stderr_width() -> Option<usize> {
        unsafe {
            let mut winsize: libc::winsize = mem::zeroed();

            // The .into() here is needed for FreeBSD which defines TIOCGWINSZ
            // as c_uint but ioctl wants c_ulong.
            if libc::ioctl(libc::STDERR_FILENO, libc::TIOCGWINSZ.into(), &mut winsize) < 0 {
                return None;
            }

            if winsize.ws_col > 0 {
                Some(winsize.ws_col as usize)
            } else {
                None
            }
        }
    }
}

#[cfg(windows)]
mod platform_shell {
    use std::mem;
    use std::ptr;

    use windows_sys::Win32::Foundation::CloseHandle;
    use windows_sys::Win32::Foundation::GENERIC_READ;
    use windows_sys::Win32::Foundation::GENERIC_WRITE;
    use windows_sys::Win32::Foundation::INVALID_HANDLE_VALUE;
    use windows_sys::Win32::Storage::FileSystem::CreateFileA;
    use windows_sys::Win32::Storage::FileSystem::FILE_SHARE_READ;
    use windows_sys::Win32::Storage::FileSystem::FILE_SHARE_WRITE;
    use windows_sys::Win32::Storage::FileSystem::OPEN_EXISTING;
    use windows_sys::Win32::System::Console::CONSOLE_SCREEN_BUFFER_INFO;
    use windows_sys::Win32::System::Console::GetConsoleScreenBufferInfo;
    use windows_sys::Win32::System::Console::GetStdHandle;
    use windows_sys::Win32::System::Console::STD_ERROR_HANDLE;
    use windows_sys::core::PCSTR;

    pub fn stderr_width() -> Option<usize> {
        unsafe {
            let stdout = GetStdHandle(STD_ERROR_HANDLE);

            let mut csbi: CONSOLE_SCREEN_BUFFER_INFO = mem::zeroed();

            if GetConsoleScreenBufferInfo(stdout, &mut csbi) != 0 {
                return Some((csbi.srWindow.Right - csbi.srWindow.Left) as usize);
            }

            // On mintty/msys/cygwin based terminals, the above fails with
            // INVALID_HANDLE_VALUE. Use an alternate method which works
            // in that case as well.
            let h = CreateFileA(
                "CONOUT$\0".as_ptr() as PCSTR,
                GENERIC_READ | GENERIC_WRITE,
                FILE_SHARE_READ | FILE_SHARE_WRITE,
                ptr::null_mut(),
                OPEN_EXISTING,
                0,
                std::ptr::null_mut()
            );

            if h == INVALID_HANDLE_VALUE {
                return None;
            }

            let mut csbi: CONSOLE_SCREEN_BUFFER_INFO = mem::zeroed();
            _ = GetConsoleScreenBufferInfo(h, &mut csbi);

            CloseHandle(h);

            None
        }
    }
}
