use thiserror::Error;

use crate::code::OpCode;

#[derive(Debug, Error, Clone, Copy)]
pub enum CodeError {
    #[error("failed to find a definition related to {0}")]
    FailedToLookupDefinitionFromOpCode(OpCode),
}
