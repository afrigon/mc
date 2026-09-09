use std::collections::BTreeMap;

use chrono::DateTime;
use chrono::Utc;
use uuid::Uuid;

use crate::minecraft::log::ServerLogEvent;
use crate::minecraft::log::ServerLogLevel;
use crate::minecraft::log::ServerLogLine;
use crate::minecraft::players::BanDetails;
use crate::minecraft::players::BanEntry;
use crate::minecraft::players::OpEntry;
use crate::minecraft::players::offline_uuid;
use crate::minecraft::server_properties::ServerProperties;
use crate::utils::errors::McResult;

#[test]
fn offline_uuid_matches_the_server() {
    // UUID.nameUUIDFromBytes("OfflinePlayer:Notch".getBytes(UTF_8))
    assert_eq!(
        offline_uuid("Notch").to_string(),
        "b50ad385-829d-3141-a216-7e7d7539ba7f"
    );
}

#[test]
fn ban_entries_fill_the_server_defaults() -> McResult<()> {
    let now = DateTime::parse_from_rfc3339("2026-09-06T14:00:00Z")?.with_timezone(&Utc);
    let details = BanDetails {
        reason: None,
        created: None,
        expires: None
    };
    let entry = BanEntry::new(Uuid::nil(), String::from("Griefer"), &details, now);

    assert_eq!(entry.created, "2026-09-06 14:00:00 +0000");
    assert_eq!(entry.expires, "forever");
    assert_eq!(entry.reason, "Banned by an operator.");
    assert_eq!(entry.source, "mc");

    Ok(())
}

#[test]
fn op_entries_use_the_server_field_names() -> McResult<()> {
    let entry = OpEntry {
        uuid: Uuid::nil(),
        name: String::from("Notch"),
        level: 4,
        bypasses_player_limit: true
    };

    assert_eq!(
        serde_json::to_string(&entry)?,
        r#"{"uuid":"00000000-0000-0000-0000-000000000000","name":"Notch","level":4,"bypassesPlayerLimit":true}"#
    );

    Ok(())
}

#[test]
fn unknown_property_keys_are_reported() -> McResult<()> {
    let overrides = BTreeMap::from([
        (String::from("spawn-protection"), String::from("0")),
        (String::from("query.port"), String::from("25565")),
        (String::from("spawn-protetion"), String::from("0")),
        (String::from("fabric.custom"), String::from("x"))
    ]);

    let result = ServerProperties::default().to_entries(&overrides, &BTreeMap::new())?;

    assert_eq!(
        result.unknown_keys,
        vec!["fabric.custom", "spawn-protetion"]
    );
    assert_eq!(
        result.entries.get("spawn-protection").map(String::as_str),
        Some("0")
    );
    assert_eq!(
        result.entries.get("spawn-protetion").map(String::as_str),
        Some("0")
    );

    Ok(())
}

#[test]
fn console_lines_split_into_level_thread_and_message() {
    let line =
        ServerLogLine::parse("[WARN] [Worker-Main-3]: Can't keep up! Is the server overloaded?");

    assert!(line.is_some_and(|line| {
        line.level == ServerLogLevel::Warn
            && line.thread == Some("Worker-Main-3")
            && line.message == "Can't keep up! Is the server overloaded?"
    }));
    assert!(
        ServerLogLine::parse("[12:34:56] [Server thread/INFO]: Notch joined the game").is_none()
    );
    assert!(ServerLogLine::parse("[LOUD] [Server thread]: Notch joined the game").is_none());
    assert!(ServerLogLine::parse("\tat net.minecraft.server.MinecraftServer.run").is_none());
    assert!(
        ServerLogLine::parse(
            "Starting net.fabricmc.loader.impl.game.minecraft.BundlerClassPathCapture"
        )
        .is_none()
    );
}

#[test]
fn jvm_lines_carry_a_level_but_no_thread() {
    let line = ServerLogLine::parse(
        "WARNING: A terminally deprecated method in sun.misc.Unsafe has been called"
    );

    assert!(line.is_some_and(|line| {
        line.level == ServerLogLevel::Warn
            && line.thread.is_none()
            && line.message == "A terminally deprecated method in sun.misc.Unsafe has been called"
    }));

    let line = ServerLogLine::parse("ERROR: Could not create the Java Virtual Machine.");

    assert!(line.is_some_and(|line| line.level == ServerLogLevel::Error && line.thread.is_none()));
    assert_eq!(recognize("WARNING: Notch joined the game"), None);
}

fn recognize(line: &str) -> Option<ServerLogEvent> {
    ServerLogLine::parse(line)
        .as_ref()
        .and_then(ServerLogEvent::recognize)
}

#[test]
fn join_leave_and_save_events_are_recognized() {
    assert_eq!(
        recognize("[INFO] [Server thread]: Notch joined the game"),
        Some(ServerLogEvent::Joined(String::from("Notch")))
    );
    assert_eq!(
        recognize("[INFO] [Server thread]: Notch left the game"),
        Some(ServerLogEvent::Left(String::from("Notch")))
    );
    assert_eq!(
        recognize("[INFO] [Server thread]: Notch lost connection: Timed out"),
        Some(ServerLogEvent::Disconnected {
            name: String::from("Notch"),
            reason: String::from("Timed out")
        })
    );
    assert_eq!(
        recognize(
            "[INFO] [Server thread]: Saving chunks for level 'ServerLevel[world]'/minecraft:overworld"
        ),
        Some(ServerLogEvent::SaveStarted)
    );
    assert_eq!(
        recognize(
            "[INFO] [Server thread]: ThreadedAnvilChunkStorage (world): All chunks are saved"
        ),
        None
    );
    assert_eq!(
        recognize("[INFO] [Server thread]: ThreadedAnvilChunkStorage: All dimensions are saved"),
        Some(ServerLogEvent::SaveCompleted)
    );
}

#[test]
fn player_controlled_text_is_not_an_event() {
    let lines = [
        "[INFO] [Server thread]: <Notch> Steve joined the game",
        "[INFO] [Server thread]: <Notch> joined the game",
        "[INFO] [Server thread]: [Server] Steve left the game",
        "[INFO] [Server thread]: * Notch left the game",
        "[INFO] [Server thread]:  joined the game",
        "[INFO] [Server thread]: <Notch> Saving chunks for level 'x'",
        "[INFO] [Server thread]: <Notch> Steve lost connection: Timed out",
        "[WARN] [Server thread]: Notch joined the game",
        "[INFO] [Netty Server IO #1]: Notch joined the game",
        "[12:34:56] [Server thread/INFO]: Notch joined the game"
    ];

    for line in lines {
        assert_eq!(recognize(line), None, "{}", line);
    }
}
