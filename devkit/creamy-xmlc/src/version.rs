use std::{fmt::Display, str::FromStr};

use binrw::{BinRead, BinWrite};

use crate::error::ProtocolError;

#[derive(BinRead, BinWrite, Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Version {
    pub major: u16,
    pub minor: u16,
}

impl FromStr for Version {
    type Err = ProtocolError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.split('.');
        let (major, minor) = match (parts.next(), parts.next(), parts.next()) {
            (Some(major), Some(minor), None) => (major.trim(), minor.trim()),
            _ => return Err(ProtocolError::InvalidVersionFormat),
        };

        Ok(Version {
            major: major.parse().map_err(|_| ProtocolError::InvalidMajor)?,
            minor: minor.parse().map_err(|_| ProtocolError::InvalidMinor)?,
        })
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
impl Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}", self.major, self.minor)
    }
}
