use crate::parse::SourceLocation;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::marker::PhantomData;

#[derive(Debug, Copy, Clone)]
pub(crate) struct Signature<'s> {
    pub(crate) with_parens: SourceLocation<'s>,
    pub(crate) symbol: SourceLocation<'s>,
}

#[derive(Debug, Copy, Clone)]
pub(crate) struct Function<'s> {
    pub(crate) symbol: SourceLocation<'s>,
}

#[derive(Debug, Copy, Clone)]
pub(crate) enum ForeignSignature<'s> {
    Explicit(Signature<'s>),
    Implicit(Signature<'s>),
}

#[derive(Debug, Copy, Clone)]
pub(crate) enum ForeignFunction<'s> {
    Explicit(Function<'s>),
    Implicit(Function<'s>),
}

#[derive(Debug, Copy, Clone)]
pub(crate) enum ConjureDependency<'s> {
    Signature(Signature<'s>),
    Function(Function<'s>),
}

#[derive(Debug, Clone)]
pub(crate) struct ConjureDependencies<'s>(pub(crate) Vec<ConjureDependency<'s>>);

#[derive(Debug, Clone)]
pub(crate) struct TakeSignature<'s> {
    pub(crate) phantom: PhantomData<SourceLocation<'s>>,
}

#[derive(Debug, Clone)]
pub(crate) struct DefineSignature<'s> {
    pub(crate) context: SignatureContext<'s>,
}

#[derive(Debug, Clone)]
pub(crate) struct TakeSignatureFrom<'s> {
    pub(crate) foreign: ForeignSignature<'s>,
    pub(crate) source: Function<'s>,
}

#[derive(Debug, Clone)]
pub(crate) struct GiveSignatureToSignature<'s> {
    pub(crate) signature: Signature<'s>,
    pub(crate) foreign: ForeignSignature<'s>,
    pub(crate) source: Signature<'s>,
}

#[derive(Debug, Clone)]
pub(crate) struct GiveFunctionToSignature<'s> {
    pub(crate) function: Function<'s>,
    pub(crate) foreign: ForeignFunction<'s>,
    pub(crate) source: Signature<'s>,
}

#[derive(Debug, Clone)]
pub(crate) enum SignatureAssignmentRhs<'s> {
    Take(TakeSignature<'s>),
    Define(DefineSignature<'s>),
    TakeFrom(TakeSignatureFrom<'s>),
    GiveSignatureToSignature(GiveSignatureToSignature<'s>),
    GiveFunctionToSignature(GiveFunctionToSignature<'s>),
}

#[derive(Debug, Clone)]
pub(crate) struct SignatureAssignment<'s> {
    pub(crate) lhs: Signature<'s>,
    pub(crate) rhs: SignatureAssignmentRhs<'s>,
    pub(crate) location: SourceLocation<'s>,
}

#[derive(Debug, Clone)]
pub(crate) struct TakeFunction<'s> {
    pub(crate) signature: Signature<'s>,
}

#[derive(Debug, Clone)]
pub(crate) struct DefineFunction<'s> {
    pub(crate) context: FunctionContext<'s>,
}

#[derive(Debug, Clone)]
pub(crate) struct TakeFunctionFrom<'s> {
    pub(crate) foreign: ForeignFunction<'s>,
    pub(crate) source: Function<'s>,
}

#[derive(Debug, Clone)]
pub(crate) struct GiveSignatureToFunction<'s> {
    pub(crate) signature: Signature<'s>,
    pub(crate) foreign: ForeignSignature<'s>,
    pub(crate) source: Function<'s>,
}

#[derive(Debug, Clone)]
pub(crate) struct GiveFunctionToFunction<'s> {
    pub(crate) function: Function<'s>,
    pub(crate) foreign: ForeignFunction<'s>,
    pub(crate) source: Function<'s>,
}

#[derive(Debug, Clone)]
pub(crate) enum FunctionAssignmentRhs<'s> {
    Take(TakeFunction<'s>),
    Define(DefineFunction<'s>),
    TakeFrom(TakeFunctionFrom<'s>),
    GiveSignatureToFunction(GiveSignatureToFunction<'s>),
    GiveFunctionToFunction(GiveFunctionToFunction<'s>),
}

#[derive(Debug, Clone)]
pub(crate) struct FunctionAssignment<'s> {
    pub(crate) lhs: Function<'s>,
    pub(crate) rhs: FunctionAssignmentRhs<'s>,
    pub(crate) location: SourceLocation<'s>,
}

#[derive(Debug, Clone)]
pub(crate) struct GiveSignature<'s> {
    pub(crate) signature: Signature<'s>,
    pub(crate) location: SourceLocation<'s>,
}

#[derive(Debug, Clone)]
pub(crate) struct GiveFunction<'s> {
    pub(crate) function: Function<'s>,
    pub(crate) location: SourceLocation<'s>,
}

#[derive(Debug, Clone)]
pub(crate) struct ConjureSignature<'s> {
    pub(crate) signature: Signature<'s>,
    pub(crate) dependencies: ConjureDependencies<'s>,
    pub(crate) location: SourceLocation<'s>,
}

#[derive(Debug, Clone)]
pub(crate) struct ConjureFunction<'s> {
    pub(crate) function: Function<'s>,
    pub(crate) signature: Signature<'s>,
    pub(crate) dependencies: ConjureDependencies<'s>,
    pub(crate) location: SourceLocation<'s>,
}

#[derive(Debug, Clone)]
pub(crate) enum SignatureStatement<'s> {
    SignatureAssignment(SignatureAssignment<'s>),
    FunctionAssignment(FunctionAssignment<'s>),
    ConjureSignature(ConjureSignature<'s>),
    ConjureFunction(ConjureFunction<'s>),
}

#[derive(Debug, Clone)]
pub(crate) enum FunctionStatement<'s> {
    SignatureAssignment(SignatureAssignment<'s>),
    FunctionAssignment(FunctionAssignment<'s>),
    GiveSignature(GiveSignature<'s>),
    GiveFunction(GiveFunction<'s>),
}

#[derive(Debug, Clone)]
pub(crate) struct SignatureContext<'s> {
    pub(crate) trace: bool,
    pub(crate) statements: Vec<SignatureStatement<'s>>,
}

#[derive(Debug, Clone)]
pub(crate) struct FunctionContext<'s> {
    pub(crate) trace: bool,
    pub(crate) statements: Vec<FunctionStatement<'s>>,
}

impl<'s> Display for Signature<'s> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        // f.write_str(self.with_parens.as_str())?;
        write!(f, " {:?}", self.with_parens)
    }
}

impl<'s> Display for Function<'s> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        // f.write_str(self.symbol.as_str())?;
        write!(f, "{:?}", self.symbol)
    }
}

impl<'s> Display for ForeignSignature<'s> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ForeignSignature::Explicit(signature) => signature.fmt(f),
            ForeignSignature::Implicit(signature) => signature.fmt(f),
        }
    }
}

impl<'s> Display for ForeignFunction<'s> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ForeignFunction::Explicit(function) => function.fmt(f),
            ForeignFunction::Implicit(function) => function.fmt(f),
        }
    }
}

impl<'s> Signature<'s> {
    pub(crate) fn as_str(self) -> &'s str {
        self.symbol.as_str()
    }
}

impl<'s> Function<'s> {
    pub(crate) fn as_str(self) -> &'s str {
        self.symbol.as_str()
    }
}

impl<'s> ForeignSignature<'s> {
    pub(crate) fn as_str(self) -> &'s str {
        match self {
            ForeignSignature::Explicit(signature) => signature.as_str(),
            ForeignSignature::Implicit(signature) => signature.as_str(),
        }
    }
}

impl<'s> ForeignFunction<'s> {
    pub(crate) fn as_str(self) -> &'s str {
        match self {
            ForeignFunction::Explicit(function) => function.as_str(),
            ForeignFunction::Implicit(function) => function.as_str(),
        }
    }
}
