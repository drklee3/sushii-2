# sushii-2

Rewrite of [sushii-bot](https://github.com/drklee3/sushii-bot) with async
[serenity-rs](https://github.com/serenity-rs/serenity/).

wip... again.

## Features

Slimmed down feature set of original sushii-bot with a focus on moderation tools

-   [ ] moderation tools
    -   [x] ban / unban
    -   [x] kick
    -   [x] prune
    -   [x] warn
    -   [x] mute / unmute
        -   [x] timed mutes
            -   [x] set duration per mute (s!!mute [ids] [duration and reason])
            -   [x] remove time duration (indefinite)
            -   [x] adjust existing mute duration
        -   [ ] allow muting users who left
        -   [ ] mute users who joined in last x minutes
    -   [x] cases
        -   [x] reason
        -   [x] history
    -   [ ] user ID lookup (list of ids to check if in guild, banned, muted, etc)
    -   [x] roles
        -   [x] json support
        -   [x] toml support
    -   [ ] settings
        -   [ ] dm users on action with reason (mutes only)
            -   [ ] toggle
            -   [ ] set message
        -   [ ] default mute duration (0 to disable)
        -   [ ] default num messages delete messages on ban
    -   [ ] auto boost role?
-   user
    -   avatar
    -   userinfo
    -   notifications
