use compiler_utils::strpool::{StringId, StringPool};

pub trait StringPoolExt {
    fn from_pool(&self, pool: &mut StringPool) -> StringId;
}

impl StringPoolExt for String {
    fn from_pool(&self, pool: &mut StringPool) -> StringId {
        pool.get_id(self)
    }
}

impl StringPoolExt for &str {
    fn from_pool(&self, pool: &mut StringPool) -> StringId {
        pool.get_id(self)
    }
}
