use crate::lexer::error::LexerError;

pub(crate) fn is_letter(ch: u8) -> bool {
    ch.is_ascii_lowercase() || ch.is_ascii_uppercase() || ch == b'_'
}

pub(crate) fn is_digit(ch: u8) -> bool {
    ch.is_ascii_digit()
}

pub(crate) trait LexerCharExt {
    fn as_str(&self) -> Result<&str, LexerError>;
}

impl LexerCharExt for [u8] {
    fn as_str(&self) -> Result<&str, LexerError> {
        str::from_utf8(self).map_err(Into::into)
    }
}
