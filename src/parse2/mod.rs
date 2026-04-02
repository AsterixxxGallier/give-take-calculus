#![allow(unused)]

use std::fmt::Debug;
use std::str::pattern::{DoubleEndedSearcher, Pattern, ReverseSearcher, Searcher};

mod error;
mod source;
mod source_location;
mod source_location_lines;
mod syntax_tree;

pub(crate) use error::*;
pub(crate) use source::*;
pub(crate) use source_location::*;
pub(crate) use source_location_lines::*;
pub(crate) use syntax_tree::*;

#[allow(non_snake_case, reason = "used in type position")]
macro_rules! ParseResult {
    ($l:lifetime, $t:ident) => { Result<$t<$l>, ParseError<$l>>};
    ($t:ident) => { Result<$t<'_>, ParseError<'_>>};
}

#[allow(non_snake_case, reason = "used in type position")]
macro_rules! LocationParseResult {
    ($l:lifetime, $t:ident) => { Result<(SourceLocation<$l>, $t<$l>), ParseError<$l>>};
    ($l:lifetime) => { Result<SourceLocation<$l>, ParseError<$l>>};
    ($t:ident) => { Result<(SourceLocation<'_>, $t<'_>), ParseError<'_>>};
    () => { Result<SourceLocation<'_>, ParseError<'_>>};
}

#[allow(non_snake_case, reason = "used in type position")]
macro_rules! LinesParseResult {
    ($l:lifetime, $t:ident) => { Result<(SourceLocationLines<$l>, $t<$l>), ParseError<$l>>};
    ($l:lifetime) => { Result<SourceLocationLines<$l>, ParseError<$l>>};
    ($t:ident) => { Result<(SourceLocationLines<'_>, $t<'_>), ParseError<'_>>};
    () => { Result<SourceLocationLines<'_>, ParseError<'_>>};
}

pub(crate) fn parse_file<'s>(source: &'s Source<'s>) -> ParseResult!['s, Context] {
    parse_context(SourceLocationLines::top_level(source)).map(|(rest, context)| context)
}

/// If this returns `None`, then all lines in `location` were empty.
/// If this returns `Some(new_location)`, `new_location` is non-empty and starts with a non-empty
/// line.
fn skip_empty_lines(mut location: SourceLocationLines) -> Option<SourceLocationLines> {
    while let Some(line) = location.first() {
        // remove leading whitespace
        let line = line.trim_start();
        // only whitespace or comment
        let line_is_empty = line.is_empty() || line.starts_with("#");
        if !line_is_empty {
            return Some(location);
        } else {
            // if location were empty, we wouldn't have entered the while loop body
            location = location.advance().expect("shouldn't be empty");
        }
    }
    None
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
        let location = location.take_until_whitespace();
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
        let location = location.take_until_whitespace();
        Err(ParseError::ExpectedSignature { location })
    }
}

fn parse_function(location: SourceLocation<'_>) -> LocationParseResult![Function] {
    match parse_symbol(location) {
        Ok((location, symbol)) => {
            let function = Function { symbol };
            Ok((location, function))
        }
        Err(ParseError::ExpectedSignatureOrFunction { location }) => {
            Err(ParseError::ExpectedFunction { location })
        }
        Err(other) => Err(other),
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
        let location = location.take_until_whitespace();
        Err(ParseError::ExpectedSignatureLiteral { location })
    }
}

fn parse_function_literal(location: SourceLocation<'_>) -> LocationParseResult![FunctionLiteral] {
    if let Some(location) = location.strip_prefix('\'') {
        match parse_function(location) {
            Ok((location, function)) => {
                if let Some(location) = location.strip_prefix('\'') {
                    let literal = FunctionLiteral::Explicit {
                        with_ticks: function.symbol.grow(1),
                        symbol: function.symbol,
                    };
                    Ok((location, literal))
                } else {
                    let location = location.truncate(1);
                    Err(ParseError::ExpectedClosingTick { location })
                }
            }
            Err(ParseError::ExpectedFunction { location }) => {
                let location = location.grow_start(1);
                Err(ParseError::ExpectedFunctionLiteral { location })
            }
            Err(other) => Err(other),
        }
    } else {
        let location = location.take_until_whitespace();
        Err(ParseError::ExpectedFunctionLiteral { location })
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

fn parse_maybe_as_function_literal<'s>(
    location: SourceLocation<'s>,
    implicit: Function<'s>,
) -> LocationParseResult!['s, FunctionLiteral] {
    if let Some(location) = location.strip_prefix("as") {
        let location = location.trim_start();
        let (location, literal) = parse_function_literal(location)?;
        Ok((location, literal))
    } else {
        Ok((location, FunctionLiteral::Implicit(implicit)))
    }
}

fn parse_give_signature_statement(location: SourceLocation<'_>) -> ParseResult![Statement] {
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
        Ok(Statement::GiveSignature(GiveSignature { signature, literal }))
    }
}

fn parse_give_function_statement(location: SourceLocation<'_>) -> ParseResult![Statement] {
    let (location, function) = parse_function(location)?;
    let location = location.trim_start();
    let (location, literal) = parse_maybe_as_function_literal(location, function)?;
    let location = location.trim_start();
    if !location.is_empty() {
        match literal {
            FunctionLiteral::Explicit { .. } => {
                Err(ParseError::ExpectedEndOfLine { location })
            }
            FunctionLiteral::Implicit(_) => {
                Err(ParseError::ExpectedAsOrEndOfLine { location })
            }
        }
    } else {
        Ok(Statement::GiveFunction(GiveFunction { function, literal }))
    }
}

fn parse_give_statement(location: SourceLocationLines<'_>) -> LinesParseResult![Statement] {
    let line = location.first().expect("should not be empty");
    let line = line.trim_start();
    let line = line
        .strip_prefix("give")
        .expect("line should start with 'give'");
    let line = line.trim_start();

    if line.starts_with('(') {
        let statement = parse_give_signature_statement(line)?;
        Ok((location.advance().expect("should not be empty"), statement))
    } else if line.starts_with(is_symbol_char) {
        let statement = parse_give_function_statement(line)?;
        Ok((location.advance().expect("should not be empty"), statement))
    } else {
        Err(ParseError::ExpectedSignatureOrFunction { location: line })
    }
}

fn is_symbol_char(char: char) -> bool {
    !char.is_whitespace() && char != '\'' && char != '(' && char != ')' && char != '='
}

fn parse_statement(mut location: SourceLocationLines<'_>) -> LinesParseResult![Statement] {
    let line = location.first().expect("should not be empty");
    // caller (parse_context) handled indentation, which we can safely trim away
    let line = line.trim_start();

    if line.starts_with("give") {
        parse_give_statement(location)
    } else if line.starts_with('(') || line.starts_with(is_symbol_char) {
        // parse_assignment_statement(source, line_index, indentation)
        todo!()
    } else {
        Err(ParseError::ExpectedStatement { location: line })
    }
}

fn parse_context(mut location: SourceLocationLines<'_>) -> LinesParseResult![Context] {
    let mut statements = Vec::new();

    while let Some(new_location) = skip_empty_lines(location) {
        location = new_location;

        // skip_empty_lines shouldn't return Some(new_location) with empty new_location
        let line = new_location.first().expect("should not be empty");

        let indentation = line.take_while_whitespace();
        if let Some(reference_indentation) = location.reference_indentation {
            if !line.starts_with(reference_indentation.as_str()) {
                // indentation has been reduced
                // => assume the next line is outside the context (one indentation level less)
                break;
            }
            if indentation.as_str() != reference_indentation.as_str() {
                // indentation has changed in another way
                return Err(ParseError::IndentationMismatch {
                    expected_indentation: reference_indentation,
                    actual_indentation: indentation,
                });
            }
        } else {
            if !indentation.is_empty() {
                return Err(ParseError::UnexpectedIndentation { indentation });
            }
        }

        let (new_location, statement) = parse_statement(location)?;
        location = new_location;
        statements.push(statement);
    }

    Ok((location, Context(statements)))
}
