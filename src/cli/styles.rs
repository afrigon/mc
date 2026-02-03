use anstyle::AnsiColor;
use anstyle::Effects;
use anstyle::Style;
use clap::builder::Styles;

pub const HEADER: Style = AnsiColor::BrightGreen.on_default().effects(Effects::BOLD);
pub const USAGE: Style = AnsiColor::BrightGreen.on_default().effects(Effects::BOLD);
pub const LITERAL: Style = AnsiColor::BrightCyan.on_default().effects(Effects::BOLD);
pub const PLACEHOLDER: Style = AnsiColor::Cyan.on_default();
pub const ERROR: Style = annotate_snippets::renderer::DEFAULT_ERROR_STYLE;
pub const WARN: Style = annotate_snippets::renderer::DEFAULT_WARNING_STYLE;
pub const VALID: Style = AnsiColor::BrightCyan.on_default().effects(Effects::BOLD);
pub const INVALID: Style = annotate_snippets::renderer::DEFAULT_WARNING_STYLE;

pub fn styles() -> Styles {
    Styles::styled()
        .header(HEADER)
        .usage(USAGE)
        .literal(LITERAL)
        .placeholder(PLACEHOLDER)
        .error(ERROR)
        .valid(VALID)
        .invalid(INVALID)
}
