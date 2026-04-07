use std::marker::PhantomData;

mod error;
mod indented;
mod source;
mod source_location;
mod source_location_lines;
mod syntax_tree;

use crate::parse::indented::{parse_indented, parse_with_indentation};
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

pub(crate) fn parse_file_as_function_context<'s>(
    source: &'s Source<'s>,
) -> ParseResult!['s, FunctionContext] {
    parse_function_context(SourceLocationLines::top_level(source)).map(|(rest, context)| {
        assert!(rest.is_empty());
        context
    })
}

#[allow(unused)]
pub(crate) fn parse_file_as_signature_context<'s>(
    source: &'s Source<'s>,
) -> ParseResult!['s, SignatureContext] {
    parse_signature_context(SourceLocationLines::top_level(source)).map(|(rest, context)| {
        assert!(rest.is_empty());
        context
    })
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

fn is_symbol_char(char: char) -> bool {
    !char.is_whitespace() && char != '(' && char != ')' && char != '='
}

fn parse_symbol(location: SourceLocation<'_>) -> LocationParseResult![SourceLocation] {
    const RESERVED_SYMBOLS: &[&str] = &["define", "take", "give", "conjure"];

    let (symbol, location) = location.partition(is_symbol_char);
    if symbol.is_empty() {
        let location = location.take_until_whitespace();
        Err(ParseError::ExpectedSignatureOrFunction { location })
    } else if RESERVED_SYMBOLS.contains(&symbol.as_str()) {
        Err(ParseError::ReservedSymbol { location: symbol })
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

fn parse_conjure_dependency(
    location: SourceLocation<'_>,
) -> LocationParseResult![ConjureDependency] {
    if location.starts_with('(') {
        let (location, signature) = parse_signature(location)?;
        Ok((location, ConjureDependency::Signature(signature)))
    } else if location.starts_with(is_symbol_char) {
        let (location, function) = parse_function(location)?;
        Ok((location, ConjureDependency::Function(function)))
    } else {
        let location = location.take_until_whitespace();
        Err(ParseError::ExpectedSignatureOrFunction { location })
    }
}

fn parse_conjure_dependencies(
    location: SourceLocation<'_>,
) -> LocationParseResult![ConjureDependencies] {
    if let Some(location) = location.strip_prefix("using") {
        let location = location.trim_start();
        let (mut location, first) = parse_conjure_dependency(location)?;
        let mut dependencies = vec![first];
        while !location.trim_start().is_empty() {
            location = location.trim_start();
            let (new_location, next) = parse_conjure_dependency(location)?;
            location = new_location;
            dependencies.push(next);
        }
        Ok((location, ConjureDependencies(dependencies)))
    } else {
        Ok((location, ConjureDependencies(Vec::new())))
    }
}

fn parse_maybe_as_signature<'s>(
    location: SourceLocation<'s>,
    implicit: Signature<'s>,
) -> LocationParseResult!['s, ForeignSignature] {
    if let Some(location) = location.strip_prefix("as") {
        let location = location.trim_start();
        let (location, signature) = parse_signature(location)?;
        Ok((location, ForeignSignature::Explicit(signature)))
    } else {
        Ok((location, ForeignSignature::Implicit(implicit)))
    }
}

fn parse_maybe_as_function<'s>(
    location: SourceLocation<'s>,
    implicit: Function<'s>,
) -> LocationParseResult!['s, ForeignFunction] {
    if let Some(location) = location.strip_prefix("as") {
        let location = location.trim_start();
        let (location, function) = parse_function(location)?;
        Ok((location, ForeignFunction::Explicit(function)))
    } else {
        Ok((location, ForeignFunction::Implicit(implicit)))
    }
}

fn parse_give_signature_statement<'s>(
    location: SourceLocation<'s>,
    statement_location: SourceLocation<'s>,
) -> ParseResult!['s, FunctionStatement] {
    let (location, signature) = parse_signature(location)?;
    let location = location.trim_start();
    if !location.is_empty() {
        Err(ParseError::ExpectedEndOfLine { location })
    } else {
        Ok(FunctionStatement::GiveSignature(GiveSignature {
            signature,
            location: statement_location,
        }))
    }
}

fn parse_give_function_statement<'s>(
    location: SourceLocation<'s>,
    statement_location: SourceLocation<'s>,
) -> ParseResult!['s, FunctionStatement] {
    let (location, function) = parse_function(location)?;
    let location = location.trim_start();
    if !location.is_empty() {
        Err(ParseError::ExpectedEndOfLine { location })
    } else {
        Ok(FunctionStatement::GiveFunction(GiveFunction {
            function,
            location: statement_location,
        }))
    }
}

fn parse_give_statement<'s>(
    location: SourceLocation<'s>,
    statement_location: SourceLocation<'s>,
) -> ParseResult!['s, FunctionStatement] {
    if location.starts_with('(') {
        parse_give_signature_statement(location, statement_location)
    } else if location.starts_with(is_symbol_char) {
        parse_give_function_statement(location, statement_location)
    } else {
        Err(ParseError::ExpectedSignatureOrFunction { location })
    }
}

fn parse_equals(location: SourceLocation<'_>) -> LocationParseResult![] {
    if let Some(rest) = location.strip_prefix("=") {
        Ok(rest)
    } else {
        let location = location.truncate(1);
        Err(ParseError::ExpectedEquals { location })
    }
}

/// location should not include the keyword and the whitespace that follows
fn parse_define_signature<'s>(
    location: SourceLocation<'s>,
    following_lines: SourceLocationLines<'s>,
) -> LinesParseResult!['s, SignatureAssignmentRhs] {
    if !location.is_empty() {
        return Err(ParseError::ExpectedEndOfLine { location });
    }
    let (following_lines, context) = parse_indented_signature_context(following_lines)?;
    let rhs = SignatureAssignmentRhs::Define(DefineSignature { context });
    Ok((following_lines, rhs))
}

/// location should not include the keyword and the whitespace that follows
fn parse_take_signature_or_take_signature_from<'s>(
    location: SourceLocation<'s>,
    implicit: Signature<'s>,
) -> ParseResult!['s, SignatureAssignmentRhs] {
    if location.is_empty() {
        let rhs = SignatureAssignmentRhs::Take(TakeSignature {
            phantom: PhantomData,
        });
        Ok(rhs)
    } else {
        let (location, foreign) = if let Some(location) = location.strip_prefix("from") {
            (location, ForeignSignature::Implicit(implicit))
        } else {
            let (location, signature) = match parse_signature(location) {
                Ok((location, signature)) => (location, signature),
                Err(ParseError::ExpectedSignature { location }) => {
                    return Err(ParseError::ExpectedFromOrSignatureOrEndOfLine { location });
                }
                Err(other) => return Err(other),
            };
            let location = location.trim_start();
            if let Some(location) = location.strip_prefix("from") {
                (location, ForeignSignature::Explicit(signature))
            } else {
                return Err(ParseError::ExpectedFrom { location });
            }
        };
        let location = location.trim_start();
        let (location, source) = parse_function(location)?;
        let location = location.trim_start();
        if location.is_empty() {
            let rhs = SignatureAssignmentRhs::TakeFrom(TakeSignatureFrom { foreign, source });
            Ok(rhs)
        } else {
            Err(ParseError::ExpectedEndOfLine { location })
        }
    }
}

/// location should not include the keyword and the whitespace that follows
fn parse_give_to_signature(location: SourceLocation<'_>) -> ParseResult![SignatureAssignmentRhs] {
    if location.starts_with('(') {
        let (location, signature) = parse_signature(location)?;
        let location = location.trim_start();
        let (location, foreign) = parse_maybe_as_signature(location, signature)?;
        let location = location.trim_start();
        if let Some(location) = location.strip_prefix("to") {
            let location = location.trim_start();
            let (location, source) = parse_signature(location)?;
            let location = location.trim_start();
            if location.is_empty() {
                let rhs =
                    SignatureAssignmentRhs::GiveSignatureToSignature(GiveSignatureToSignature {
                        signature,
                        foreign,
                        source,
                    });
                Ok(rhs)
            } else {
                Err(ParseError::ExpectedEndOfLine { location })
            }
        } else {
            // both 'as' and 'to' keywords are two chars long
            let location = location.truncate(2);
            match foreign {
                ForeignSignature::Explicit { .. } => Err(ParseError::ExpectedTo { location }),
                ForeignSignature::Implicit(_) => Err(ParseError::ExpectedAsOrTo { location }),
            }
        }
    } else if location.starts_with(is_symbol_char) {
        let (location, function) = parse_function(location)?;
        let location = location.trim_start();
        let (location, foreign) = parse_maybe_as_function(location, function)?;
        let location = location.trim_start();
        if let Some(location) = location.strip_prefix("to") {
            let location = location.trim_start();
            let (location, source) = parse_signature(location)?;
            let location = location.trim_start();
            if location.is_empty() {
                let rhs =
                    SignatureAssignmentRhs::GiveFunctionToSignature(GiveFunctionToSignature {
                        function,
                        foreign,
                        source,
                    });
                Ok(rhs)
            } else {
                Err(ParseError::ExpectedEndOfLine { location })
            }
        } else {
            // both 'as' and 'to' keywords are two chars long
            let location = location.truncate(2);
            match foreign {
                ForeignFunction::Explicit(_) => Err(ParseError::ExpectedTo { location }),
                ForeignFunction::Implicit(_) => Err(ParseError::ExpectedAsOrTo { location }),
            }
        }
    } else {
        let location = location.take_until_whitespace();
        Err(ParseError::ExpectedSignatureOrFunction { location })
    }
}

fn parse_signature_assignment_rhs<'s>(
    location: SourceLocation<'s>,
    following_lines: SourceLocationLines<'s>,
    implicit: Signature<'s>,
) -> LinesParseResult!['s, SignatureAssignmentRhs] {
    if let Some(location) = location.strip_prefix("define") {
        let location = location.trim_start();
        let (following_lines, rhs) = parse_define_signature(location, following_lines)?;
        Ok((following_lines, rhs))
    } else if let Some(location) = location.strip_prefix("take") {
        let location = location.trim_start();
        let rhs = parse_take_signature_or_take_signature_from(location, implicit)?;
        Ok((following_lines, rhs))
    } else if let Some(location) = location.strip_prefix("give") {
        let location = location.trim_start();
        let rhs = parse_give_to_signature(location)?;
        Ok((following_lines, rhs))
    } else {
        let location = location.take_until_whitespace();
        Err(ParseError::ExpectedSignatureAssignmentRhs { location })
    }
}

/// location should not include the keyword and the whitespace that follows
fn parse_define_function<'s>(
    location: SourceLocation<'s>,
    following_lines: SourceLocationLines<'s>,
) -> LinesParseResult!['s, FunctionAssignmentRhs] {
    if !location.is_empty() {
        return Err(ParseError::ExpectedEndOfLine { location });
    }
    let (following_lines, context) = parse_indented_function_context(following_lines)?;
    let rhs = FunctionAssignmentRhs::Define(DefineFunction { context });
    Ok((following_lines, rhs))
}

/// location should not include the keyword and the whitespace that follows
fn parse_take_function_or_take_function_from<'s>(
    location: SourceLocation<'s>,
    implicit: Function<'s>,
) -> ParseResult!['s, FunctionAssignmentRhs] {
    if location.starts_with('(') {
        let (location, signature) = parse_signature(location)?;
        let location = location.trim_start();
        if !location.is_empty() {
            return Err(ParseError::ExpectedEndOfLine { location });
        }
        let rhs = FunctionAssignmentRhs::Take(TakeFunction { signature });
        Ok(rhs)
    } else {
        let (location, foreign) = if let Some(location) = location.strip_prefix("from") {
            (location, ForeignFunction::Implicit(implicit))
        } else {
            let (location, function) = match parse_function(location) {
                Ok((location, function)) => (location, function),
                Err(ParseError::ExpectedFunction { location }) => {
                    return Err(ParseError::ExpectedFromOrSignatureOrFunction { location });
                }
                Err(other) => return Err(other),
            };
            let location = location.trim_start();
            if let Some(location) = location.strip_prefix("from") {
                (location, ForeignFunction::Explicit(function))
            } else {
                return Err(ParseError::ExpectedFrom { location });
            }
        };
        let location = location.trim_start();
        let (location, source) = parse_function(location)?;
        let location = location.trim_start();
        if location.is_empty() {
            let rhs = FunctionAssignmentRhs::TakeFrom(TakeFunctionFrom { foreign, source });
            Ok(rhs)
        } else {
            Err(ParseError::ExpectedEndOfLine { location })
        }
    }
}

/// location should not include the keyword and the whitespace that follows
fn parse_give_to_function(location: SourceLocation<'_>) -> ParseResult![FunctionAssignmentRhs] {
    if location.starts_with('(') {
        let (location, signature) = parse_signature(location)?;
        let location = location.trim_start();
        let (location, foreign) = parse_maybe_as_signature(location, signature)?;
        let location = location.trim_start();
        if let Some(location) = location.strip_prefix("to") {
            let location = location.trim_start();
            let (location, source) = parse_function(location)?;
            let location = location.trim_start();
            if location.is_empty() {
                let rhs = FunctionAssignmentRhs::GiveSignatureToFunction(GiveSignatureToFunction {
                    signature,
                    foreign,
                    source,
                });
                Ok(rhs)
            } else {
                Err(ParseError::ExpectedEndOfLine { location })
            }
        } else {
            // both 'as' and 'to' keywords are two chars long
            let location = location.truncate(2);
            match foreign {
                ForeignSignature::Explicit { .. } => Err(ParseError::ExpectedTo { location }),
                ForeignSignature::Implicit(_) => Err(ParseError::ExpectedAsOrTo { location }),
            }
        }
    } else if location.starts_with(is_symbol_char) {
        let (location, function) = parse_function(location)?;
        let location = location.trim_start();
        let (location, foreign) = parse_maybe_as_function(location, function)?;
        let location = location.trim_start();
        if let Some(location) = location.strip_prefix("to") {
            let location = location.trim_start();
            let (location, source) = parse_function(location)?;
            let location = location.trim_start();
            if location.is_empty() {
                let rhs = FunctionAssignmentRhs::GiveFunctionToFunction(GiveFunctionToFunction {
                    function,
                    foreign,
                    source,
                });
                Ok(rhs)
            } else {
                Err(ParseError::ExpectedEndOfLine { location })
            }
        } else {
            // both 'as' and 'to' keywords are two chars long
            let location = location.truncate(2);
            match foreign {
                ForeignFunction::Explicit(_) => Err(ParseError::ExpectedTo { location }),
                ForeignFunction::Implicit(_) => Err(ParseError::ExpectedAsOrTo { location }),
            }
        }
    } else {
        let location = location.take_until_whitespace();
        Err(ParseError::ExpectedSignatureOrFunction { location })
    }
}

fn parse_function_assignment_rhs<'s>(
    location: SourceLocation<'s>,
    following_lines: SourceLocationLines<'s>,
    implicit: Function<'s>,
) -> LinesParseResult!['s, FunctionAssignmentRhs] {
    if let Some(location) = location.strip_prefix("define") {
        let location = location.trim_start();
        let (following_lines, rhs) = parse_define_function(location, following_lines)?;
        Ok((following_lines, rhs))
    } else if let Some(location) = location.strip_prefix("take") {
        let location = location.trim_start();
        let rhs = parse_take_function_or_take_function_from(location, implicit)?;
        Ok((following_lines, rhs))
    } else if let Some(location) = location.strip_prefix("give") {
        let location = location.trim_start();
        let rhs = parse_give_to_function(location)?;
        Ok((following_lines, rhs))
    } else {
        let location = location.take_until_whitespace();
        Err(ParseError::ExpectedFunctionAssignmentRhs { location })
    }
}

fn parse_signature_assignment_statement<'s>(
    location: SourceLocation<'s>,
    following_lines: SourceLocationLines<'s>,
) -> LinesParseResult!['s, SignatureAssignment] {
    let statement_location = location.trim();
    let (location, lhs) = parse_signature(location)?;
    let location = location.trim_start();
    let location = parse_equals(location)?;
    let location = location.trim_start();
    let (following_lines, rhs) = parse_signature_assignment_rhs(location, following_lines, lhs)?;
    let statement = SignatureAssignment {
        lhs,
        rhs,
        location: statement_location,
    };
    Ok((following_lines, statement))
}

fn parse_function_assignment_statement<'s>(
    location: SourceLocation<'s>,
    following_lines: SourceLocationLines<'s>,
) -> LinesParseResult!['s, FunctionAssignment] {
    let statement_location = location.trim();
    let (location, lhs) = parse_function(location)?;
    let location = location.trim_start();
    let location = parse_equals(location)?;
    let location = location.trim_start();
    let (following_lines, rhs) = parse_function_assignment_rhs(location, following_lines, lhs)?;
    let statement = FunctionAssignment {
        lhs,
        rhs,
        location: statement_location,
    };
    Ok((following_lines, statement))
}

fn parse_conjure_statement<'s>(
    location: SourceLocation<'s>,
    statement_location: SourceLocation<'s>,
) -> ParseResult!['s, SignatureStatement] {
    let (location, signature) = parse_signature(location)?;
    let location = location.trim_start();
    let (location, function) = if location.starts_with("using") || location.is_empty() {
        (location, None)
    } else {
        let (location, function) = parse_function(location)?;
        let location = location.trim_start();
        (location, Some(function))
    };
    let (location, dependencies) = parse_conjure_dependencies(location)?;
    if !location.is_empty() {
        if dependencies.0.is_empty() {
            Err(ParseError::ExpectedUsingOrFunctionOrEndOfLine { location })
        } else {
            Err(ParseError::ExpectedFunctionOrEndOfLine { location })
        }
    } else if let Some(function) = function {
        Ok(SignatureStatement::ConjureFunction(ConjureFunction {
            function,
            signature,
            dependencies,
            location: statement_location,
        }))
    } else {
        Ok(SignatureStatement::ConjureSignature(ConjureSignature {
            signature,
            dependencies,
            location: statement_location,
        }))
    }
}

fn parse_signature_statement(
    location: SourceLocationLines<'_>,
) -> LinesParseResult![SignatureStatement] {
    let line = location.first().expect("should not be empty");
    let line = line.trim_start();
    let statement_location = line.trim();

    let location = location.advance().expect("should not be empty");
    if let Some(line) = line.take_prefix("give") {
        Err(ParseError::GiveInSignatureContext { location: line })
    } else if let Some(line) = line.strip_prefix("conjure") {
        let line = line.trim_start();
        let statement = parse_conjure_statement(line, statement_location)?;
        Ok((location, statement))
    } else if line.starts_with('(') {
        let (location, statement) = parse_signature_assignment_statement(line, location)?;
        Ok((location, SignatureStatement::SignatureAssignment(statement)))
    } else if line.starts_with(is_symbol_char) {
        let (location, statement) = parse_function_assignment_statement(line, location)?;
        Ok((location, SignatureStatement::FunctionAssignment(statement)))
    } else {
        Err(ParseError::ExpectedSignatureStatement {
            location: line.trim(),
        })
    }
}

fn parse_function_statement(
    location: SourceLocationLines<'_>,
) -> LinesParseResult![FunctionStatement] {
    let line = location.first().expect("should not be empty");
    let line = line.trim_start();
    let statement_location = line.trim();

    let location = location.advance().expect("should not be empty");
    if let Some(line) = line.take_prefix("conjure") {
        Err(ParseError::ConjureInFunctionContext { location: line })
    } else if let Some(line) = line.strip_prefix("give") {
        let line = line.trim_start();
        let statement = parse_give_statement(line, statement_location)?;
        Ok((location, statement))
    } else if line.starts_with('(') {
        let (location, statement) = parse_signature_assignment_statement(line, location)?;
        Ok((location, FunctionStatement::SignatureAssignment(statement)))
    } else if line.starts_with(is_symbol_char) {
        let (location, statement) = parse_function_assignment_statement(line, location)?;
        Ok((location, FunctionStatement::FunctionAssignment(statement)))
    } else {
        Err(ParseError::ExpectedFunctionStatement {
            location: line.trim(),
        })
    }
}

fn parse_indented_function_context(
    location: SourceLocationLines<'_>,
) -> LinesParseResult![FunctionContext] {
    parse_indented(location, parse_function_context, || FunctionContext {
        trace: false,
        statements: Vec::new(),
    })
}

fn parse_indented_signature_context(
    location: SourceLocationLines<'_>,
) -> LinesParseResult![SignatureContext] {
    parse_indented(location, parse_signature_context, || SignatureContext {
        trace: false,
        statements: Vec::new(),
    })
}

enum StatementOrTrace<T> {
    Statement(T),
    Trace,
}

fn parse_function_statement_or_trace(
    location: SourceLocationLines<'_>,
) -> Result<
    (
        SourceLocationLines<'_>,
        StatementOrTrace<FunctionStatement<'_>>,
    ),
    ParseError<'_>,
> {
    if location
        .first()
        .expect("should not be empty")
        .trim()
        .as_str()
        == "TRACE"
    {
        Ok((
            location.advance().expect("should not be empty"),
            StatementOrTrace::Trace,
        ))
    } else {
        let (location, statement) = parse_function_statement(location)?;
        Ok((location, StatementOrTrace::Statement(statement)))
    }
}

fn parse_signature_statement_or_trace(
    location: SourceLocationLines<'_>,
) -> Result<
    (
        SourceLocationLines<'_>,
        StatementOrTrace<SignatureStatement<'_>>,
    ),
    ParseError<'_>,
> {
    if location
        .first()
        .expect("should not be empty")
        .trim()
        .as_str()
        == "TRACE"
    {
        Ok((
            location.advance().expect("should not be empty"),
            StatementOrTrace::Trace,
        ))
    } else {
        let (location, statement) = parse_signature_statement(location)?;
        Ok((location, StatementOrTrace::Statement(statement)))
    }
}

fn parse_function_context(location: SourceLocationLines<'_>) -> LinesParseResult![FunctionContext] {
    let (location, statements) = parse_with_indentation(location, parse_function_statement_or_trace)?;
    let mut trace = false;
    let statements = statements.into_iter().filter_map(|statement_or_trace| {
        match statement_or_trace {
            StatementOrTrace::Statement(statement) => Some(statement),
            StatementOrTrace::Trace => {
                trace = true;
                None
            }
        }
    }).collect();
    Ok((location, FunctionContext { trace, statements }))
}

fn parse_signature_context(
    location: SourceLocationLines<'_>,
) -> LinesParseResult![SignatureContext] {
    let (location, statements) = parse_with_indentation(location, parse_signature_statement_or_trace)?;
    let mut trace = false;
    let statements = statements.into_iter().filter_map(|statement_or_trace| {
        match statement_or_trace {
            StatementOrTrace::Statement(statement) => Some(statement),
            StatementOrTrace::Trace => {
                trace = true;
                None
            }
        }
    }).collect();
    Ok((location, SignatureContext { trace, statements }))
}
