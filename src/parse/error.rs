use crate::parse::SourceLocation;
use ariadne::{Color, Label, Report, ReportKind};

#[derive(Debug)]
pub(crate) enum ParseError<'s> {
    IndentationMismatch {
        expected_indentation: SourceLocation<'s>,
        actual_indentation: SourceLocation<'s>,
    },
    UnexpectedIndentation {
        indentation: SourceLocation<'s>,
    },
    ReservedSymbol {
        location: SourceLocation<'s>,
    },
    ExpectedSignatureStatement {
        location: SourceLocation<'s>,
    },
    ExpectedFunctionStatement {
        location: SourceLocation<'s>,
    },
    ExpectedSignatureOrFunction {
        location: SourceLocation<'s>,
    },
    ExpectedSignature {
        location: SourceLocation<'s>,
    },
    ExpectedFunction {
        location: SourceLocation<'s>,
    },
    ExpectedClosingParen {
        location: SourceLocation<'s>,
    },
    ExpectedEndOfLine {
        location: SourceLocation<'s>,
    },
    ExpectedEquals {
        location: SourceLocation<'s>,
    },
    ExpectedSignatureAssignmentRhs {
        location: SourceLocation<'s>,
    },
    ExpectedTo {
        location: SourceLocation<'s>,
    },
    ExpectedAsOrTo {
        location: SourceLocation<'s>,
    },
    ExpectedFunctionAssignmentRhs {
        location: SourceLocation<'s>,
    },
    ExpectedFromOrSignatureOrEndOfLine {
        location: SourceLocation<'s>,
    },
    ExpectedFromOrSignatureOrFunction {
        location: SourceLocation<'s>,
    },
    ExpectedUsingOrEndOfLine {
        location: SourceLocation<'s>,
    },
    ExpectedFrom {
        location: SourceLocation<'s>,
    },
    ExpectedUsingOrFunctionOrEndOfLine {
        location: SourceLocation<'s>,
    },
    ExpectedFunctionOrEndOfLine {
        location: SourceLocation<'s>,
    },
    GiveInSignatureContext {
        location: SourceLocation<'s>,
    },
    ConjureInFunctionContext {
        location: SourceLocation<'s>,
    },
}

fn simple_report<'s>(
    location: SourceLocation<'s>,
    message: &str,
) -> Report<'s, SourceLocation<'s>> {
    Report::build(ReportKind::Error, location)
        .with_label(
            Label::new(location)
                .with_color(Color::Cyan)
                .with_message("here"),
        )
        .with_message(message)
        .finish()
}

impl<'s> ParseError<'s> {
    pub(crate) fn report(self) -> Report<'s, SourceLocation<'s>> {
        match self {
            ParseError::IndentationMismatch {
                expected_indentation,
                actual_indentation,
            } => Report::build(ReportKind::Error, actual_indentation)
                .with_label(
                    Label::new(actual_indentation)
                        .with_color(Color::BrightRed)
                        .with_message("this indentation..."),
                )
                .with_label(
                    Label::new(expected_indentation)
                        .with_color(Color::Cyan)
                        .with_message("... should be like this"),
                )
                .with_message("indentation mismatch")
                .finish(),
            ParseError::UnexpectedIndentation { indentation } => {
                simple_report(indentation, "unexpected indentation")
            }
            ParseError::ReservedSymbol { location } => simple_report(
                location,
                "reserved symbol used as function or signature name",
            ),
            ParseError::ExpectedSignatureStatement { location } => {
                simple_report(location, "expected assignment or 'conjure' keyword")
            }
            ParseError::ExpectedFunctionStatement { location } => {
                simple_report(location, "expected assignment or 'give' keyword")
            }
            ParseError::ExpectedSignatureOrFunction { location } => {
                simple_report(location, "expected signature or function")
            }
            ParseError::ExpectedSignature { location } => {
                simple_report(location, "expected signature")
            }
            ParseError::ExpectedFunction { location } => {
                simple_report(location, "expected function")
            }
            ParseError::ExpectedClosingParen { location } => {
                simple_report(location, "expected ')'")
            }
            ParseError::ExpectedEndOfLine { location } => {
                simple_report(location, "expected end of line")
            }
            ParseError::ExpectedUsingOrEndOfLine { location } => {
                simple_report(location, "expected 'using' keyword or end of line")
            }
            ParseError::ExpectedEquals { location } => simple_report(location, "expected '='"),
            ParseError::ExpectedSignatureAssignmentRhs { location } => simple_report(
                location,
                "expected 'define', 'give', 'take' or 'conjure' keyword",
            ),
            ParseError::ExpectedFunctionAssignmentRhs { location } => simple_report(
                location,
                "expected 'define', 'give', 'take' or 'conjure' keyword",
            ),
            ParseError::ExpectedTo { location } => simple_report(location, "expected 'to' keyword"),
            ParseError::ExpectedAsOrTo { location } => {
                simple_report(location, "expected 'as' or 'to' keyword")
            }
            ParseError::ExpectedFromOrSignatureOrEndOfLine { location } => simple_report(
                location,
                "expected 'from' keyword, signature or end of line",
            ),
            ParseError::ExpectedFromOrSignatureOrFunction { location } => {
                simple_report(location, "expected 'from' keyword, signature or function")
            }
            ParseError::ExpectedFrom { location } => {
                simple_report(location, "expected 'from' keyword")
            }
            ParseError::ExpectedUsingOrFunctionOrEndOfLine { location } => simple_report(
                location,
                "expected 'using' keyword, function or end of line",
            ),
            ParseError::ExpectedFunctionOrEndOfLine { location } => {
                simple_report(location, "expected function or end of line")
            }
            ParseError::GiveInSignatureContext { location } => simple_report(
                location,
                "'give' statements cannot be used in signature contexts",
            ),
            ParseError::ConjureInFunctionContext { location } => simple_report(
                location,
                "'conjure' statements cannot be used in function contexts",
            ),
        }
    }
}
