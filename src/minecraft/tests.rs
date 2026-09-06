use chrono::DateTime;
use chrono::Utc;
use uuid::Uuid;

use crate::minecraft::players::BanDetails;
use crate::minecraft::players::BanEntry;
use crate::minecraft::players::OpEntry;
use crate::minecraft::players::offline_uuid;
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
