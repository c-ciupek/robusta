use crate::transformation::JavaPath;
use syn::{LifetimeDef, Path};

#[derive(Clone)]
pub(crate) struct StructContext {
    pub(crate) struct_type: Path,
    pub(crate) struct_name: String,
    pub(crate) struct_lifetimes: Vec<LifetimeDef>,
    pub(crate) package: Option<JavaPath>,
}
