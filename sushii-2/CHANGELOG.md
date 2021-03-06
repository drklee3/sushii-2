# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [unreleased]

## [0.2.2] - 2021-02-23

### Added

-  Exponential back-off delays and max attempts for reminder DMs

### Fixed

-  Commands now correctly work in DMs

## [0.2.1] - 2021-02-19

### Added

-  Add more bot information to `stats` / `about` command.
-  Display created datetime on `tag info`
-  Add `tag remove` command alias to `tag delete`
-  Add `feed delete` command
-  Show feed IDs in `feed list`

### Fixed

-  Exit `feed add` when timed out

## [0.2.0] - 2021-02-18

### Added

-  Re-added old command aliases for `fishy` and `rank`
### Fixed

-  Correctly quit feed add
-  Prevent tag duplication when renaming
-  Added missing role mention for vlive feeds
-  Updated sushii.xyz links from old 2.sushii.xyz
-  Updated `invite` command link

## [0.2.0-rc.4] - 2021-02-11

### Added

-  `deletecase` command to delete a mod log case
-  `settings disablechannel` `settings enablechannel` `settings disabledchannels` commands

### Changed

-  `roles get` returns yaml default instead of json
-  DM users for reminders instead of pinging in channel
-  Ignore member updates for mutes when old/new member has same roles if available

## [0.2.0-rc.3] - 2021-02-02

### Fixed

-  Save all vlive feed items to prevent outdated notifications when adding a new feed
-  Sort mutes by remaining time in `listmutes`

## [0.2.0-rc.1] - 2021-01-29

### Added

-  Add other details to feeds embed
-  CPU / memory info to `about` / `stats` command

## [0.2.0-rc.0] - 2021-01-27

### Added

-  **BREAKING:** Feeds fetching, Requires `FEED_SERVER_URL` in environment variable config

### Fixed

-  Respond with error if commands use with invalid user ID

## [0.1.19] - 2021-01-11

### Added

-  `fm profile` command
-  `userinfo` command
-  `slowmode` command
-  `avatar` command
-  Reminders with `remind`, `remind list` commands
-  Add Last.fm metrics

### Fixed

-  Require `MANAGE_GUILD` permission for `prune`

## [0.1.18] - 2021-01-05

### Fixed

-  Prevent mentions when using short `[prefix][tag]`
-  Allow missing guild icon for guild cache
-  Fetch member when checking if user can see notification message

## [0.1.17] - 2021-01-04

### Added

-  Last.fm loved tracks command with `fm loved`
-  Last.fm top artists command with `fm topartists`

### Changed

-  Remove global keywords for performance reasons, users need to set keywords per guild
-  Allow `tag get` without the tag command prefix. Now supports `[prefix][tag name]` like regular commands
-  Format join message member number with commas

## [0.1.16] - 2021-01-02

### Added

-  Keyword notifications (`noti add`, `noti list`, `noti delete`)

## [0.1.15] - 2020-12-29

### Added

-  Prompt for confirmation when overwriting mod log reasons with `reason`
-  `first` command to get the first message in a channel

### Fixed

-  Fix mute DM formatting and add duration

## [0.1.14] - 2020-12-27

### Fixed

-  Set user level last_msg after resetting intervals instead of before
-  Save mod log entry msg_id after sending mod log message

## [0.1.13] - 2020-12-27

### Added

-  `fm recent` command
-  Send typing status when rendering rank image
-  `fishy` command alias `fwishy`
-  `serverinfo` command
-  Respond with prefix info when bot is mentioned
-  Add member logs
-  Leaderboards

### Changed

-  Use embeds for message logs
-  Migrate to `metrics` crate from `prometheus` crate
-  Clean up `listmutes` and reduce clutter
-  Delete mute entries when a member is banned
-  Lower fishy/rep cooldown from 1 day to 12 hours

### Fixed

-  Ignore bots for message logs
-  Replace usage of `naive_local()` with `naive_utc()`

## [0.1.12]

### Added

-  Add short `t` alias to `tag` commands
-  Add deleted / edited message logging (only caches messages if message logs are enabled)

### Fixed

-  Add tag delete command to command group
-  Update serenity to `current#692e98`, fixes ban reason truncation

### Changed

-  Delete 0 messages by default for `ban` command

## [0.1.11]

### Added

-  Last.fm commands `set` and `np` (now playing)

### Fixed

-  Show rank data for target user data instead of author's
-  Reset last fishy instead of rep when fishing for another user
-  Add to existing user fishies instead of replacing
-  Zero pad user discriminator in rank image
-  Prevent mentions when adding a tag

## [0.1.10]

### Added

-  Add on/off guild setting aliases "enable" and "disable"
-  Fishy and rep commands
-  Tag commands

### Changed

-  Use updated rank template context variables

### Fixed

-  Set user's last message to make max XP gain once per minute
-  Actually send roles.txt file with `roles listids` when length is over 2000 chars
-  Auto delete role error messages
-  Respond with error if configuration is invalid

## [0.1.9]

### Fixed

-  Parse missing reason when using `mute` command without a duration
-  Check permissions per command instead of only per group

## [0.1.8]

### Added

-  Join reactions
-  Allow setting mute defaultduration to indefinite
-  Sentry error logging
-  Update latest n mod log cases with `reason latest~n [reason]`

### Changed

-  Preserve group and role order to match role configuration
-  Respond with mute subcommands if no args are given
-  Replace TOML role configuration with YAML
-  Display `settings list` nicer in an embed

### Fixed

-  Validate join react setting

## [0.1.7]

### Added

-   `mute setduration` and `mute addduration` commands to update existing mute durations
-   Short prune alias `p` for faster prunes
-   Guild and guild member count metric gauges

### Changed

-   Use case insensitive role name search for `settings mute role` command
-   Help commands now link to website commands list
-   Move mute duration next to action name in mod logs
-   Update serenity to `0.9.1`

### Fixed

-   Respond with an error if attempting to mute already muted members
-   Prevent mutes with only duration provided to consider reason as empty string, causing Discord embeds to fail
-   Mute entries properly save their false pending state so that re-joining will re-assign mute roles
-   Remove serenity's default prefix `~`

## [0.1.6]

### Added

-   Reference to mute mod log case to determine mute executor on unmute
-   Pending status to mutes to allow for mute duration to be set per command
-   Allow duration to be set per mute by the `mute` command
-   Display mute duration in mod logs
-   `ModLogReporter` to send messages to mod logs easier
-   Settings subcommands to `set`, `off`, `on`, `toggle`, `show` the given guild setting

### Changed

-   `reason` responds with number of cases modified and reason
-   Enable guild settings by default
-   Rename `stats` command to `about`

### Fixed

-   `warn` command sends message in mod log

## [0.1.5] - 2020-10-16

### Added

-   Traefik, PostgreSQL backup, cAdvisor, Node Exporter docker-compose services
-   `listmutes` command
-   Mod action reason in response message
-   Info when roles group is run without sub command
-   Respond with error if no valid IDs are given to mod action executor

### Changed

-   Listen on sub paths instead of subdomains for Prometheus and Grafana services
-   Start interval tasks only once

### Fixed

-   Parse mod action executor IDs with length 17-19 instead of just 18-19
-   Handle unmutes when members are no longer in the guild

## [0.1.4] - 2020-10-01

### Fixed

-   Remove target-cpu codegen option

## [0.1.3] - 2020-10-01

### Added

-   Configurable metrics interface
-   Add colors to mod actions
-   Embed DB migrations
-   Add toggles for guild config settings
-   Remute users if leaving with a mute role
-   Auto unmute users after set time
-   Save moderator tag and id to audit log
-   Add case history summary

### Changed

-   Move Docker sushii binary to /usr/local/bin
-   Dedupe mod action ids
-   Handle shutdown signals cleanly

### Fixed

-   Add libssl-dev in Docker image
-   Fix correct NaiveDateTime format string
-   Prevent saving placeholder modlog reason
-   Prevent role handler from running outside of role channels
-   Fix mod log entry saving

[unreleased]: https://github.com/sushiibot/sushii-2/compare/sushii-2-v0.2.2...HEAD
[0.2.2]: https://github.com/sushiibot/sushii-2/compare/sushii-2-v0.2.1...sushii-2-v0.2.2
[0.2.1]: https://github.com/sushiibot/sushii-2/compare/sushii-2-v0.2.0...sushii-2-v0.2.1
[0.2.0]: https://github.com/sushiibot/sushii-2/compare/sushii-2-v0.2.0-rc.7...sushii-2-v0.2.0
[0.2.0-rc.0]: https://github.com/sushiibot/sushii-2/compare/sushii-2-v0.1.19...sushii-2-v0.2.0-rc.0
[0.1.19]: https://github.com/sushiibot/sushii-2/compare/sushii-2-v0.1.18...sushii-2-v0.1.19
[0.1.18]: https://github.com/sushiibot/sushii-2/compare/sushii-2-v0.1.17...sushii-2-v0.1.18
[0.1.17]: https://github.com/sushiibot/sushii-2/compare/sushii-2-v0.1.16...sushii-2-v0.1.17
[0.1.16]: https://github.com/sushiibot/sushii-2/compare/sushii-2-v0.1.15...sushii-2-v0.1.16
[0.1.15]: https://github.com/sushiibot/sushii-2/compare/sushii-2-v0.1.14...sushii-2-v0.1.15
[0.1.14]: https://github.com/sushiibot/sushii-2/compare/sushii-2-v0.1.13...sushii-2-v0.1.14
[0.1.13]: https://github.com/sushiibot/sushii-2/compare/v0.1.12...sushii-2-v0.1.13
[0.1.12]: https://github.com/sushiibot/sushii-2/compare/v0.1.11...v0.1.12
[0.1.11]: https://github.com/sushiibot/sushii-2/compare/v0.1.10...v0.1.11
[0.1.10]: https://github.com/sushiibot/sushii-2/compare/v0.1.9...v0.1.10
[0.1.9]: https://github.com/sushiibot/sushii-2/compare/v0.1.8...v0.1.9
[0.1.8]: https://github.com/sushiibot/sushii-2/compare/v0.1.7...v0.1.8
[0.1.7]: https://github.com/sushiibot/sushii-2/compare/v0.1.6...v0.1.7
[0.1.6]: https://github.com/sushiibot/sushii-2/compare/v0.1.5...v0.1.6
[0.1.5]: https://github.com/sushiibot/sushii-2/compare/v0.1.4...v0.1.5
[0.1.4]: https://github.com/sushiibot/sushii-2/compare/v0.1.3...v0.1.4
[0.1.3]: https://github.com/sushiibot/sushii-2/compare/v0.1.2...v0.1.3
[c:27120af]: https://github.com/sushiibot/sushii-2/commit/27120af575aed5f7a437152d8b4d16b3fcc7e7c1
