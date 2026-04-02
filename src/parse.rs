use itertools::Itertools;
use pest::iterators::Pair;
use pest::Parser;
use pest_derive::Parser;
use std::marker::PhantomData;

#[derive(Parser)]
#[grammar = "syntax.pest"]
struct MyParser;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub(crate) struct Signature<'s>(pub(crate) &'s str);

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub(crate) struct Function<'s>(pub(crate) &'s str);

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub(crate) struct SignatureLiteral<'s>(pub(crate) &'s str);

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub(crate) struct FunctionLiteral<'s>(pub(crate) &'s str);

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct TakeSignature<'s> {
    pub(crate) literal: SignatureLiteral<'s>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct ConjureSignature<'s> {
    pub(crate) phantom: PhantomData<&'s str>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct DefineSignature<'s> {
    pub(crate) context: Context<'s>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct TakeSignatureFrom<'s> {
    pub(crate) literal: SignatureLiteral<'s>,
    pub(crate) source: Function<'s>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct GiveSignatureToSignature<'s> {
    pub(crate) signature: Signature<'s>,
    pub(crate) literal: SignatureLiteral<'s>,
    pub(crate) source: Signature<'s>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct GiveFunctionToSignature<'s> {
    pub(crate) function: Function<'s>,
    pub(crate) literal: FunctionLiteral<'s>,
    pub(crate) source: Signature<'s>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) enum SignatureAssignmentRhs<'s> {
    Take(TakeSignature<'s>),
    Conjure(ConjureSignature<'s>),
    Define(DefineSignature<'s>),
    TakeFrom(TakeSignatureFrom<'s>),
    GiveSignatureToSignature(GiveSignatureToSignature<'s>),
    GiveFunctionToSignature(GiveFunctionToSignature<'s>),
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct SignatureAssignment<'s> {
    pub(crate) lhs: Signature<'s>,
    pub(crate) rhs: SignatureAssignmentRhs<'s>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct TakeFunction<'s> {
    pub(crate) signature: Signature<'s>,
    pub(crate) literal: FunctionLiteral<'s>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct ConjureFunction<'s> {
    pub(crate) signature: Signature<'s>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct DefineFunction<'s> {
    pub(crate) context: Context<'s>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct TakeFunctionFrom<'s> {
    pub(crate) literal: FunctionLiteral<'s>,
    pub(crate) source: Function<'s>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct GiveSignatureToFunction<'s> {
    pub(crate) signature: Signature<'s>,
    pub(crate) literal: SignatureLiteral<'s>,
    pub(crate) source: Function<'s>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct GiveFunctionToFunction<'s> {
    pub(crate) function: Function<'s>,
    pub(crate) literal: FunctionLiteral<'s>,
    pub(crate) source: Function<'s>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) enum FunctionAssignmentRhs<'s> {
    Take(TakeFunction<'s>),
    Conjure(ConjureFunction<'s>),
    Define(DefineFunction<'s>),
    TakeFrom(TakeFunctionFrom<'s>),
    GiveSignatureToFunction(GiveSignatureToFunction<'s>),
    GiveFunctionToFunction(GiveFunctionToFunction<'s>),
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct FunctionAssignment<'s> {
    pub(crate) lhs: Function<'s>,
    pub(crate) rhs: FunctionAssignmentRhs<'s>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct GiveSignature<'s> {
    pub(crate) signature: Signature<'s>,
    pub(crate) literal: SignatureLiteral<'s>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct GiveFunction<'s> {
    pub(crate) function: Function<'s>,
    pub(crate) literal: FunctionLiteral<'s>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) enum Statement<'s> {
    SignatureAssignment(SignatureAssignment<'s>),
    FunctionAssignment(FunctionAssignment<'s>),
    GiveSignature(GiveSignature<'s>),
    GiveFunction(GiveFunction<'s>),
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct Context<'s>(pub(crate) Vec<Statement<'s>>);

fn parse_signature(pair: Pair<Rule>) -> Signature {
    let (symbol,) = pair.into_inner().collect_tuple().unwrap();
    Signature(symbol.as_str())
}

fn parse_function(pair: Pair<Rule>) -> Function {
    let (symbol,) = pair.into_inner().collect_tuple().unwrap();
    Function(symbol.as_str())
}

fn parse_signature_literal(pair: Pair<Rule>) -> SignatureLiteral {
    let (symbol,) = pair.into_inner().collect_tuple().unwrap();
    SignatureLiteral(symbol.as_str())
}

fn parse_function_literal(pair: Pair<Rule>) -> FunctionLiteral {
    let (symbol,) = pair.into_inner().collect_tuple().unwrap();
    FunctionLiteral(symbol.as_str())
}

fn parse_maybe_signature_literal<'s>(pair: Pair<'s, Rule>, default: &'s str) -> SignatureLiteral<'s> {
    if let Some(literal) = pair.into_inner().next() {
        parse_signature_literal(literal)
    } else {
        SignatureLiteral(default)
    }
}

fn parse_maybe_function_literal<'s>(pair: Pair<'s, Rule>, default: &'s str) -> FunctionLiteral<'s> {
    if let Some(literal) = pair.into_inner().next() {
        parse_function_literal(literal)
    } else {
        FunctionLiteral(default)
    }
}

fn parse_take_signature<'s>(pair: Pair<'s, Rule>, default_literal: &'s str) -> TakeSignature<'s> {
    let (literal,) = pair.into_inner().collect_tuple().unwrap();
    TakeSignature {
        literal: parse_maybe_signature_literal(literal, default_literal),
    }
}

fn parse_conjure_signature(pair: Pair<Rule>) -> ConjureSignature {
    _ = pair;
    ConjureSignature {
        phantom: PhantomData
    }
}

fn parse_define_signature(pair: Pair<Rule>) -> DefineSignature {
    let (context,) = pair.into_inner().collect_tuple().unwrap();
    DefineSignature {
        context: parse_context(context),
    }
}

fn parse_take_signature_from<'s>(pair: Pair<'s, Rule>, default_literal: &'s str) -> TakeSignatureFrom<'s> {
    let (literal, source) = pair.into_inner().collect_tuple().unwrap();
    TakeSignatureFrom {
        literal: parse_maybe_signature_literal(literal, default_literal),
        source: parse_function(source),
    }
}

fn parse_give_signature_to_signature(pair: Pair<Rule>) -> GiveSignatureToSignature {
    let (local, literal, source) = pair.into_inner().collect_tuple().unwrap();
    let signature = parse_signature(local);
    GiveSignatureToSignature {
        signature,
        literal: parse_maybe_signature_literal(literal, signature.0),
        source: parse_signature(source),
    }
}

fn parse_give_function_to_signature(pair: Pair<Rule>) -> GiveFunctionToSignature {
    let (local, literal, source) = pair.into_inner().collect_tuple().unwrap();
    let function = parse_function(local);
    GiveFunctionToSignature {
        function,
        literal: parse_maybe_function_literal(literal, function.0),
        source: parse_signature(source),
    }
}

fn parse_take_function<'s>(pair: Pair<'s, Rule>, default_literal: &'s str) -> TakeFunction<'s> {
    let (signature, literal) = pair.into_inner().collect_tuple().unwrap();
    TakeFunction {
        signature: parse_signature(signature),
        literal: parse_maybe_function_literal(literal, default_literal),
    }
}

fn parse_conjure_function(pair: Pair<Rule>) -> ConjureFunction {
    let (signature, ) = pair.into_inner().collect_tuple().unwrap();
    ConjureFunction {
        signature: parse_signature(signature),
    }
}

fn parse_define_function(pair: Pair<Rule>) -> DefineFunction {
    let (context, ) = pair.into_inner().collect_tuple().unwrap();
    DefineFunction {
        context: parse_context(context),
    }
}

fn parse_take_function_from<'s>(pair: Pair<'s, Rule>, default_literal: &'s str) -> TakeFunctionFrom<'s> {
    let (literal, source) = pair.into_inner().collect_tuple().unwrap();
    TakeFunctionFrom {
        literal: parse_maybe_function_literal(literal, default_literal),
        source: parse_function(source),
    }
}

fn parse_give_signature_to_function(pair: Pair<Rule>) -> GiveSignatureToFunction {
    let (local, literal, source) = pair.into_inner().collect_tuple().unwrap();
    let signature = parse_signature(local);
    GiveSignatureToFunction {
        signature,
        literal: parse_maybe_signature_literal(literal, signature.0),
        source: parse_function(source),
    }
}

fn parse_give_function_to_function(pair: Pair<Rule>) -> GiveFunctionToFunction {
    let (local, literal, source) = pair.into_inner().collect_tuple().unwrap();
    let function = parse_function(local);
    GiveFunctionToFunction {
        function,
        literal: parse_maybe_function_literal(literal, function.0),
        source: parse_function(source),
    }
}

fn parse_signature_assignment_rhs<'s>(pair: Pair<'s, Rule>, lhs: &'s str) -> SignatureAssignmentRhs<'s> {
    match pair.as_rule() {
        Rule::take_signature => SignatureAssignmentRhs::Take(parse_take_signature(pair, lhs)),
        Rule::conjure_signature => SignatureAssignmentRhs::Conjure(parse_conjure_signature(pair)),
        Rule::define_signature => SignatureAssignmentRhs::Define(parse_define_signature(pair)),
        Rule::take_signature_from => {
            SignatureAssignmentRhs::TakeFrom(parse_take_signature_from(pair, lhs))
        }
        Rule::give_signature_to_signature => {
            SignatureAssignmentRhs::GiveSignatureToSignature(parse_give_signature_to_signature(pair))
        }
        Rule::give_function_to_signature => {
            SignatureAssignmentRhs::GiveFunctionToSignature(parse_give_function_to_signature(pair))
        }
        _ => unreachable!(),
    }
}

fn parse_function_assignment_rhs<'s>(pair: Pair<'s, Rule>, lhs: &'s str) -> FunctionAssignmentRhs<'s> {
    match pair.as_rule() {
        Rule::take_function => FunctionAssignmentRhs::Take(parse_take_function(pair, lhs)),
        Rule::conjure_function => FunctionAssignmentRhs::Conjure(parse_conjure_function(pair)),
        Rule::define_function => FunctionAssignmentRhs::Define(parse_define_function(pair)),
        Rule::take_function_from => FunctionAssignmentRhs::TakeFrom(parse_take_function_from(pair, lhs)),
        Rule::give_signature_to_function => {
            FunctionAssignmentRhs::GiveSignatureToFunction(parse_give_signature_to_function(pair))
        }
        Rule::give_function_to_function => {
            FunctionAssignmentRhs::GiveFunctionToFunction(parse_give_function_to_function(pair))
        }
        _ => unreachable!(),
    }
}

fn parse_signature_assignment(pair: Pair<Rule>) -> SignatureAssignment {
    let (lhs_pair, rhs_pair) = pair.into_inner().collect_tuple().unwrap();
    let lhs = parse_signature(lhs_pair);
    SignatureAssignment {
        lhs,
        rhs: parse_signature_assignment_rhs(rhs_pair, lhs.0),
    }
}

fn parse_function_assignment(pair: Pair<Rule>) -> FunctionAssignment {
    let (lhs_pair, rhs_pair) = pair.into_inner().collect_tuple().unwrap();
    let lhs = parse_function(lhs_pair);
    FunctionAssignment {
        lhs,
        rhs: parse_function_assignment_rhs(rhs_pair, lhs.0),
    }
}

fn parse_give_signature(pair: Pair<Rule>) -> GiveSignature {
    let (signature, literal) = pair.into_inner().collect_tuple().unwrap();
    let signature = parse_signature(signature);
    GiveSignature {
        signature,
        literal: parse_maybe_signature_literal(literal, signature.0),
    }
}

fn parse_give_function(pair: Pair<Rule>) -> GiveFunction {
    let (function, literal) = pair.into_inner().collect_tuple().unwrap();
    let function = parse_function(function);
    GiveFunction {
        function,
        literal: parse_maybe_function_literal(literal, function.0),
    }
}

fn parse_statement(pair: Pair<Rule>) -> Statement {
    match pair.as_rule() {
        Rule::signature_assignment => {
            Statement::SignatureAssignment(parse_signature_assignment(pair))
        }
        Rule::function_assignment => Statement::FunctionAssignment(parse_function_assignment(pair)),
        Rule::give_signature => Statement::GiveSignature(parse_give_signature(pair)),
        Rule::give_function => Statement::GiveFunction(parse_give_function(pair)),
        _ => unreachable!(),
    }
}

fn parse_context(pair: Pair<Rule>) -> Context {
    let statements = pair
        .into_inner()
        .map(|statement| parse_statement(statement))
        .collect();
    Context(statements)
}

pub(crate) fn parse(text: &'_ str) -> Context<'_> {
    let pairs = MyParser::parse(Rule::file, text).unwrap();
    let statements = pairs
        .filter_map(|statement| {
            if let Rule::EOI = statement.as_rule() {
                None
            } else {
                Some(parse_statement(statement))
            }
        })
        .collect();
    Context(statements)
}
