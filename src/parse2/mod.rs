#![allow(unused)]

pub(crate) use error::*;
pub(crate) use source::*;
pub(crate) use source_location::*;
use std::fmt::Debug;
use std::str::pattern::{DoubleEndedSearcher, Pattern, ReverseSearcher, Searcher};
pub(crate) use syntax_tree::*;

mod source;
mod syntax_tree;
mod source_location;
mod error;

#[allow(non_snake_case, reason = "used in type position")]
macro_rules! ParseResult {
    ($l:lifetime, $t:ident) => { Result<$t<$l>, ParseError<$l>>};
}

#[allow(non_snake_case, reason = "used in type position")]
macro_rules! LocationParseResult {
    ($l:lifetime, $t:ident) => { Result<(SourceLocation<$l>, $t<$l>), ParseError<$l>>};
    ($l:lifetime) => { Result<SourceLocation<$l>, ParseError<$l>>};
    ($t:ident) => { Result<(SourceLocation<'_>, $t<'_>), ParseError<'_>>};
    () => { Result<SourceLocation<'_>, ParseError<'_>>};
}

pub(crate) fn parse_file<'s>(source: &'s Source<'s>) -> ParseResult!['s, Context] {
    parse_context(source, &mut 0, None)
}

/// Returns true if there are more (non-empty) lines in the file. If this function returns true,
/// then `line_index` is the index of the next non-empty line. Else, it is `source.lines.len()`.
fn skip_empty_lines<'s>(source: &'s Source<'s>, line_index: &mut usize) -> bool {
    loop {
        let line = source.lines[*line_index];
        // remove leading whitespace
        let line = line.trim_start();
        // only whitespace or comment
        let line_is_empty = line.is_empty() || line.starts_with("#");
        let line_is_last = *line_index == source.lines.len();
        if line_is_last {
            break false;
        } else if !line_is_empty {
            break true;
        } else {
            *line_index += 1;
            continue;
        }
    }
}

fn parse_as(location: SourceLocation<'_>) -> LocationParseResult![] {
    if let Some(rest) = location.strip_prefix("as") {
        Ok(rest)
    } else {
        let location = location.truncate(2);
        Err(ParseError::ExpectedAs { location })
    }
}

fn parse_symbol(location: SourceLocation<'_>) -> LocationParseResult![SourceLocation] {
    let (symbol, location) = location.partition(is_symbol_char);
    if symbol.is_empty() {
        let location = location.truncate_to_word();
        Err(ParseError::ExpectedSignatureOrFunction { location })
    } else {
        Ok((location, symbol))
    }
}

fn parse_signature(location: SourceLocation<'_>) -> LocationParseResult![Signature] {
    if let Some(location) = location.strip_prefix('(') {
        match parse_symbol(location) {
            Ok((location, symbol)) => {
                if let Some(location) = location.strip_prefix(')') {
                    let signature = Signature {
                        with_parens: symbol.grow(1),
                        symbol,
                    };
                    Ok((location, signature))
                } else {
                    let location = location.truncate(1);
                    Err(ParseError::ExpectedClosingParen { location })
                }
            }
            Err(ParseError::ExpectedSignatureOrFunction { location }) => {
                let location = location.grow_start(1);
                Err(ParseError::ExpectedSignature { location })
            }
            Err(other) => Err(other),
        }
    } else {
        let location = location.truncate_to_word();
        Err(ParseError::ExpectedSignature { location })
    }
}

fn parse_signature_literal(location: SourceLocation<'_>) -> LocationParseResult![SignatureLiteral] {
    if let Some(location) = location.strip_prefix('\'') {
        match parse_signature(location) {
            Ok((location, signature)) => {
                if let Some(location) = location.strip_prefix('\'') {
                    let literal = SignatureLiteral::Explicit {
                        with_ticks: signature.with_parens.grow(1),
                        with_parens: signature.with_parens,
                        symbol: signature.symbol,
                    };
                    Ok((location, literal))
                } else {
                    let location = location.truncate(1);
                    Err(ParseError::ExpectedClosingTick { location })
                }
            }
            Err(ParseError::ExpectedSignature { location }) => {
                let location = location.grow_start(1);
                Err(ParseError::ExpectedSignatureLiteral { location })
            }
            Err(other) => Err(other),
        }
    } else {
        let location = location.truncate_to_word();
        Err(ParseError::ExpectedSignatureLiteral { location })
    }
}

fn parse_maybe_as_signature_literal<'s>(
    location: SourceLocation<'s>,
    implicit: Signature<'s>,
) -> LocationParseResult!['s, SignatureLiteral] {
    if let Some(location) = location.strip_prefix("as") {
        let location = location.trim_start();
        let (location, literal) = parse_signature_literal(location)?;
        Ok((location, literal))
    } else {
        Ok((location, SignatureLiteral::Implicit(implicit)))
    }
}

fn parse_give_statement<'s>(
    source: &'s Source<'s>,
    line_index: &'_ mut usize,
) -> ParseResult!['s, Statement] {
    let location = SourceLocation::full_line(source, *line_index);
    let location = location.trim();
    let location = location
        .strip_prefix("give")
        .expect("line should start with 'give'");
    let location = location.trim_start();

    if location.starts_with('(') {
        let (location, signature) = parse_signature(location)?;
        let location = location.trim_start();
        let (location, literal) = parse_maybe_as_signature_literal(location, signature)?;
        let location = location.trim_start();
        if !location.is_empty() {
            match literal {
                SignatureLiteral::Explicit { .. } => {
                    Err(ParseError::ExpectedEndOfLine { location })
                }
                SignatureLiteral::Implicit(_) => {
                    Err(ParseError::ExpectedAsOrEndOfLine { location })
                }
            }
        } else {
            Ok(Statement::GiveSignature(GiveSignature {
                signature,
                literal,
            }))
        }
    } else if location.starts_with(is_symbol_char) {
        todo!()
    } else {
        Err(ParseError::ExpectedSignatureOrFunction { location })
    }
}

fn is_symbol_char(char: char) -> bool {
    !char.is_whitespace() && char != '\'' && char != '(' && char != ')' && char != '='
}

fn parse_statement<'s>(
    source: &'s Source<'s>,
    line_index: &mut usize,
    indentation: SourceLocation<'s>,
) -> ParseResult!['s, Statement] {
    // caller checked indentation

    let location = SourceLocation::full_line(source, *line_index);
    let location = location.trim();
    let line = location.as_str();

    if line.starts_with("give") {
        parse_give_statement(source, line_index)
    } else if line.starts_with('(') || line.starts_with(is_symbol_char) {
        // parse_assignment_statement(source, line_index, indentation)
        todo!()
    } else {
        Err(ParseError::ExpectedStatement { location })
    }
}

fn parse_context<'s>(
    source: &'s Source<'s>,
    line_index: &mut usize,
    indentation: Option<SourceLocation<'s>>,
) -> ParseResult!['s, Context] {
    let mut statements = Vec::new();

    if skip_empty_lines(source, line_index) {
        // Use first non-empty line indentation as indentation for context.

        let line = source.lines[*line_index];

        let indentation_chars = line
            .find(|char: char| !char.is_whitespace())
            .expect("line_index should be the index of a non-empty line");
        let new_indentation = SourceLocation::new(source, *line_index, 0..indentation_chars);

        if let Some(indentation) = indentation {
            if !line.starts_with(indentation.as_str()) {
                return Err(ParseError::IndentationMismatch {
                    expected_indentation: indentation,
                    actual_indentation: new_indentation,
                });
            }
        }

        if new_indentation.len() == indentation.map_or(0, |indentation| indentation.len()) {
            return Err(ParseError::InsufficientIndentation {
                indentation: new_indentation,
            });
        }

        loop {
            let line = source.lines[*line_index];
            if !line.starts_with(new_indentation.as_str()) {
                // context indentation does not apply for this line
                // => assume the next line is outside the context (one indentation level less)
                // if this assumption is false (and the indentation is simply nonsensical), the
                // caller will return a ParseError::IndentationMismatch
                break;
            }

            statements.push(parse_statement(source, line_index, new_indentation)?);
            if !skip_empty_lines(source, line_index) {
                break;
            }
        }
    }

    Ok(Context(statements))
}
