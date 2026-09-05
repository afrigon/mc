# Environment Variables

The environment acts as an override layer on top of `mc.kdl`. Secrets are
deliberately kept out of the manifest so it can be committed and shared
safely: mc reads them from the environment, or from files it generates
readable only by the owning user (see [Instance
Layout](./instance-layout.md)).

## `MC_RCON_PASSWORD`

The password for RCON, the remote console protocol used to coordinate
backups with a running instance. It takes precedence over a
`"rcon.password"` entry in the `properties` block of the `server` section.

RCON is enabled exactly when a password is configured. When backups are
enabled and no password is set anywhere, mc generates one at each start so
backups work out of the box; set this variable when other tooling needs to
reach the remote console with a known password.

## `MC_BACKUPS_S3_BUCKET`

The S3 bucket that receives backup archives when the `storage` node of `backups` has
`type = "s3"`. It takes precedence over the `bucket` key in the manifest and
can replace it entirely.

S3 credentials are not read through mc-specific variables; they come from
the standard AWS credential chain — `AWS_ACCESS_KEY_ID` /
`AWS_SECRET_ACCESS_KEY`, the `~/.aws` configuration files, or an attached
IAM role.

## `MC_DISCORD_WEBHOOK`

The Discord webhook URL that notifications are posted to. Setting it is
what turns notifications on; without it none are sent. The `notifications`
section of the manifest selects which events are reported — instance
lifecycle, crashes, forced shutdowns, and backup results are all on by
default.
