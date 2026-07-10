use thiserror::Error;

#[derive(Debug, Error)]
pub enum EvaluateError {
    #[error("expected return but no value attached")]
    ExpectedReturnButNoValueAttached,
}
