use std::{fs::File, path::Path};

use binrw::BinRead;

use crate::BinaryPlugin;

impl BinaryPlugin {
    pub fn load_from_file(path: impl AsRef<Path>) -> Result<Self, Box<dyn std::error::Error>> {
        let path = path.as_ref();
        let mut file = File::open(path)?;
        Ok(Self::read(&mut file)?)
    }
}
