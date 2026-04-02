use crate::parse2::SourceLocation;

pub(crate) enum ParseError<'s> {
    IndentationMismatch {
        expected_indentation: SourceLocation<'s>,
        actual_indentation: SourceLocation<'s>,
    },
    InsufficientIndentation {
        indentation: SourceLocation<'s>,
    },
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
    ExpectedAs {
        location: SourceLocation<'s>,
    },
    ExpectedEndOfLine {
        location: SourceLocation<'s>,
    },
    ExpectedAsOrEndOfLine {
        location: SourceLocation<'s>,
    },
}
