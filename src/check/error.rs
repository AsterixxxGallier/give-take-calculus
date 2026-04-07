use crate::check::format::{Format, IndentingFormatter};
use crate::check::{FunctionValue, KnownFunctionValue, KnownSignatureValue, Resolver, SignatureValue, UnknownFunctionValue, UnknownSignatureValue};
use crate::parse::{ForeignFunction, ForeignSignature, Function, Signature, SourceLocation};
use ariadne::{Color, Label, Report, ReportKind};
use std::io;
use std::io::Write;

#[allow(unused)]
pub(crate) enum CheckError<'s> {
    CannotResolveSignature {
        signature: Signature<'s>,
    },
    CannotResolveFunction {
        function: Function<'s>,
    },
    CannotGiveSignatureToUnknownSignature {
        statement: SourceLocation<'s>,
        source: Signature<'s>,
        source_value: UnknownSignatureValue<'s>,
    },
    CannotGiveFunctionToUnknownSignature {
        statement: SourceLocation<'s>,
        source: Signature<'s>,
        source_value: UnknownSignatureValue<'s>,
    },
    CannotGiveSignatureToUnknownFunction {
        statement: SourceLocation<'s>,
        source: Function<'s>,
        source_value: UnknownFunctionValue<'s>,
    },
    CannotGiveFunctionToUnknownFunction {
        statement: SourceLocation<'s>,
        source: Function<'s>,
        source_value: UnknownFunctionValue<'s>,
    },
    CannotTakeSignatureFromUnknownFunction {
        statement: SourceLocation<'s>,
        source: Function<'s>,
        source_value: UnknownFunctionValue<'s>,
    },
    CannotTakeFunctionFromUnknownFunction {
        statement: SourceLocation<'s>,
        source: Function<'s>,
        source_value: UnknownFunctionValue<'s>,
    },
    CannotResolveGivenSignature {
        statement: SourceLocation<'s>,
        source: Function<'s>,
        source_value: KnownFunctionValue<'s>,
        foreign: ForeignSignature<'s>,
    },
    CannotResolveGivenFunction {
        statement: SourceLocation<'s>,
        source: Function<'s>,
        source_value: KnownFunctionValue<'s>,
        foreign: ForeignFunction<'s>,
    },
    CannotResolveTakenSignatureOfSignature {
        statement: SourceLocation<'s>,
        source: Signature<'s>,
        source_value: KnownSignatureValue<'s>,
        foreign: ForeignSignature<'s>,
    },
    CannotResolveTakenFunctionOfSignature {
        statement: SourceLocation<'s>,
        source: Signature<'s>,
        source_value: KnownSignatureValue<'s>,
        foreign: ForeignFunction<'s>,
    },
    CannotResolveTakenSignatureOfFunction {
        statement: SourceLocation<'s>,
        source: Function<'s>,
        source_value: KnownFunctionValue<'s>,
        foreign: ForeignSignature<'s>,
    },
    CannotResolveTakenFunctionOfFunction {
        statement: SourceLocation<'s>,
        source: Function<'s>,
        source_value: KnownFunctionValue<'s>,
        foreign: ForeignFunction<'s>,
    },
    TakenSignatureDependenciesNotProvided {
        statement: SourceLocation<'s>,
        source: Function<'s>,
        source_value: KnownFunctionValue<'s>,
        foreign: ForeignSignature<'s>,
    },
    TakenFunctionDependenciesNotProvided {
        statement: SourceLocation<'s>,
        source: Function<'s>,
        source_value: KnownFunctionValue<'s>,
        foreign: ForeignFunction<'s>,
    },
    FunctionGivenToSignatureDoesNotHaveExpectedSignature {
        statement: SourceLocation<'s>,
        function: Function<'s>,
        function_value: FunctionValue<'s>,
        foreign: ForeignFunction<'s>,
        source: Signature<'s>,
        source_value: KnownSignatureValue<'s>,
        expected_signature_value: SignatureValue<'s>,
    },
    FunctionGivenToFunctionDoesNotHaveExpectedSignature {
        statement: SourceLocation<'s>,
        function: Function<'s>,
        function_value: FunctionValue<'s>,
        foreign: ForeignFunction<'s>,
        source: Function<'s>,
        source_value: KnownFunctionValue<'s>,
        expected_signature_value: SignatureValue<'s>,
    },
    CannotGiveSignatureTwice {
        signature: Signature<'s>,
        statement: SourceLocation<'s>,
    },
    CannotGiveTwoSignaturesWithIdenticalName {
        signature: Signature<'s>,
        statement: SourceLocation<'s>,
    },
    CannotGiveFunctionTwice {
        function: Function<'s>,
        statement: SourceLocation<'s>,
    },
    CannotGiveTwoFunctionsWithIdenticalName {
        function: Function<'s>,
        statement: SourceLocation<'s>,
    },
    CannotConjureTwoSignaturesWithIdenticalName {
        signature: Signature<'s>,
        statement: SourceLocation<'s>,
    },
    RepeatedSignatureDependencyInConjure {
        statement: SourceLocation<'s>,
        signature: Signature<'s>,
    },
    RepeatedFunctionDependencyInConjure {
        statement: SourceLocation<'s>,
        function: Function<'s>,
    },
    CannotConjureTwoFunctionsWithIdenticalName {
        function: Function<'s>,
        statement: SourceLocation<'s>,
    },
}

impl<'s> CheckError<'s> {
    pub(super) fn print(self, resolve: &Resolver<'s>, mut out: &mut dyn Write) -> io::Result<()> {
        match self {
            CheckError::CannotResolveSignature { signature } => {
                Report::build(ReportKind::Error, signature.with_parens)
                    .with_label(
                        Label::new(signature.with_parens)
                            .with_color(Color::Cyan)
                            .with_message("here"),
                    )
                    .with_message(format!("cannot resolve signature '{signature}'"))
                    .finish()
                    .write(
                        (
                            signature.with_parens.file.file_name.clone(),
                            &signature.with_parens.file.inner,
                        ),
                        out,
                    )
            }
            CheckError::CannotResolveFunction { function } => {
                Report::build(ReportKind::Error, function.symbol)
                    .with_label(
                        Label::new(function.symbol)
                            .with_color(Color::Cyan)
                            .with_message("here"),
                    )
                    .with_message(format!("cannot resolve function '{function}'"))
                    .finish()
                    .write(
                        (
                            function.symbol.file.file_name.clone(),
                            &function.symbol.file.inner,
                        ),
                        out,
                    )
            }
            CheckError::CannotGiveSignatureToUnknownSignature { .. } => {
                writeln!(out, "CheckError::CannotGiveSignatureToUnknownSignature")
            }
            CheckError::CannotGiveFunctionToUnknownSignature { .. } => {
                writeln!(out, "CheckError::CannotGiveFunctionToUnknownSignature")
            }
            CheckError::CannotGiveSignatureToUnknownFunction { .. } => {
                writeln!(out, "CheckError::CannotGiveSignatureToUnknownFunction")
            }
            CheckError::CannotGiveFunctionToUnknownFunction { .. } => {
                writeln!(out, "CheckError::CannotGiveFunctionToUnknownFunction")
            }
            CheckError::CannotTakeSignatureFromUnknownFunction { .. } => {
                writeln!(out, "CheckError::CannotTakeSignatureFromUnknownFunction")
            }
            CheckError::CannotTakeFunctionFromUnknownFunction { .. } => {
                writeln!(out, "CheckError::CannotTakeFunctionFromUnknownFunction")
            }
            CheckError::CannotResolveGivenSignature { .. } => {
                writeln!(out, "CheckError::CannotResolveGivenSignature")
            }
            CheckError::CannotResolveGivenFunction { .. } => {
                writeln!(out, "CheckError::CannotResolveGivenFunction")
            }
            CheckError::CannotResolveTakenSignatureOfSignature { .. } => {
                writeln!(out, "CheckError::CannotResolveTakenSignatureOfSignature")
            }
            CheckError::CannotResolveTakenFunctionOfSignature { .. } => {
                writeln!(out, "CheckError::CannotResolveTakenFunctionOfSignature")
            }
            CheckError::CannotResolveTakenSignatureOfFunction { .. } => {
                writeln!(out, "CheckError::CannotResolveTakenSignatureOfFunction")
            }
            CheckError::CannotResolveTakenFunctionOfFunction { .. } => {
                writeln!(out, "CheckError::CannotResolveTakenFunctionOfFunction")
            }
            CheckError::TakenSignatureDependenciesNotProvided { .. } => {
                writeln!(out, "CheckError::TakenSignatureDependenciesNotProvided")
            }
            CheckError::TakenFunctionDependenciesNotProvided { .. } => {
                writeln!(out, "CheckError::TakenFunctionDependenciesNotProvided")
            }
            CheckError::FunctionGivenToSignatureDoesNotHaveExpectedSignature { .. } => writeln!(
                out,
                "CheckError::FunctionGivenToSignatureDoesNotHaveExpectedSignature"
            ),
            CheckError::FunctionGivenToFunctionDoesNotHaveExpectedSignature {
                statement,
                function,
                function_value,
                foreign,
                source,
                source_value,
                expected_signature_value,
            } => {
                // TODO
                _ = foreign;
                Report::build(ReportKind::Error, statement)
                    .with_label(
                        Label::new(function.symbol)
                            .with_color(Color::Cyan)
                            .with_message("this function..."),
                    )
                    .with_label(
                        Label::new(source.symbol)
                            .with_color(Color::Cyan)
                            .with_message("... is given to this function, but it does not have the expected signature"),
                    )
                    .with_message("function given to function does not have expected signature")
                    .finish()
                    .write(
                        (
                            function.symbol.file.file_name.clone(),
                            &function.symbol.file.inner,
                        ),
                        &mut out,
                    )?;
                let mut formatter = IndentingFormatter::new(&mut out);
                // write!(formatter, "value of source function: ")?;
                // source_value.format(resolve, &mut formatter)?;
                // formatter.new_line()?;
                // write!(formatter, "value of given function: ")?;
                // function_value.format(resolve, &mut formatter)?;
                // formatter.new_line()?;
                write!(formatter, "expected signature: ")?;
                expected_signature_value.format(resolve, &mut formatter)?;
                formatter.new_line()?;
                write!(formatter, "actual signature: ")?;
                function_value.signature().format(resolve, &mut formatter)?;
                formatter.new_line()?;
                Ok(())
            }
            CheckError::CannotGiveSignatureTwice { .. } => {
                writeln!(out, "CheckError::CannotGiveSignatureTwice")
            }
            CheckError::CannotGiveTwoSignaturesWithIdenticalName { .. } => {
                writeln!(out, "CheckError::CannotGiveTwoSignaturesWithIdenticalName")
            }
            CheckError::CannotGiveFunctionTwice { .. } => {
                writeln!(out, "CheckError::CannotGiveFunctionTwice")
            }
            CheckError::CannotGiveTwoFunctionsWithIdenticalName { .. } => {
                writeln!(out, "CheckError::CannotGiveTwoFunctionsWithIdenticalName")
            }
            CheckError::CannotConjureTwoSignaturesWithIdenticalName { .. } => writeln!(
                out,
                "CheckError::CannotConjureTwoSignaturesWithIdenticalName"
            ),
            CheckError::RepeatedSignatureDependencyInConjure { .. } => {
                writeln!(out, "CheckError::RepeatedSignatureDependencyInConjure")
            }
            CheckError::RepeatedFunctionDependencyInConjure { .. } => {
                writeln!(out, "CheckError::RepeatedFunctionDependencyInConjure")
            }
            CheckError::CannotConjureTwoFunctionsWithIdenticalName { .. } => writeln!(
                out,
                "CheckError::CannotConjureTwoFunctionsWithIdenticalName"
            ),
        }
    }
}
