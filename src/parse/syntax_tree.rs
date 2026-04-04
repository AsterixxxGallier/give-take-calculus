use crate::parse::SourceLocation;
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
pub(crate) struct ConjureSignature<'s> {
    pub(crate) dependencies: ConjureDependencies<'s>,
}

#[derive(Debug, Clone)]
pub(crate) struct DefineSignature<'s> {
    pub(crate) context: Context<'s>,
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
    Conjure(ConjureSignature<'s>),
    Define(DefineSignature<'s>),
    TakeFrom(TakeSignatureFrom<'s>),
    GiveSignatureToSignature(GiveSignatureToSignature<'s>),
    GiveFunctionToSignature(GiveFunctionToSignature<'s>),
}

#[derive(Debug, Clone)]
pub(crate) struct SignatureAssignment<'s> {
    pub(crate) lhs: Signature<'s>,
    pub(crate) rhs: SignatureAssignmentRhs<'s>,
}

#[derive(Debug, Clone)]
pub(crate) struct TakeFunction<'s> {
    pub(crate) signature: Signature<'s>,
}

#[derive(Debug, Clone)]
pub(crate) struct ConjureFunction<'s> {
    pub(crate) signature: Signature<'s>,
    pub(crate) dependencies: ConjureDependencies<'s>,
}

#[derive(Debug, Clone)]
pub(crate) struct DefineFunction<'s> {
    pub(crate) context: Context<'s>,
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
    Conjure(ConjureFunction<'s>),
    Define(DefineFunction<'s>),
    TakeFrom(TakeFunctionFrom<'s>),
    GiveSignatureToFunction(GiveSignatureToFunction<'s>),
    GiveFunctionToFunction(GiveFunctionToFunction<'s>),
}

#[derive(Debug, Clone)]
pub(crate) struct FunctionAssignment<'s> {
    pub(crate) lhs: Function<'s>,
    pub(crate) rhs: FunctionAssignmentRhs<'s>,
}

#[derive(Debug, Clone)]
pub(crate) struct GiveSignature<'s> {
    pub(crate) signature: Signature<'s>,
}

#[derive(Debug, Clone)]
pub(crate) struct GiveFunction<'s> {
    pub(crate) function: Function<'s>,
}

#[derive(Debug, Clone)]
pub(crate) enum Statement<'s> {
    SignatureAssignment(SignatureAssignment<'s>),
    FunctionAssignment(FunctionAssignment<'s>),
    GiveSignature(GiveSignature<'s>),
    GiveFunction(GiveFunction<'s>),
}

#[derive(Debug, Clone)]
pub(crate) struct Context<'s>(pub(crate) Vec<Statement<'s>>);

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
