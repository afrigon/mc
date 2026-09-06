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
    allow-list #false
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
        spawn-protection 16
        "query.port" 25565
        rcon {
            broadcast "yes"
        }
    }
}

mods {
    modrinth {
        lithium "UPNexAfy"
        carpet "bGrLxJ8v"
    }

    http {
        my-mod "https://example.com/my-mod.jar"
    }
}

backups {
    on
    frequency "0 30 * * * *"
    keep 5
    s3 "my-bucket" region="us-east-1"
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
    assert!(!manifest.server.allow_list);
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
    assert_eq!(properties.len(), 3);

    assert_eq!(manifest.mods.len(), 3);
    assert!(matches!(
        manifest.mods.get("lithium"),
        Some(ManifestMod::Modrinth(version)) if version == "UPNexAfy"
    ));
    assert!(matches!(
        manifest.mods.get("carpet"),
        Some(ManifestMod::Modrinth(version)) if version == "bGrLxJ8v"
    ));
    assert!(matches!(
        manifest.mods.get("my-mod"),
        Some(ManifestMod::Http(url)) if url.as_str() == "https://example.com/my-mod.jar"
    ));

    assert!(manifest.backups.enabled);
    assert_eq!(manifest.backups.frequency, "0 30 * * * *");
    assert!(matches!(
        &manifest.backups.storage,
        BackupStorage::S3 { bucket, region: Some(region) } if bucket == "my-bucket" && region == "us-east-1"
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
    assert!(manifest.server.allow_list);
    assert_eq!(manifest.server.port, 25565);
    assert!(manifest.server.property_overrides()?.is_empty());
    assert!(manifest.mods.is_empty());
    assert!(!manifest.backups.enabled);
    assert_eq!(manifest.backups.frequency, "0 0 * * * *");
    assert!(matches!(
        &manifest.backups.storage,
        BackupStorage::Local { path, keep: 20 } if path.to_str() == Some("backups")
    ));
    assert!(manifest.notifications.on_sigkill);
    assert!(manifest.tunnel.is_none());

    Ok(())
}

#[test]
fn empty_blocks_use_defaults() -> McResult<()> {
    let manifest = Manifest::from_kdl_str(&with_section("java {\n}\nserver {\n}\nmods {\n}"))?;

    assert_eq!(manifest.java.max_memory, 4096);
    assert_eq!(manifest.server.capacity, 20);
    assert!(manifest.mods.is_empty());

    Ok(())
}

#[test]
fn missing_name_is_rejected() {
    let message = error_message(Manifest::from_kdl_str("description \"d\"\n"));

    assert!(message.contains("missing field `name`"), "{message}");
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
fn backups_flag_forms() -> McResult<()> {
    let off = Manifest::from_kdl_str(&with_section("backups {\n    keep 3\n}"))?;
    assert!(!off.backups.enabled);
    assert!(matches!(
        off.backups.storage,
        BackupStorage::Local { keep: 3, .. }
    ));

    let on = Manifest::from_kdl_str(&with_section("backups {\n    on\n}"))?;
    assert!(on.backups.enabled);

    let explicit = Manifest::from_kdl_str(&with_section("backups {\n    on #false\n}"))?;
    assert!(!explicit.backups.enabled);

    Ok(())
}

#[test]
fn local_storage_decodes() -> McResult<()> {
    let manifest = Manifest::from_kdl_str(&with_section(
        "backups {\n    keep 7\n    local \"/mnt/data/mc\"\n}"
    ))?;

    assert!(matches!(
        &manifest.backups.storage,
        BackupStorage::Local { path, keep: 7 } if path.to_str() == Some("/mnt/data/mc")
    ));

    Ok(())
}

#[test]
fn s3_storage_without_region_decodes() -> McResult<()> {
    let manifest = Manifest::from_kdl_str(&with_section("backups {\n    s3 \"b\"\n}"))?;

    assert!(matches!(
        manifest.backups.storage,
        BackupStorage::S3 { bucket, region: None } if bucket == "b"
    ));

    Ok(())
}

#[test]
fn both_storage_targets_are_rejected() {
    let source = with_section("backups {\n    local \"p\"\n    s3 \"b\"\n}");
    let message = error_message(Manifest::from_kdl_str(&source));

    assert!(message.contains("either `local` or `s3`"), "{message}");
}

#[test]
fn s3_without_bucket_is_rejected() {
    let source = with_section("backups {\n    s3 region=\"r\"\n}");
    let message = error_message(Manifest::from_kdl_str(&source));

    assert!(message.contains("missing the bucket value"), "{message}");
    assert!(!message.contains("#0"), "{message}");
}

#[test]
fn s3_unknown_property_is_rejected() {
    let source = with_section("backups {\n    s3 \"b\" bucket=\"c\"\n}");
    let message = error_message(Manifest::from_kdl_str(&source));

    assert!(message.contains("unknown field `bucket`"), "{message}");
}

#[test]
fn tunnel_forms() -> McResult<()> {
    for source in [with_section("tunnel"), with_section("tunnel {\n}")] {
        let manifest = Manifest::from_kdl_str(&source)?;

        assert_eq!(
            manifest.tunnel.map(|tunnel| tunnel.provider),
            Some("playit".parse()?)
        );
    }

    let pinned =
        Manifest::from_kdl_str(&with_section("tunnel {\n    provider \"playit@1.0.10\"\n}"))?;

    assert_eq!(
        pinned.tunnel.map(|tunnel| tunnel.provider),
        Some("playit@1.0.10".parse()?)
    );

    Ok(())
}

#[test]
fn tunnel_unknown_key_is_rejected() {
    let source = with_section("tunnel {\n    provider \"playit\"\n    foo 1\n}");
    let message = error_message(Manifest::from_kdl_str(&source));

    assert!(message.contains("unknown field `foo`"), "{message}");
}

#[test]
fn mod_listed_under_two_sources_is_rejected() {
    let source = with_section(
        "mods {\n    modrinth {\n        a \"1\"\n    }\n    http {\n        a \"https://e.com/a.jar\"\n    }\n}"
    );
    let message = error_message(Manifest::from_kdl_str(&source));

    assert!(
        message.contains("`a` is listed under more than one source"),
        "{message}"
    );
}

#[test]
fn unknown_mod_source_is_rejected() {
    let source = with_section("mods {\n    github {\n        a \"b\"\n    }\n}");
    let message = error_message(Manifest::from_kdl_str(&source));

    assert!(message.contains("unknown field `github`"), "{message}");
}

#[test]
fn invalid_mod_url_is_rejected_with_position() {
    let source = with_section("mods {\n    http {\n        a \"not a url\"\n    }\n}");
    let message = error_message(Manifest::from_kdl_str(&source));

    assert!(message.contains("line 5, column 11"), "{message}");
}

#[test]
fn wrong_value_type_is_rejected_with_position() {
    let source = with_section("server {\n    hardcore \"yes\"\n}");
    let message = error_message(Manifest::from_kdl_str(&source));

    assert!(message.contains("line 4, column 14"), "{message}");
    assert!(message.contains("expected a boolean"), "{message}");
}

#[test]
fn out_of_range_port_is_rejected_with_position() {
    let source = with_section("server {\n    port 70000\n}");
    let message = error_message(Manifest::from_kdl_str(&source));

    assert!(message.contains("line 4, column 10"), "{message}");
    assert!(message.contains("expected u16"), "{message}");
}

#[test]
fn white_list_property_is_rejected() {
    let source = with_section("server {\n    properties {\n        white-list #false\n    }\n}");
    let message = error_message(Manifest::from_kdl_str(&source));

    assert!(
        message.contains("the `white-list` entry in `properties` is managed by mc; set `allow-list` in `server` instead"),
        "{message}"
    );
}

#[test]
fn managed_property_is_rejected() {
    let source = with_section("server {\n    properties {\n        server-port 25566\n    }\n}");
    let message = error_message(Manifest::from_kdl_str(&source));

    assert!(
        message.contains("the `server-port` entry in `properties` is managed by mc; set `port` in `server` instead"),
        "{message}"
    );
}

#[test]
fn rcon_password_property_is_rejected() {
    let source =
        with_section("server {\n    properties {\n        \"rcon.password\" \"hunter2\"\n    }\n}");
    let message = error_message(Manifest::from_kdl_str(&source));

    assert!(
        message.contains(
            "the `rcon.password` entry in `properties` is managed by mc; set `MC_RCON_PASSWORD` in the environment instead"
        ),
        "{message}"
    );
}

#[test]
fn enable_rcon_property_is_rejected() {
    let source = with_section("server {\n    properties {\n        enable-rcon #true\n    }\n}");
    let message = error_message(Manifest::from_kdl_str(&source));

    assert!(
        message.contains("the `enable-rcon` entry in `properties` is managed by mc; rcon is enabled when a rcon password is configured"),
        "{message}"
    );
}

#[test]
fn unknown_key_is_rejected() {
    let source = with_section("java {\n    min_memory 1024\n}");
    let message = error_message(Manifest::from_kdl_str(&source));

    assert!(message.contains("unknown field `min_memory`"), "{message}");
}

#[test]
fn repeated_key_is_rejected_with_position() {
    let source = with_section("server {\n    port 1\n    port 2\n}");
    let message = error_message(Manifest::from_kdl_str(&source));

    assert!(
        message.contains("line 5, column 5: the `port` node appears more than once"),
        "{message}"
    );
}

#[test]
fn stray_argument_on_block_is_rejected_with_position() {
    let source = with_section("server \"stray\" {\n    port 1\n}");
    let message = error_message(Manifest::from_kdl_str(&source));

    assert!(
        message.contains("line 3, column 1: `server` cannot have both values and a block"),
        "{message}"
    );
}

#[test]
fn two_values_on_leaf_are_rejected_with_position() {
    let source = with_section("backups {\n    s3 \"b\" \"c\"\n}");
    let message = error_message(Manifest::from_kdl_str(&source));

    assert!(
        message.contains("line 4, column 5: `s3` takes a single value"),
        "{message}"
    );
}

#[test]
fn bare_leaf_is_rejected_with_position() {
    let source = with_section("server {\n    properties {\n        motd\n    }\n}");
    let message = error_message(Manifest::from_kdl_str(&source));

    assert!(
        message.contains("line 5, column 9: `motd` is empty"),
        "{message}"
    );
}

#[test]
fn property_null_is_rejected() {
    let source = with_section("server {\n    properties {\n        motd #null\n    }\n}");
    let message = error_message(Manifest::from_kdl_str(&source));

    assert!(!message.is_empty(), "{message}");
}

#[test]
fn syntax_error_reports_position() {
    let message = error_message(Manifest::from_kdl_str("name \"x\"\ndescription \"oops\n"));

    assert!(message.contains("line 2, column"), "{message}");
}

#[test]
fn slashdash_node_is_ignored() -> McResult<()> {
    let manifest =
        Manifest::from_kdl_str(&with_section("backups {\n    /-local \"x\"\n    on\n}"))?;

    assert!(manifest.backups.enabled);
    assert!(matches!(
        &manifest.backups.storage,
        BackupStorage::Local { path, .. } if path.to_str() == Some("backups")
    ));

    Ok(())
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
        "modrinth {\n\
         \x20   lithium version=\"UPNexAfy\" hash=\"sha512:abc\"\n\
         }\n\
         \n\
         http {\n\
         \x20   my-mod url=\"https://example.com/my-mod.jar\"\n\
         }\n"
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
fn empty_lockfile_round_trips() -> McResult<()> {
    let text = ModLockfile { mods: vec![] }.to_kdl_document().to_string();

    assert_eq!(text, "");
    assert!(ModLockfile::from_kdl_str(&text)?.mods.is_empty());

    Ok(())
}

#[test]
fn set_mod_version_creates_the_blocks() -> McResult<()> {
    let mut document: KdlDocument = "name \"x\"\n\nserver {\n    eula #true\n}\n".parse()?;

    document::set_mod_version(&mut document, "lithium", "UPNexAfy")?;

    assert_eq!(
        document.to_string(),
        "name \"x\"\n\nserver {\n    eula #true\n}\n\nmods {\n    modrinth {\n        lithium \"UPNexAfy\"\n    }\n}\n"
    );

    Ok(())
}

#[test]
fn set_mod_version_replaces_in_place_and_keeps_comments() -> McResult<()> {
    let source = "mods {\n  modrinth {\n    // keep me\n    lithium \"AAA\" // trailing\n    carpet \"BBB\"\n  }\n}\n";
    let mut document: KdlDocument = source.parse()?;

    document::set_mod_version(&mut document, "lithium", "ZZZ")?;
    document::set_mod_version(&mut document, "carpet", "CCC")?;
    document::set_mod_version(&mut document, "sodium", "DDD")?;

    assert_eq!(
        document.to_string(),
        "mods {\n  modrinth {\n    // keep me\n    lithium \"ZZZ\" // trailing\n    carpet \"CCC\"\n    sodium \"DDD\"\n  }\n}\n"
    );

    Ok(())
}

#[test]
fn set_mod_version_refuses_a_mod_from_another_source() -> McResult<()> {
    let mut document: KdlDocument =
        "mods {\n    http {\n        a \"https://e.com/a.jar\"\n    }\n}\n".parse()?;
    let message = error_message(document::set_mod_version(&mut document, "a", "V"));

    assert!(message.contains("already listed under `http`"), "{message}");

    Ok(())
}

#[test]
fn remove_mod_searches_every_source() -> McResult<()> {
    let source = "mods {\n    modrinth {\n        a \"1\"\n        b \"2\"\n    }\n    http {\n        c \"https://e.com/c.jar\"\n    }\n}\n";
    let mut document: KdlDocument = source.parse()?;

    assert!(document::remove_mod(&mut document, "b"));
    assert!(document::remove_mod(&mut document, "c"));
    assert!(!document::remove_mod(&mut document, "missing"));
    assert_eq!(
        document.to_string(),
        "mods {\n    modrinth {\n        a \"1\"\n    }\n    http {\n    }\n}\n"
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
         \x20   allow-list #true\n\
         \n\
         \x20   // Setting this to true indicates YOU have read and agree to the Minecraft EULA (https://aka.ms/MinecraftEULA).\n\
         \x20   // This agreement is between you and Mojang/Microsoft.\n\
         \x20   eula #true\n\
         }\n\
         \n\
         backups {\n\
         \x20   on\n\
         \x20   frequency \"0 0 * * * *\"\n\
         }\n"
    );

    let manifest = Manifest::from_kdl_str(&text)?;

    assert_eq!(manifest.name, "demo");
    assert!(manifest.server.eula);
    assert!(manifest.backups.enabled);
    assert_eq!(manifest.minecraft.loader, Some("fabric".parse()?));

    Ok(())
}

#[test]
fn preset_with_mods_appends_grouped_block() -> McResult<()> {
    let mut document = presets::create_document_base("demo", false, "26.2", true);

    document::set_mod_version(&mut document, "lithium", "UPNexAfy")?;
    document::set_mod_version(&mut document, "carpet", "bGrLxJ8v")?;

    let text = document.to_string();

    assert!(
        text.ends_with(
            "}\n\nmods {\n    modrinth {\n        lithium \"UPNexAfy\"\n        carpet \"bGrLxJ8v\"\n    }\n}\n"
        ),
        "{text}"
    );

    let manifest = Manifest::from_kdl_str(&text)?;

    assert_eq!(manifest.mods.len(), 2);
    assert!(!manifest.server.eula);

    Ok(())
}
