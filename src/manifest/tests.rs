use kdl::KdlDocument;
use url::Url;

use crate::manifest::Manifest;
use crate::manifest::ManifestMod;
use crate::manifest::document;
use crate::manifest::lock::ModLockfile;
use crate::manifest::lock::ModLockfileEntry;
use crate::manifest::lock::ModLockfileSource;
use crate::manifest::presets;
use crate::minecraft::MinecraftDifficulty;
use crate::minecraft::MinecraftGamemode;
use crate::minecraft::MinecraftLevelKind;
use crate::minecraft::seed::MinecraftSeed;
use crate::mods::service::ModServiceKind;
use crate::ops::backups::BackupStorage;
use crate::utils::errors::McResult;

const FULL: &str = r#"
name "x"
description "X Minecraft Server"

java {
    version "graal@25"
    min-memory 2048
    max-memory 8192
    jvm-arguments "-XX:+AlwaysPreTouch" "-Dfoo=bar"
}

minecraft {
    version "26.2"
    loader "fabric"
}

server {
    gamemode "creative"
    difficulty "hard"
    level-type "minecraft:flat"
    hardcore #true
    seed -1412583731547517931
    ip "0.0.0.0"
    port 25566
    rcon-port 25576
    capacity 10
    view-distance 12
    simulation-distance 8

    // Setting this to true indicates YOU have read and agree to the Minecraft EULA (https://aka.ms/MinecraftEULA).
    // This agreement is between you and Mojang/Microsoft.
    eula #true

    properties {
        white-list #false
        spawn-protection 16
        "query.port" 25565
        rcon {
            broadcast "yes"
        }
    }
}

mods {
    lithium "UPNexAfy"
    carpet "bGrLxJ8v" service="modrinth"
    my-mod url="https://example.com/my-mod.jar"
}

backups {
    enabled #true
    frequency "0 0 * * * *"
    storage "local" path="/mnt/data/mc" keep=5
}

notifications {
    on-lifecycle-event #false
    on-backup #false
}

tunnel {
    provider "playit@1.0.10"
}
"#;

fn error_message<T>(result: McResult<T>) -> String {
    match result {
        Ok(_) => String::new(),
        Err(error) => format!("{:#}", error)
    }
}

fn with_section(section: &str) -> String {
    format!("name \"x\"\ndescription \"d\"\n{}\n", section)
}

#[test]
fn full_manifest_decodes() -> McResult<()> {
    let manifest = Manifest::from_kdl_str(FULL)?;

    assert_eq!(manifest.name, "x");
    assert_eq!(manifest.description, "X Minecraft Server");

    assert_eq!(manifest.java.version, "graal@25".parse()?);
    assert_eq!(manifest.java.min_memory, 2048);
    assert_eq!(manifest.java.max_memory, 8192);
    assert_eq!(
        manifest.java.jvm_arguments,
        ["-XX:+AlwaysPreTouch", "-Dfoo=bar"]
    );

    assert_eq!(manifest.minecraft.version.as_deref(), Some("26.2"));
    assert_eq!(manifest.minecraft.loader, Some("fabric".parse()?));

    assert!(matches!(
        manifest.server.gamemode,
        MinecraftGamemode::Creative
    ));
    assert!(matches!(
        manifest.server.difficulty,
        MinecraftDifficulty::Hard
    ));
    assert!(matches!(
        manifest.server.level_type,
        MinecraftLevelKind::Flat
    ));
    assert!(manifest.server.hardcore);
    assert_eq!(
        manifest.server.seed,
        Some(MinecraftSeed::Numeric(-1412583731547517931))
    );
    assert_eq!(manifest.server.ip.as_deref(), Some("0.0.0.0"));
    assert_eq!(manifest.server.port, 25566);
    assert_eq!(manifest.server.rcon_port, 25576);
    assert_eq!(manifest.server.capacity, 10);
    assert_eq!(manifest.server.view_distance, 12);
    assert_eq!(manifest.server.simulation_distance, 8);
    assert!(manifest.server.eula);

    let properties = manifest.server.property_overrides()?;
    assert_eq!(
        properties.get("white-list").map(String::as_str),
        Some("false")
    );
    assert_eq!(
        properties.get("spawn-protection").map(String::as_str),
        Some("16")
    );
    assert_eq!(
        properties.get("query.port").map(String::as_str),
        Some("25565")
    );
    assert_eq!(
        properties.get("rcon.broadcast").map(String::as_str),
        Some("yes")
    );
    assert_eq!(properties.len(), 4);

    assert_eq!(manifest.mods.len(), 3);
    assert!(matches!(
        manifest.mods.get("lithium"),
        Some(ManifestMod::Version(version)) if version == "UPNexAfy"
    ));
    assert!(matches!(
        manifest.mods.get("carpet"),
        Some(ManifestMod::Detailed { version, service: ModServiceKind::Modrinth }) if version == "bGrLxJ8v"
    ));
    assert!(matches!(
        manifest.mods.get("my-mod"),
        Some(ManifestMod::Remote { url }) if url.as_str() == "https://example.com/my-mod.jar"
    ));

    assert!(manifest.backups.enabled);
    assert_eq!(manifest.backups.frequency, "0 0 * * * *");
    assert!(matches!(
        &manifest.backups.storage,
        BackupStorage::Local { path, keep: 5 } if path.to_str() == Some("/mnt/data/mc")
    ));

    assert!(!manifest.notifications.on_lifecycle_event);
    assert!(!manifest.notifications.on_backup);
    assert!(manifest.notifications.on_backup_failure);
    assert!(manifest.notifications.on_panic);
    assert!(manifest.notifications.on_sigkill);

    assert_eq!(
        manifest.tunnel.map(|tunnel| tunnel.provider),
        Some("playit@1.0.10".parse()?)
    );

    Ok(())
}

#[test]
fn minimal_manifest_uses_defaults() -> McResult<()> {
    let manifest = Manifest::from_kdl_str("name \"x\"\ndescription \"d\"\n")?;

    assert_eq!(manifest.java.version, "graal@25".parse()?);
    assert_eq!(manifest.java.min_memory, 4096);
    assert_eq!(manifest.java.jvm_arguments.len(), 3);
    assert_eq!(manifest.minecraft.version, None);
    assert_eq!(manifest.minecraft.loader, None);
    assert!(matches!(
        manifest.server.gamemode,
        MinecraftGamemode::Survival
    ));
    assert!(matches!(
        manifest.server.difficulty,
        MinecraftDifficulty::Normal
    ));
    assert!(!manifest.server.eula);
    assert_eq!(manifest.server.port, 25565);
    assert!(manifest.server.property_overrides()?.is_empty());
    assert!(manifest.mods.is_empty());
    assert!(!manifest.backups.enabled);
    assert!(matches!(
        &manifest.backups.storage,
        BackupStorage::Local { path, keep: 20 } if path.to_str() == Some("backups")
    ));
    assert!(manifest.notifications.on_sigkill);
    assert!(manifest.tunnel.is_none());

    Ok(())
}

#[test]
fn tunnel_section_without_provider_uses_the_default_provider() -> McResult<()> {
    for source in [with_section("tunnel"), with_section("tunnel {\n}")] {
        let manifest = Manifest::from_kdl_str(&source)?;

        assert_eq!(
            manifest.tunnel.map(|tunnel| tunnel.provider),
            Some("playit".parse()?)
        );
    }

    Ok(())
}

#[test]
fn local_storage_requires_a_path() {
    let source = with_section("backups {\n    storage \"local\" keep=3\n}");
    let message = error_message(Manifest::from_kdl_str(&source));

    assert!(
        message.contains("local storage requires a `path`"),
        "{message}"
    );
}

#[test]
fn empty_section_uses_defaults() -> McResult<()> {
    let manifest = Manifest::from_kdl_str(&with_section("java\nserver {\n}"))?;

    assert_eq!(manifest.java.max_memory, 4096);
    assert_eq!(manifest.server.capacity, 20);

    Ok(())
}

#[test]
fn missing_name_is_rejected() {
    let message = error_message(Manifest::from_kdl_str("description \"d\"\n"));

    assert!(message.contains("`name` node is required"), "{message}");
}

#[test]
fn seed_accepts_text() -> McResult<()> {
    let manifest = Manifest::from_kdl_str(&with_section("server {\n    seed \"glacier\"\n}"))?;

    assert_eq!(
        manifest.server.seed,
        Some(MinecraftSeed::Text(String::from("glacier")))
    );

    Ok(())
}

#[test]
fn mod_with_version_and_url_is_rejected() {
    let source = with_section("mods {\n    a \"v\" url=\"https://example.com/a.jar\"\n}");
    let message = error_message(Manifest::from_kdl_str(&source));

    assert!(message.contains("either a version or a url"), "{message}");
}

#[test]
fn mod_without_version_or_url_is_rejected() {
    let message = error_message(Manifest::from_kdl_str(&with_section("mods {\n    a\n}")));

    assert!(message.contains("requires a version or a url"), "{message}");
}

#[test]
fn mod_listed_twice_is_rejected() {
    let source = with_section("mods {\n    a \"1\"\n    a \"2\"\n}");
    let message = error_message(Manifest::from_kdl_str(&source));

    assert!(message.contains("listed more than once"), "{message}");
}

#[test]
fn storage_s3_decodes_without_bucket() -> McResult<()> {
    let manifest = Manifest::from_kdl_str(&with_section("backups {\n    storage \"s3\"\n}"))?;

    assert!(matches!(
        manifest.backups.storage,
        BackupStorage::S3 { bucket: None }
    ));

    Ok(())
}

#[test]
fn storage_s3_decodes_bucket() -> McResult<()> {
    let source = with_section("backups {\n    storage \"s3\" bucket=\"b\"\n}");
    let manifest = Manifest::from_kdl_str(&source)?;

    assert!(matches!(
        manifest.backups.storage,
        BackupStorage::S3 { bucket: Some(bucket) } if bucket == "b"
    ));

    Ok(())
}

#[test]
fn unknown_storage_kind_is_rejected() {
    let source = with_section("backups {\n    storage \"ftp\" host=\"h\"\n}");
    let message = error_message(Manifest::from_kdl_str(&source));

    assert!(message.contains("unknown storage kind `ftp`"), "{message}");
}

#[test]
fn storage_rejects_property_of_other_kind() {
    let source = with_section("backups {\n    storage \"local\" bucket=\"b\"\n}");
    let message = error_message(Manifest::from_kdl_str(&source));

    assert!(message.contains("unknown property `bucket`"), "{message}");
}

#[test]
fn property_without_scalar_is_rejected() {
    let source = with_section("server {\n    properties {\n        motd #null\n    }\n}");
    let message = error_message(Manifest::from_kdl_str(&source));

    assert!(message.contains("`motd` must be a string"), "{message}");
}

#[test]
fn property_with_several_values_is_rejected() {
    let source = with_section("server {\n    properties {\n        motd \"a\" \"b\"\n    }\n}");
    let message = error_message(Manifest::from_kdl_str(&source));

    assert!(
        message.contains("either a single value or a block"),
        "{message}"
    );
}

#[test]
fn unknown_node_in_section_is_rejected() {
    let source = with_section("java {\n    min_memory 1024\n}");
    let message = error_message(Manifest::from_kdl_str(&source));

    assert!(
        message.contains("unknown node `min_memory` in the `java` section"),
        "{message}"
    );
}

#[test]
fn unknown_top_level_node_is_rejected() {
    let message = error_message(Manifest::from_kdl_str(&with_section("jaba {\n}")));

    assert!(
        message.contains("unknown node `jaba` in the manifest"),
        "{message}"
    );
}

#[test]
fn repeated_node_is_rejected() {
    let source = with_section("server {\n    port 1\n    port 2\n}");
    let message = error_message(Manifest::from_kdl_str(&source));

    assert!(
        message.contains("`port` node appears more than once"),
        "{message}"
    );
}

#[test]
fn wrong_value_type_is_rejected() {
    let source = with_section("server {\n    hardcore \"yes\"\n}");
    let message = error_message(Manifest::from_kdl_str(&source));

    assert!(
        message.contains("`hardcore` node must be #true or #false"),
        "{message}"
    );
}

#[test]
fn syntax_error_reports_position() {
    let message = error_message(Manifest::from_kdl_str("name \"x\"\ndescription \"oops\n"));

    assert!(message.contains("line 2, column"), "{message}");
}

#[test]
fn lockfile_round_trips() -> McResult<()> {
    let lockfile = ModLockfile {
        mods: vec![
            ModLockfileEntry {
                name: String::from("lithium"),
                version: Some(String::from("UPNexAfy")),
                source: ModLockfileSource::Modrinth,
                hash: Some(String::from("sha512:abc"))
            },
            ModLockfileEntry {
                name: String::from("my-mod"),
                version: None,
                source: ModLockfileSource::Url(Url::parse("https://example.com/my-mod.jar")?),
                hash: None
            },
        ]
    };

    let text = lockfile.to_kdl_document().to_string();

    assert_eq!(
        text,
        "mod \"lithium\" version=\"UPNexAfy\" source=\"modrinth\" hash=\"sha512:abc\"\n\
         mod \"my-mod\" source=\"url+https://example.com/my-mod.jar\"\n"
    );

    let parsed = ModLockfile::from_kdl_str(&text)?;

    assert_eq!(parsed.mods.len(), 2);
    assert_eq!(parsed.mods[0].name, "lithium");
    assert_eq!(parsed.mods[0].version.as_deref(), Some("UPNexAfy"));
    assert_eq!(parsed.mods[0].source, ModLockfileSource::Modrinth);
    assert_eq!(parsed.mods[0].hash.as_deref(), Some("sha512:abc"));
    assert_eq!(parsed.mods[1].name, "my-mod");
    assert_eq!(parsed.mods[1].version, None);
    assert_eq!(parsed.mods[1].source, lockfile.mods[1].source);

    Ok(())
}

#[test]
fn set_mod_version_creates_the_block() -> McResult<()> {
    let mut document: KdlDocument = "name \"x\"\n\nserver {\n    eula #true\n}\n".parse()?;

    document::set_mod_version(&mut document, "lithium", "UPNexAfy")?;

    assert_eq!(
        document.to_string(),
        "name \"x\"\n\nserver {\n    eula #true\n}\n\nmods {\n    lithium \"UPNexAfy\"\n}\n"
    );

    Ok(())
}

#[test]
fn set_mod_version_replaces_in_place_and_keeps_comments() -> McResult<()> {
    let source = "mods {\n  // keep me\n  lithium \"AAA\" // trailing\n  carpet \"BBB\" service=\"modrinth\"\n}\n";
    let mut document: KdlDocument = source.parse()?;

    document::set_mod_version(&mut document, "lithium", "ZZZ")?;
    document::set_mod_version(&mut document, "carpet", "CCC")?;
    document::set_mod_version(&mut document, "sodium", "DDD")?;

    assert_eq!(
        document.to_string(),
        "mods {\n  // keep me\n  lithium \"ZZZ\" // trailing\n  carpet \"CCC\" service=\"modrinth\"\n  sodium \"DDD\"\n}\n"
    );

    Ok(())
}

#[test]
fn set_mod_version_turns_a_remote_mod_into_a_versioned_one() -> McResult<()> {
    let mut document: KdlDocument =
        "mods {\n    a url=\"https://example.com/a.jar\"\n}\n".parse()?;

    document::set_mod_version(&mut document, "a", "V")?;

    assert_eq!(document.to_string(), "mods {\n    a \"V\"\n}\n");

    Ok(())
}

#[test]
fn remove_mod_deletes_only_the_named_node() -> McResult<()> {
    let mut document: KdlDocument = "mods {\n    a \"1\"\n    b \"2\"\n    c \"3\"\n}\n".parse()?;

    assert!(document::remove_mod(&mut document, "b"));
    assert!(!document::remove_mod(&mut document, "missing"));
    assert_eq!(
        document.to_string(),
        "mods {\n    a \"1\"\n    c \"3\"\n}\n"
    );

    Ok(())
}

#[test]
fn remove_mod_without_block_is_a_noop() -> McResult<()> {
    let mut document: KdlDocument = "name \"x\"\n".parse()?;

    assert!(!document::remove_mod(&mut document, "a"));
    assert_eq!(document.to_string(), "name \"x\"\n");

    Ok(())
}

#[test]
fn preset_base_document_is_a_valid_manifest() -> McResult<()> {
    let document = presets::create_document_base("demo", true, "26.2", true);
    let text = document.to_string();

    assert_eq!(
        text,
        "name \"demo\"\n\
         description \"A Minecraft Server\"\n\
         \n\
         minecraft {\n\
         \x20   version \"26.2\"\n\
         \x20   loader \"fabric\"\n\
         }\n\
         \n\
         server {\n\
         \x20   gamemode \"survival\"\n\
         \x20   difficulty \"normal\"\n\
         \x20   hardcore #false\n\
         \n\
         \x20   // Setting this to true indicates YOU have read and agree to the Minecraft EULA (https://aka.ms/MinecraftEULA).\n\
         \x20   // This agreement is between you and Mojang/Microsoft.\n\
         \x20   eula #true\n\
         }\n\
         \n\
         backups {\n\
         \x20   enabled #true\n\
         \x20   frequency \"0 0 * * * *\"\n\
         }\n"
    );

    let manifest = Manifest::from_kdl_str(&text)?;

    assert_eq!(manifest.name, "demo");
    assert!(manifest.server.eula);
    assert_eq!(manifest.minecraft.loader, Some("fabric".parse()?));

    Ok(())
}

#[test]
fn preset_with_mods_appends_an_indented_block() -> McResult<()> {
    let mut document = presets::create_document_base("demo", false, "26.2", true);

    document::set_mod_version(&mut document, "lithium", "UPNexAfy")?;
    document::set_mod_version(&mut document, "carpet", "bGrLxJ8v")?;

    let text = document.to_string();

    assert!(
        text.ends_with("}\n\nmods {\n    lithium \"UPNexAfy\"\n    carpet \"bGrLxJ8v\"\n}\n"),
        "{text}"
    );

    let manifest = Manifest::from_kdl_str(&text)?;

    assert_eq!(manifest.mods.len(), 2);
    assert!(!manifest.server.eula);

    Ok(())
}
