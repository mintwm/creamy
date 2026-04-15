use compiler_utils::strpool::{StringId, StringPool};

pub trait StringPoolExt {
    fn intern(&self, pool: &mut StringPool) -> StringId;
}

impl StringPoolExt for String {
    fn intern(&self, pool: &mut StringPool) -> StringId {
        pool.get_id(self)
    }
}

impl StringPoolExt for &str {
    fn intern(&self, pool: &mut StringPool) -> StringId {
        pool.get_id(self)
    }
}
