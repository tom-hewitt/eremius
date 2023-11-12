use crate::{preprocessor::PreProcessError, resolver::ResolveError};

#[derive(Debug)]
pub enum AssemblyError {
    PreProcessError(PreProcessError),
    ResolveError(ResolveError),
}

impl From<PreProcessError> for AssemblyError {
    fn from(value: PreProcessError) -> AssemblyError {
        AssemblyError::PreProcessError(value)
    }
}

impl From<ResolveError> for AssemblyError {
    fn from(value: ResolveError) -> AssemblyError {
        AssemblyError::ResolveError(value)
    }
}
