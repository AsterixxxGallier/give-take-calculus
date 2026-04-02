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
pub(crate) enum SignatureLiteral<'s> {
    Explicit {
        #[expect(unused)]
        with_ticks: SourceLocation<'s>,
        #[expect(unused)]
        with_parens: SourceLocation<'s>,
        symbol: SourceLocation<'s>,
    },
    Implicit(Signature<'s>),
}

#[derive(Debug, Copy, Clone)]
pub(crate) enum FunctionLiteral<'s> {
    Explicit {
        #[expect(unused)]
        with_ticks: SourceLocation<'s>,
        symbol: SourceLocation<'s>,
    },
    Implicit(Function<'s>),
}

#[derive(Debug, Clone)]
pub(crate) struct TakeSignature<'s> {
    pub(crate) literal: SignatureLiteral<'s>,
}

#[derive(Debug, Clone)]
pub(crate) struct ConjureSignature<'s> {
    pub(crate) phantom: PhantomData<SourceLocation<'s>>,
}

#[derive(Debug, Clone)]
pub(crate) struct DefineSignature<'s> {
    pub(crate) context: Context<'s>,
}

#[derive(Debug, Clone)]
pub(crate) struct TakeSignatureFrom<'s> {
    pub(crate) literal: SignatureLiteral<'s>,
    pub(crate) source: Function<'s>,
}

#[derive(Debug, Clone)]
pub(crate) struct GiveSignatureToSignature<'s> {
    pub(crate) signature: Signature<'s>,
    pub(crate) literal: SignatureLiteral<'s>,
    pub(crate) source: Signature<'s>,
}

#[derive(Debug, Clone)]
pub(crate) struct GiveFunctionToSignature<'s> {
    pub(crate) function: Function<'s>,
    pub(crate) literal: FunctionLiteral<'s>,
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
    pub(crate) literal: FunctionLiteral<'s>,
}

#[derive(Debug, Clone)]
pub(crate) struct ConjureFunction<'s> {
    pub(crate) signature: Signature<'s>,
}

#[derive(Debug, Clone)]
pub(crate) struct DefineFunction<'s> {
    pub(crate) context: Context<'s>,
}

#[derive(Debug, Clone)]
pub(crate) struct TakeFunctionFrom<'s> {
    pub(crate) literal: FunctionLiteral<'s>,
    pub(crate) source: Function<'s>,
}

#[derive(Debug, Clone)]
pub(crate) struct GiveSignatureToFunction<'s> {
    pub(crate) signature: Signature<'s>,
    pub(crate) literal: SignatureLiteral<'s>,
    pub(crate) source: Function<'s>,
}

#[derive(Debug, Clone)]
pub(crate) struct GiveFunctionToFunction<'s> {
    pub(crate) function: Function<'s>,
    pub(crate) literal: FunctionLiteral<'s>,
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
    pub(crate) literal: SignatureLiteral<'s>,
}

#[derive(Debug, Clone)]
pub(crate) struct GiveFunction<'s> {
    pub(crate) function: Function<'s>,
    pub(crate) literal: FunctionLiteral<'s>,
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

impl<'s> SignatureLiteral<'s> {
    pub(crate) fn as_str(self) -> &'s str {
        match self {
            SignatureLiteral::Explicit { symbol, .. } => symbol.as_str(),
            SignatureLiteral::Implicit(signature) => signature.as_str(),
        }
    }
}

impl<'s> FunctionLiteral<'s> {
    pub(crate) fn as_str(self) -> &'s str {
        match self {
            FunctionLiteral::Explicit { symbol, .. } => symbol.as_str(),
            FunctionLiteral::Implicit(function) => function.as_str(),
        }
    }
}
