use std::io::BufRead;
use std::io::IsTerminal;

use crate::utils::errors::McResult;
use crate::utils::shell::Shell;

/// Asks a yes/no question on the terminal, defaulting to no. Answers no
/// without asking when stdin is not a terminal, so scripted use never hangs
/// on a prompt.
pub fn confirm(shell: &mut Shell, question: &str) -> McResult<bool> {
    if !std::io::stdin().is_terminal() {
        return Ok(false);
    }

    write!(shell.err(), "{} [y/N] ", question)?;
    shell.err().flush()?;

    let mut answer = String::new();
    std::io::stdin().lock().read_line(&mut answer)?;

    let answer = answer.trim();

    Ok(answer.eq_ignore_ascii_case("y") || answer.eq_ignore_ascii_case("yes"))
}
