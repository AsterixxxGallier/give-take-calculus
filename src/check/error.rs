use crate::parse::{ForeignFunction, ForeignSignature, Function, Signature, SourceLocation};

#[derive(Debug)]
#[allow(unused)]
pub(crate) enum CheckError<'s> {
    CannotResolveSignature {
        signature: Signature<'s>,
    },
    CannotResolveFunction {
        function: Function<'s>,
    },
    CannotTakeSignatureFromUnknownFunction {
        statement: SourceLocation<'s>,
        source: Function<'s>,
    },
    CannotResolveGivenSignature {
        statement: SourceLocation<'s>,
        source: Function<'s>,
        foreign: ForeignSignature<'s>,
    },
    GiveSignatureDependenciesNotProvided {
        statement: SourceLocation<'s>,
        source: Function<'s>,
        foreign: ForeignSignature<'s>,
    },
    CannotResolveTakenSignatureOfSignature {
        statement: SourceLocation<'s>,
        source: Signature<'s>,
        foreign: ForeignSignature<'s>,
    },
    CannotGiveSignatureToUnknownSignature {
        statement: SourceLocation<'s>,
        source: Signature<'s>,
    },
    CannotResolveTakenFunctionOfSignature {
        statement: SourceLocation<'s>,
        source: Signature<'s>,
        foreign: ForeignFunction<'s>,
    },
    FunctionGivenToSignatureDoesNotHaveExpectedSignature {
        statement: SourceLocation<'s>,
        function: Function<'s>,
        foreign: ForeignFunction<'s>,
        source: Signature<'s>,
    },
    CannotResolveGivenFunction {
        statement: SourceLocation<'s>,
        source: Function<'s>,
        foreign: ForeignFunction<'s>,
    },
    GiveFunctionDependenciesNotProvided {
        statement: SourceLocation<'s>,
        source: Function<'s>,
        foreign: ForeignFunction<'s>,
    },
    CannotResolveTakenSignatureOfFunction {
        statement: SourceLocation<'s>,
        source: Function<'s>,
        foreign: ForeignSignature<'s>,
    },
    CannotGiveSignatureToUnknownFunction {
        statement: SourceLocation<'s>,
        source: Function<'s>,
    },
    CannotResolveTakenFunctionOfFunction {
        statement: SourceLocation<'s>,
        source: Function<'s>,
        foreign: ForeignFunction<'s>,
    },
    CannotGiveFunctionToUnknownFunction {
        statement: SourceLocation<'s>,
        source: Function<'s>,
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