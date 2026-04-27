# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build & run

Standard Cargo workflow:

```bash
cargo check            # fast iteration (does NOT need DECtalk libs to link)
cargo build --release  # full build, links DECtalk static libs from vendor/dectalk/dist/
cargo run -- -l debug  # run with verbose logging
```

Build-time requirements not in `Cargo.toml`:
- `clang` + `libclang-dev` for `bindgen` (build.rs generates DECtalk FFI from `vendor/dectalk/include/ttsapi.h`).
- The `vendor/dectalk/` directory must be present — DECtalk is statically linked and the headers live there.

Runtime requirements:
- `s.json` in the working directory containing `{ "token": "..." }` — gitignored, must be created locally before `cargo run`.
- `user.json` and `wallet_list.json` are created automatically.
- `LD_LIBRARY_PATH=vendor/dectalk/dist` is only needed if the embedded rpath (set in `build.rs` on Linux) fails. Docker sets it as a fallback.

There is no test suite. `cargo check` and a manual run are the only feedback loops.

## Architecture orientation

Read `README.md` ("Architecture notes" + "Project structure") first — it's accurate for the macro-driven DB layer, persistence task, and schedule reminders. The README's command list is **stale**: it lists three feature areas (mimic, schedule, wallet) but the codebase now has five. Current `User` aggregate (`src/pawthos/structs/user.rs`):

| Sub-struct | `User` field | Commands |
|---|---|---|
| `MimicUser` | `mimic` | `/mimic` |
| `ScheduleUser` | `schedule` | `/schedule` |
| `WalletUser` | `wallet` | `/daily`, `/balance`, `/color`, `/leaderboard` |
| `ProfileUser` | `profile` | `/profile` (bio, banner, colorway, equipped items) |
| `InventoryUser` | `inventory` | `/shop`, `/achievements` |

Profile + shop are an in-flight expansion governed by **`SHOP_PLAN.md`** (the design intent is in `SHOP_IDEAS.md`). At time of writing Phases 0–3 and 5–8 have landed; Phase 4 (paywall banners) is pending — `BANNERS: &[BannerDef] = &[]` in `src/pawthos/structs/shop_catalog.rs` is the explicit hole. When asked to work on the shop, treat `SHOP_PLAN.md` as the source of truth for phase ordering, error semantics, and migration concerns.

## Invariants

- **No direct file I/O in commands.** All persistence flows through `RwLock<UserDB>` writes, which auto-snapshot via mpsc to the persistence task. If you find yourself writing `std::fs::write` in a command, you're off the path.
- **Writes are atomic** (`.tmp` then rename). Don't bypass this with direct writes — a crash mid-write would corrupt `user.json`.
- **Adding a DB-backed feature requires both macros**, paired one-to-one: `def_db_access!` in `structs/data.rs` AND `impl_user_db_spec!` in `traits/mod.rs`. Forgetting either gives confusing trait-resolution errors. The 8-step recipe in README's "Adding a new feature" section is current.
- **Use `utils::reply_ok` / `reply_err` / `reply_info`** instead of constructing `Reply`/`EmbedType` manually. Command files should not import `EmbedType`.
- **Magic values live in `pawthos::consts`** (TAB_EMOJI, COLOR_ROLE_COST, faucet/lootbox tuning, etc.) — don't inline numeric constants in commands.
- **Catalog IDs are stable string keys** in `shop_catalog.rs` (e.g. `unlock_custom_banner`, `box_*`, `ach_*`). They're persisted in `InventoryUser` collections, so renaming a catalog ID is a data migration, not a refactor.
- **Errors use `thiserror`** per feature (`enums/{feature}_errors.rs`), with a `NoUserFound` variant required by the macro contract. The top-level `PawthosError` in `enums/pawthos_errors.rs` collects them via `#[from]`.

## What changes a lot, what doesn't

Stable: `pawthos/structs/data.rs` macro pair, `traits/mod.rs` markers, `utils.rs` reply helpers, the persistence task in `framework.rs`. Treat these as load-bearing — small changes ripple.

Volatile: `commands/shop/`, `commands/profile/`, `pawthos/structs/{shop_catalog,inventory_user,profile_user}.rs`. These are still under active iteration per `SHOP_PLAN.md`.
