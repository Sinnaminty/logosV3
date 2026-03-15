# logosV3

A Discord bot written in Rust. Third rewrite of a passion project — this time with async, proper error handling, and a macro-driven database layer.

---

## Features

| Command group | What it does |
|---|---|
| `/mimic` | Create named personas (name + avatar). Talk as them via Discord webhooks. Enable auto-mode to have every message you send automatically re-posted as your active mimic. |
| `/schedule` | Add timezone-aware events with date and time. The bot DMs you a reminder when the event arrives. Reminders survive bot restarts. |
| `/color` | Preview a hex colour as a 256×256 PNG swatch, or spend 10 tabs to buy a custom colour role. |
| `/daily` | Claim 10 tabs once every 24 hours. |
| `/balance` | Check your tab balance. |
| `/vox say` | Synthesise text as speech using the [DECtalk](https://github.com/dectalk/dectalk) TTS engine and post the WAV file. |

---

## Prerequisites

- Rust (edition 2024, stable toolchain)
- A Discord bot token in `s.json` (see [Configuration](#configuration))
- The DECtalk shared libraries — pre-built copies live in `vendor/dectalk/dist/`

On Linux the DECtalk `.so` files must be on `LD_LIBRARY_PATH` at runtime. The Docker setup handles this automatically.

---

## Configuration

Create `s.json` in the working directory (next to the binary or in the project root when running with `cargo run`):

```json
{ "token": "your-discord-bot-token-here" }
```

**Do not commit this file.** Add it to `.gitignore`.

### Data files

The bot reads and writes three JSON files in the working directory:

| File | Contents |
|---|---|
| `user.json` | All per-user data (mimics, schedule events, wallet balances). Created automatically on first run. |
| `wallet_list.json` | Tracks which users have claimed their daily reward today. Resets at midnight. |
| `s.json` | Bot token (you provide this). |

---

## Building & Running

### Cargo (local)

```bash
cargo build --release
LD_LIBRARY_PATH=vendor/dectalk/dist ./target/release/logosV3
```

Pass `--log-level debug` (or `-l debug`) for verbose output:

```bash
./target/release/logosV3 --log-level debug
```

### Docker

```bash
# Build the image
docker build -t logos-bot:latest .

# Run (mount s.json and data files from the host)
docker run -d --name logos --restart unless-stopped \
           -v $(pwd)/s.json:/app/s.json \
           -v $(pwd)/user.json:/app/user.json \
           logos-bot:latest
```

---

## Project structure

```
src/
├── main.rs             # Entry point — CLI args, logging, client startup
├── framework.rs        # Poise framework construction, persistence task, schedule task
├── handlers.rs         # Discord event handler (auto-mode) and error handler
├── logging.rs          # SimpleLogger initialisation
├── setup.rs            # Token loading, re-exports for main.rs
├── utils.rs            # reply_ok/err/info helpers, embed builder, webhook helper
├── dectalk.rs          # Safe Rust wrapper around the DECtalk C library
├── commands/
│   ├── mod.rs          # Command registry + general commands (help, pfp, daily, balance, color)
│   ├── vox.rs          # /vox say — DECtalk TTS
│   ├── mimic/
│   │   ├── mod.rs      # /mimic add, list, say
│   │   ├── set.rs      # /mimic set active_mimic, channel_override, auto
│   │   └── delete.rs   # /mimic delete mimic, active_mimic, channel_override
│   └── schedule/
│       └── mod.rs      # /schedule add, list, delete, set_tz
└── pawthos/            # Core domain — all data structures and logic
    ├── mod.rs
    ├── consts/         # Magic numbers and strings (costs, colours, emoji, …)
    ├── types/          # Type aliases (Error, Context, Reply, Result)
    ├── traits/         # UserDbSpec marker trait + impl_user_db_spec! macro
    ├── enums/          # Error types, EmbedType, PersistentData
    └── structs/        # Data, UserDB, User, MimicUser, ScheduleUser, WalletUser, …
```

---

## Architecture notes

### Database access

All per-user state lives in a single `RwLock<UserDB>` inside `Data`. Three marker types (`MimicDbMarker`, `ScheduleDbMarker`, `WalletDbMarker`) implement the `UserDbSpec` trait to route generic read/write helpers to the correct field on each `User`. The `def_db_access!` macro in `data.rs` generates the public async methods from one line each.

Every write automatically snapshots the database and sends it to the persistence task over an mpsc channel — no command ever touches the filesystem directly.

### Persistence

A single background `tokio::spawn` loop receives `PersistentData` messages and handles all file I/O sequentially. Writes are atomic: the bot writes to a `.tmp` file and renames it into place, so a crash mid-write never corrupts the database.

### Schedule reminders

A second background loop receives `(UserId, ScheduleEvent)` pairs and spawns a `tokio::time::sleep` task for each one. All saved events are re-queued on startup so reminders survive restarts.

### Mimic auto-mode

When auto-mode is enabled, the Discord `Message` event handler intercepts every message the user sends, re-posts it via a per-channel webhook as the active mimic persona, and deletes the original message. Channel overrides let the user use a different mimic in specific channels.

---

## Adding a new feature

### Simple slash command (no database)

1. Write the command function in `src/commands/myfeature.rs`.
2. Add `mod myfeature;` and `use crate::commands::myfeature::*;` to `commands/mod.rs`.
3. Add `myfeature()` to the `return_commands()` vector.
4. Use `ctx.send(utils::reply_ok("Title", "body")).await?`.

### Database-backed feature

1. Create `src/pawthos/structs/myfeature_user.rs` — the per-user sub-struct.
2. Create `src/pawthos/enums/myfeature_errors.rs` — a `thiserror` error enum with a `NoUserFound` variant.
3. Add `pub myfeature: MyfeatureUser` to `src/pawthos/structs/user.rs`.
4. Add a `Myfeature(#[from] MyfeatureError)` variant to `pawthos_errors.rs`.
5. Add `pub struct MyfeatureDbMarker;` and `impl_user_db_spec!(MyfeatureDbMarker, MyfeatureUser, myfeature);` to `traits/mod.rs`.
6. Add `def_db_access!(with_myfeature_user_read, with_myfeature_user_write, MyfeatureDbMarker, MyfeatureUser, MyfeatureError, MyfeatureError::NoUserFound);` inside `impl Data` in `data.rs`.
7. Export the new modules from the relevant `mod.rs` files.
8. Write the command file and register it (see above).

---

###### Powered by caffeine and lambda functions
