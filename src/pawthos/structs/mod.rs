//! Domain structs.
//!
//! The structs in this module form the in-memory data model for the bot.
//! All of them derive [`serde::Serialize`] / [`serde::Deserialize`] so they
//! can be round-tripped through JSON for persistence.
//!
//! | Module | Contents |
//! |---|---|
//! | [`data`] | [`data::Data`] — the shared state object injected into every command |
//! | [`inventory_user`] | Per-user shop inventory, unlock flags, interaction stats |
//! | [`mimic`] | A single [`mimic::Mimic`] definition (name + optional avatar) |
//! | [`mimic_user`] | Per-user mimic state: active mimic, list, auto-mode, channel overrides |
//! | [`schedule_event`] | A single [`schedule_event::ScheduleEvent`] with time and timezone |
//! | [`schedule_user`] | Per-user schedule state: timezone and event list |
//! | [`user`] | Aggregates all per-user sub-structs into one [`user::User`] |
//! | [`user_db`] | [`user_db::UserDB`] — the top-level `HashMap<UserId, User>` |
//! | [`wallet_user`] | Per-user wallet state: tab balance and owned roles |

pub mod badge;
pub mod data;
pub mod inventory_user;
pub mod mimic;
pub mod mimic_user;
pub mod profile_user;
pub mod schedule_event;
pub mod schedule_user;
pub mod shop_catalog;
pub mod user;
pub mod user_db;
pub mod wallet_user;
