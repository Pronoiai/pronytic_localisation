use logos::Logos;

use tiny_bail::prelude::*;

pub enum LocalisationText {
    String(String),
    Highlight(String),
    Link(String),
    Goto(String),
}

#[derive(Logos, Debug, PartialEq)]
enum Token {
    // Or regular expressions.
    #[regex("[a-zA-Z0-9]+")]
    String,

    // Tokens can be literal strings, of any length.
    #[token("{")]
    OpeningCurly,

    #[token("}")]
    ClosingCurly,

    #[token("[")]
    OpeningSquare,

    #[token("]")]
    ClosingSquare,

    #[token("GoTo")]
    Goto,

    #[token("Highlight")]
    Highlight,

    #[token("Link")]
    Link,
}

pub fn parse_localisation(input: String) -> Vec<LocalisationText> {
    let mut lex = Token::lexer(&input);

    Vec::new()
}
