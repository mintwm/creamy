use std::{fs::File, io::BufWriter, path::Path};

use binrw::BinWrite;

use crate::BinaryPlugin;

impl BinaryPlugin {
    pub fn write_to_file(&self, path: impl AsRef<Path>) -> Result<(), Box<dyn std::error::Error>> {
        dbg!(self);
        let path = path.as_ref();
        println!("path: {}", path.display());
        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);
        self.write(&mut writer)?;
        Ok(())
    }
}
