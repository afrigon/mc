#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServerLogLevel {
    Fatal,
    Error,
    Warn,
    Info,
    Debug,
    Trace
}

impl ServerLogLevel {
    fn parse(level: &str) -> Option<Self> {
        match level {
            "FATAL" => Some(ServerLogLevel::Fatal),
            "ERROR" => Some(ServerLogLevel::Error),
            "WARN" => Some(ServerLogLevel::Warn),
            "INFO" => Some(ServerLogLevel::Info),
            "DEBUG" => Some(ServerLogLevel::Debug),
            "TRACE" => Some(ServerLogLevel::Trace),
            _ => None
        }
    }
}

/// A console line as printed with the pattern mc writes into `log4j2.xml`:
/// `[LEVEL] [thread]: message`.
pub struct ServerLogLine<'a> {
    pub level: ServerLogLevel,
    pub thread: &'a str,
    pub message: &'a str
}

impl<'a> ServerLogLine<'a> {
    pub fn parse(line: &'a str) -> Option<Self> {
        let rest = line.strip_prefix('[')?;
        let (level, rest) = rest.split_once("] [")?;
        let (thread, message) = rest.split_once("]: ")?;

        Some(ServerLogLine {
            level: ServerLogLevel::parse(level)?,
            thread,
            message
        })
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum ServerLogEvent {
    Joined(String),
    Left(String),
    SaveStarted,
    SaveCompleted
}

impl ServerLogEvent {
    pub fn recognize(line: &ServerLogLine<'_>) -> Option<Self> {
        if line.level != ServerLogLevel::Info || line.thread != "Server thread" {
            return None;
        }

        if let Some(name) = line.message.strip_suffix(" joined the game") {
            return player_name(name).map(ServerLogEvent::Joined);
        }

        if let Some(name) = line.message.strip_suffix(" left the game") {
            return player_name(name).map(ServerLogEvent::Left);
        }

        if line.message.starts_with("Saving chunks for level '") {
            return Some(ServerLogEvent::SaveStarted);
        }

        if line.message == "ThreadedAnvilChunkStorage: All dimensions are saved" {
            return Some(ServerLogEvent::SaveCompleted);
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
