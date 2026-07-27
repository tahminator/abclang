use interpreter::lexer::{Lexer, TokenType};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Clone, Copy)]
pub enum Category {
    Keyword = 0,
    Number,
    String,
    Operator,
    Punctuation,
    Ident,
    Comment,
    Illegal,
}

fn category_for(typ: TokenType) -> Option<Category> {
    use TokenType::*;
    Some(match typ {
        Function | Let | True | False | If | Else | Return | For | In => Category::Keyword,
        Int | Float => Category::Number,
        String => Category::String,
        Assign | Plus | Minus | Bang | Asterisk | Slash | Lt | Gt | Eq | NotEq => {
            Category::Operator
        }
        Comma | Semicolon | LParen | RParen | LBrace | RBrace | LBracket | RBracket | Colon
        | Dot => Category::Punctuation,
        Ident => Category::Ident,
        Illegal => Category::Illegal,
        Eof => return None,
    })
}

type Start = usize;
type End = usize;
type Cat = u32;
struct Span(Start, End, Cat);

/// Vec<u32> should be read in triplicates (start, end, cat)
pub fn tokenize(input: &str) -> Vec<u32> {
    let mut spans: Vec<Span> = Vec::new();

    let mut lexer = Lexer::new(input);

    while let Ok(token) = lexer.next_token() {
        if token.typ == TokenType::Eof {
            break;
        }
        if let Some(cat) = category_for(token.typ) {
            spans.push(Span(lexer.start(), lexer.end(), cat as u32));
        }
    }

    for &(start, end) in lexer.comments() {
        spans.push(Span(start, end, Category::Comment as u32));
    }

    let mut out = Vec::with_capacity(spans.len() * 3);
    for Span(start, end, cat) in spans {
        out.push(start as u32);
        out.push(end as u32);
        out.push(cat);
    }
    out
}
