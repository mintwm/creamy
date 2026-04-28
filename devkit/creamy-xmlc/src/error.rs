use std::num::ParseIntError;

use thiserror::Error;

/*
 * 000-099 - Ошибки парсера
 * 100-199 - Ошибки валидации
 * 200-299 - Ошибки анализатора
 */

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ProtocolError {
    // --- Parser errors ---
    #[error("[P000] Roxmltree: {0}")]
    Xml(#[from] roxmltree::Error),

    #[error("[P001] Unknown tag: {0}")]
    UnknownTag(String),

    #[error("[P002] Expected <protocol> token, found something else")]
    MissingProtocolToken,

    #[error("[P003] <{tag}>: missing '{attr}' attribute")]
    MissingAttribute { tag: String, attr: String },

    #[error("[P004] Version must be in format MAJOR(255).MINOR(255)")]
    InvalidVersionFormat,

    #[error("[P005] Invalid major version")]
    InvalidMajor,

    #[error("[P007] Invalid minor version")]
    InvalidMinor,

    #[error("[P008] Unexpected token: {0}")]
    UnexpectedToken(String),

    #[error("[P009] Invalid access value: {0}. Allowed: [SingleWrite, MultipleWrite]")]
    InvalidAccess(String),

    #[error("[P010] Invalid syntax: {src}. Should be: {should_be}")]
    Syntax { src: String, should_be: String },

    #[error("[P011] Parse error: {0}")]
    IntParse(#[from] ParseIntError),

    // --- Global Limits ---
    #[error("[P101] Too many unique strings. Max 65535")]
    TooManyUniqueStrings,

    #[error("[P102] Invalid size {0}: must be between 1 and 28")]
    InvalidSize(usize),

    #[error("[P103] Target align '{0}' is not power of 2")]
    AlignIsNotPowerOfTwo(u8),

    #[error("[P104] Target align cannot be '{0}'. Allowed align: [1, 2, 4, 8]")]
    ForbiddenAlign(u8),

    #[error("[P105] Target raw align cannot be '{0}'. Allowed align: [0, 1, 2, 3]")]
    ForbiddenRawAlign(u8),

    // --- Protocol Limits ---
    #[error("[P201] Too many structures in protocol. Max 2048")]
    TooManyStructs,

    #[error("[P202] Too many enums in protocol. Max 2048")]
    TooManyEnums,

    #[error("[P203] Too many total fields in protocol. Max 2048")]
    TooManyFields,

    #[error("[P204] Too many enum variants in protocol. Max 65535)")]
    TooManyVariants,

    #[error("[P205] Invalid enum underlying type. Supported: i8, i16, u8, u16")]
    InvalidEnumStorageType,

    #[error("[P206] Too many fields in '{0}' struct. Max 28")]
    StructFieldLimitExceeded(String),

    #[error("[P207] Not enough free space in '{0}' struct. Max 28 reserved bytes")]
    FreeBytesLimitExceeded(String),

    #[error("[P208] Too many groups in protocol. Max 255")]
    TooManyGroups,

    #[error("[P209] Too many messages in protocol. Max 255")]
    TooManyMessages,
}

pub trait ProtocolErrorExt<T: Default> {
    fn or_save_to(self, errors: &mut Vec<ProtocolError>) -> T;
}

impl<T: Default> ProtocolErrorExt<T> for Result<T, ProtocolError> {
    fn or_save_to(self, errors: &mut Vec<ProtocolError>) -> T {
        match self {
            Ok(value) => value,
            Err(error) => {
                errors.push(error);
                T::default()
            }
        }
    }
}
