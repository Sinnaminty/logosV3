//! Core domain logic and data layer for logosV3.
//!
//! The `pawthos` module is the heart of the bot. Everything that is not
//! Discord-specific plumbing lives here. It is organised into five
//! sub-modules:
//!
//! | Sub-module | Contents |
//! |---|---|
//! | [`consts`] | Compile-time constants (colours, costs, emoji strings, …) |
//! | [`enums`] | Error types, embed styling, and the persistence message enum |
//! | [`structs`] | All domain structs and the central [`structs::data::Data`] type |
//! | [`traits`] | The [`traits::UserDbSpec`] marker-trait system for generic DB access |
//! | [`types`] | Short type aliases used throughout the codebase |

pub mod consts;
pub mod enums;
pub mod structs;
pub mod traits;
pub mod types;
