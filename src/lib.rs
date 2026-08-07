use std::fmt;

use logos::Logos;

use lalrpop_util::lalrpop_mod;
use miette::{Diagnostic, NamedSource, SourceSpan};
use regex::Regex;
use thiserror::Error;
use tiny_bail::prelude::*;

pub enum LocalisationText {
    String(String),
    Highlight(String),
    Link(String),
    Goto(String),
}

#[derive(Default, Debug, Clone, PartialEq)]
pub enum LexicalError {
    #[default]
    InvalidToken,
}

///Token errors
#[derive(Error, Debug, Diagnostic)]
#[error("I came across an invalid token")]
struct TokenErrorReport {
    // The Source that we're gonna be printing snippets out of.
    // This can be a String if you don't have or care about file names.
    #[source_code]
    src: NamedSource<String>,
    // Snippets and highlights can be included in the diagnostic!
    #[label("Problem started here")]
    bad_bit: SourceSpan,
    #[help]
    advice: Option<String>,
}

#[derive(Logos, Clone, Debug, PartialEq)]
#[logos(error = LexicalError)]
pub enum Token {
    // Or regular expressions.
    #[regex("[a-zA-Z0-9]+", |lex| lex.slice().to_string())]
    String(String),

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

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

lalrpop_mod!(pub lib);

pub fn parse_localisation(file_name: &str, contents: &str) -> Vec<LocalisationText> {
    let mut lex = Token::lexer(&contents);

    let mut tokens = Vec::new();

    while let Some(tok) = lex.next() {
        let token = match tok {
            Ok(t) => t,
            Err(e) => match e {
                LexicalError::InvalidToken => {
                    let last: usize = tokens.last().map(|(_, _, x)| *x).unwrap_or_default();

                    handle_lexical_errors(file_name, e, contents, last);
                }
            },
        };
        let span = lex.span();
        tokens.push((span.start, token, span.end));
    }

    let localisation = r!(lib::LocalisationTextParser::new().parse(tokens));
    let localised = localisation.into_iter().fold(Vec::new(), |mut acc, x| {
        if let LocalisationText::String(ref next) = x
            && let Some(last) = acc.last_mut()
            && let LocalisationText::String(last_string) = last
        {
            last_string.push_str(&next);
        } else {
            acc.push(x);
        }
        acc
    });
    localised
}

/// Pretty prints the lexical error message to try and give the user
/// The clearest indication where the error likely is
fn handle_lexical_errors(
    file_name: &str,
    lexical_error: LexicalError,
    input: &str,
    last: usize,
) -> ! {
    // Needs to be in sync with actual skip tokens, unfortunately
    // cannot be done in const context as macros require string
    let skip_texts: [&str; 2] = [r"//[^\n\r]*", r"[\s\t\f]+"];
    match lexical_error {
        LexicalError::InvalidToken => {
            let regexes = skip_texts
                .into_iter()
                .flat_map(Regex::new)
                .collect::<Vec<_>>();

            let error = input
                .char_indices()
                .skip(last)
                .skip_while(|(_, c)| regexes.iter().any(|r| r.is_match(c.to_string().as_str())))
                .map(|(i, _)| i)
                .next()
                .unwrap();
            let problem = TokenErrorReport {
                src: NamedSource::new(file_name, input.to_string()),
                bad_bit: (error).into(),
                advice: Some("I don't have a definition for this word\nNote:File numbers are relative to the '#' sections".to_string()),
            };
            panic!("{:?}", miette::Error::new(problem));
        }
    }
}
