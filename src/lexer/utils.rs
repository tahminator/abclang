use crate::lexer::error::LexerError;

pub(crate) fn is_letter(ch: u8) -> bool {
    return b'a' <= ch && ch <= b'z' || b'A' <= ch && ch <= b'Z' || ch == b'_';
}

pub(crate) fn is_digit(ch: u8) -> bool {
    return b'0' <= ch && ch <= b'9';
}

pub(crate) trait LexerCharExt {
    fn as_str(&self) -> Result<&str, LexerError>;
}

impl LexerCharExt for [u8] {
    fn as_str(&self) -> Result<&str, LexerError> {
        str::from_utf8(self).map_err(Into::into)
    }
}
