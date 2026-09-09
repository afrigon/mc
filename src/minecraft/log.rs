use std::fmt;

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

/// A console line, either printed with the pattern mc writes into
/// `log4j2.xml` (`[LEVEL] [thread]: message`) or by the JVM itself outside
/// the logger (`WARNING: message`, `ERROR: message`), which has no thread.
pub struct ServerLogLine<'a> {
    pub level: ServerLogLevel,
    pub thread: Option<&'a str>,
    pub message: &'a str
}

impl<'a> ServerLogLine<'a> {
    pub fn parse(line: &'a str) -> Option<Self> {
        Self::parse_logger(line).or_else(|| Self::parse_jvm(line))
    }

    fn parse_logger(line: &'a str) -> Option<Self> {
        let rest = line.strip_prefix('[')?;
        let (level, rest) = rest.split_once("] [")?;
        let (thread, message) = rest.split_once("]: ")?;

        Some(ServerLogLine {
            level: ServerLogLevel::parse(level)?,
            thread: Some(thread),
            message
        })
    }

    fn parse_jvm(line: &'a str) -> Option<Self> {
        let (level, message) = if let Some(message) = line.strip_prefix("WARNING: ") {
            (ServerLogLevel::Warn, message)
        } else if let Some(message) = line.strip_prefix("ERROR: ") {
            (ServerLogLevel::Error, message)
        } else {
            return None;
        };

        Some(ServerLogLine {
            level,
            thread: None,
            message
        })
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum RefusalReason {
    NotInAllowList,
    Banned,
    AddressBanned,
    Full,
    NameTaken,
    Other(String)
}

impl RefusalReason {
    fn parse(reason: &str) -> Self {
        match reason.trim_end_matches(['!', '.']) {
            "You are not white-listed on this server" => RefusalReason::NotInAllowList,
            "You are banned from this server" => RefusalReason::Banned,
            "Your IP address is banned from this server"
            | "You have been IP banned from this server" => RefusalReason::AddressBanned,
            "The server is full" => RefusalReason::Full,
            "That name is already taken" => RefusalReason::NameTaken,
            _ => RefusalReason::Other(reason.to_string())
        }
    }

    pub fn raw(&self) -> Option<&str> {
        match self {
            RefusalReason::Other(reason) => Some(reason),
            _ => None
        }
    }
}

impl fmt::Display for RefusalReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RefusalReason::NotInAllowList => write!(f, "the user is not in the allow list"),
            RefusalReason::Banned => write!(f, "the user is banned"),
            RefusalReason::AddressBanned => write!(f, "the address is banned"),
            RefusalReason::Full => write!(f, "the instance is full"),
            RefusalReason::NameTaken => write!(f, "the name is already taken"),
            RefusalReason::Other(reason) => write!(f, "{}", reason)
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum ServerLogEvent {
    Joined(String),
    Left(String),
    Disconnected {
        name: String,
        reason: String
    },
    /// A connection dropped while the player was still logging in, which is
    /// how a login turned away by the server is reported.
    Refused {
        name: String,
        address: String,
        reason: RefusalReason
    },
    SaveStarted,
    SaveCompleted
}

impl ServerLogEvent {
    pub fn recognize(line: &ServerLogLine<'_>) -> Option<Self> {
        if line.level != ServerLogLevel::Info || line.thread != Some("Server thread") {
            return None;
        }

        if let Some(name) = line.message.strip_suffix(" joined the game") {
            return player_name(name).map(ServerLogEvent::Joined);
        }

        if let Some(name) = line.message.strip_suffix(" left the game") {
            return player_name(name).map(ServerLogEvent::Left);
        }

        if let Some((subject, reason)) = line.message.split_once(" lost connection: ") {
            let login = subject
                .strip_suffix(')')
                .and_then(|subject| subject.split_once(" (/"));

            return match login {
                Some((name, address)) => player_name(name).map(|name| ServerLogEvent::Refused {
                    name,
                    address: address.to_string(),
                    reason: RefusalReason::parse(reason)
                }),
                None => player_name(subject).map(|name| ServerLogEvent::Disconnected {
                    name,
                    reason: reason.to_string()
                })
            };
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
