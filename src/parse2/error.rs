use crate::parse2::SourceLocation;
use ariadne::{Color, Label, Report, ReportKind};

#[derive(Debug)]
pub(crate) enum ParseError<'s> {
    IndentationMismatch {
        expected_indentation: SourceLocation<'s>,
        actual_indentation: SourceLocation<'s>,
    },
    InsufficientIndentation {
        indentation: SourceLocation<'s>,
    },
    UnexpectedIndentation { indentation: SourceLocation<'s> },
    ExpectedStatement {
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
    ExpectedSignatureLiteral {
        location: SourceLocation<'s>,
    },
    ExpectedFunctionLiteral {
        location: SourceLocation<'s>,
    },
    ExpectedClosingParen {
        location: SourceLocation<'s>,
    },
    ExpectedClosingTick {
        location: SourceLocation<'s>,
    },
    ExpectedEndOfLine {
        location: SourceLocation<'s>,
    },
    ExpectedAsOrEndOfLine {
        location: SourceLocation<'s>,
    },
    ExpectedEquals {
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
            ParseError::IndentationMismatch { expected_indentation, actual_indentation } => {
                Report::build(ReportKind::Error, actual_indentation)
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
                    .finish()
            }
            ParseError::InsufficientIndentation { indentation } => {
                simple_report(indentation, "insufficient indentation")
            }
            ParseError::UnexpectedIndentation { indentation } => {
                simple_report(indentation, "unexpected indentation")
            }
            ParseError::ExpectedStatement { location } => {
                simple_report(location, "expected statement")
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
            ParseError::ExpectedSignatureLiteral { location } => {
                simple_report(location, "expected signature literal")
            }
            ParseError::ExpectedFunctionLiteral { location } => {
                simple_report(location, "expected function literal")
            }
            ParseError::ExpectedClosingParen { location } => {
                simple_report(location, "expected closing parenthesis")
            }
            ParseError::ExpectedClosingTick { location } => {
                simple_report(location, "expected single quotation mark")
            }
            ParseError::ExpectedEndOfLine { location } => {
                simple_report(location, "expected end of line")
            }
            ParseError::ExpectedAsOrEndOfLine { location } => {
                simple_report(location, "expected 'as' keyword or end of line")
            }
            ParseError::ExpectedEquals { location } => {
                simple_report(location, "expected equals sign")
            }
        }
    }
}
