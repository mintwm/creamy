#![cfg_attr(coverage_nightly, feature(coverage_attribute))]
#![allow(clippy::unreadable_literal)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
//#![deny(clippy::unwrap_used)]
//#![deny(clippy::panic)]
//#![deny(clippy::todo)]
//#![deny(clippy::as_conversions)]

mod compiler;
pub mod error;
pub mod model;
mod nodes;
mod table;
pub mod tokenizer;
mod tree;
mod utils;
mod version;

pub use compiler::ProtocolCompiler;
pub use model::definition::{Access, ProtocolDefinition};
pub use table::FinishedTypeTable;
pub use utils::{StringPoolIntern, StringPoolResolver};
pub use version::Version;

pub mod constraints {
    // Глобальные ограничения:
    //TODO: move to pool crate
    pub const MAX_UNIQUE_STRINGS: usize = 65536;

    /// * dst: u8,
    /// * group: u8,
    /// * src: u8,
    /// * kind: u8,
    pub const HEADER_BYTES: u8 = 4;

    pub const MAX_STRUCTS: usize = 2048;
    pub const MAX_ENUMS: usize = 2048;
    pub const MAX_FIELDS: usize = 2048;
    pub const MAX_VARIANTS: usize = 65536;
    pub const MAX_FIELD_PER_STRUCT: usize = 28;
    pub const MAX_PAYLOAD: usize = 28;
    pub const MAX_GROUPS: usize = 255;
    pub const MAX_MESSAGES_PER_GROUP: usize = 255;
    pub const MAX_MESSAGES: usize = MAX_GROUPS * MAX_MESSAGES_PER_GROUP;
    pub const MAX_TYPE_COUNT: usize = MAX_STRUCTS + MAX_ENUMS;
}

/*
 * TODO: bool type
 * TODO: C Header generation
 *
 * Pipeline:
 *     Tokenizer -> AST
 *     Resolve:
 *         Errors:
 *             Name duplicates
 *             Empty structs, fields, enums, messages, flags, arrays,
 *             Invalid size
 *             Field, variant, option count
 *
 *         Cache size and align
 *
 *
 *
 * Ограничения на протокол:
 *     i8/i16/u8/u16 - enum types
 *     28 полей на структуру
 *     28 свободных байт
 *     255 групп
 *     255 сообщений
 */
