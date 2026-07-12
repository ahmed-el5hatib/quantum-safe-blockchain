use crate::schema::QsbConfig;
use crate::CoreResult;

pub trait ConfigLoader: Send + Sync + fmt::Debug {
    fn load(&self) -> CoreResult<QsbConfig>;
    fn load_from_file(&self, path: &std::path::Path) -> CoreResult<QsbConfig>;
    fn load_from_env(&self) -> CoreResult<QsbConfig>;
    fn validate(&self, config: &QsbConfig) -> CoreResult<()>;
}

pub trait ConfigSource: Send + Sync + fmt::Debug {
    fn read(&self) -> CoreResult<Vec<u8>>;
}
