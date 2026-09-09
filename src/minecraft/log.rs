/// A console line as printed with the pattern mc writes into `log4j2.xml`:
/// `MINECRAFT [LEVEL] [thread]: message`.
pub struct ServerLogLine<'a> {
    pub level: &'a str,
    pub thread: &'a str,
    pub message: &'a str
}

impl<'a> ServerLogLine<'a> {
    pub fn parse(line: &'a str) -> Option<Self> {
        let rest = line.strip_prefix("MINECRAFT [")?;
        let (level, rest) = rest.split_once("] [")?;
        let (thread, message) = rest.split_once("]: ")?;

        Some(ServerLogLine {
            level,
            thread,
            message
        })
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum ServerLogEvent {
    Joined(String),
    Left(String)
}

impl ServerLogEvent {
    pub fn recognize(line: &str) -> Option<Self> {
        let line = ServerLogLine::parse(line)?;

        if line.level != "INFO" || line.thread != "Server thread" {
            return None;
        }

        if let Some(name) = line.message.strip_suffix(" joined the game") {
            return player_name(name).map(ServerLogEvent::Joined);
        }

        if let Some(name) = line.message.strip_suffix(" left the game") {
            return player_name(name).map(ServerLogEvent::Left);
        }

        None
    }
}

// Chat, `/say`, and `/me` all put a space or angle brackets before the
// player-controlled text, so a bare name is the only shape accepted.
fn player_name(name: &str) -> Option<String> {
    let valid = !name.is_empty()
        && name.len() <= 16
        && name
            .chars()
            .all(|c| c.is_ascii_graphic() && !matches!(c, '<' | '>'));

    valid.then(|| name.to_string())
}
