use itertools::Itertools;
use pest::iterators::Pair;
use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "syntax.pest"]
struct MyParser;

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct Signature<'s>(pub(crate) &'s str);

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct Function<'s>(pub(crate) &'s str);

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct SignatureLiteral<'s>(pub(crate) &'s str);

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct FunctionLiteral<'s>(pub(crate) &'s str);

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) enum ConjureDependency<'s> {
    Signature(Signature<'s>),
    Function(Function<'s>),
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct ConjureDependencies<'s>(pub(crate) Vec<ConjureDependency<'s>>);

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct TakeSignature<'s> {
    pub(crate) literal: SignatureLiteral<'s>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct ConjureSignature<'s> {
    pub(crate) dependencies: ConjureDependencies<'s>,
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
pub(crate) enum SignatureAssignmentRhs<'s> {
    Take(TakeSignature<'s>),
    Conjure(ConjureSignature<'s>),
    Define(DefineSignature<'s>),
    TakeFrom(TakeSignatureFrom<'s>),
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
    pub(crate) dependencies: ConjureDependencies<'s>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct DefineFunction<'s> {
    pub(crate) signature: Signature<'s>,
    pub(crate) context: Context<'s>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct TakeFunctionFrom<'s> {
    pub(crate) literal: FunctionLiteral<'s>,
    pub(crate) source: Function<'s>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct GiveSignatureTo<'s> {
    pub(crate) signature: Signature<'s>,
    pub(crate) literal: SignatureLiteral<'s>,
    pub(crate) source: Function<'s>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct GiveFunctionTo<'s> {
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
    GiveSignatureTo(GiveSignatureTo<'s>),
    GiveFunctionTo(GiveFunctionTo<'s>),
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

fn parse_conjure_dependency(pair: Pair<Rule>) -> ConjureDependency {
    let rule = pair.as_rule();
    let (symbol,) = pair.into_inner().collect_tuple().unwrap();
    match rule {
        Rule::signature => ConjureDependency::Signature(Signature(symbol.as_str())),
        Rule::function => ConjureDependency::Function(Function(symbol.as_str())),
        _ => unreachable!(),
    }
}

fn parse_conjure_dependencies(pair: Pair<Rule>) -> ConjureDependencies {
    let dependencies = pair
        .into_inner()
        .map(|dependency| parse_conjure_dependency(dependency))
        .collect();
    ConjureDependencies(dependencies)
}

fn parse_take_signature(pair: Pair<Rule>) -> TakeSignature {
    let (literal,) = pair.into_inner().collect_tuple().unwrap();
    TakeSignature {
        literal: parse_signature_literal(literal),
    }
}

fn parse_conjure_signature(pair: Pair<Rule>) -> ConjureSignature {
    let (dependencies,) = pair.into_inner().collect_tuple().unwrap();
    ConjureSignature {
        dependencies: parse_conjure_dependencies(dependencies),
    }
}

fn parse_define_signature(pair: Pair<Rule>) -> DefineSignature {
    let (context,) = pair.into_inner().collect_tuple().unwrap();
    DefineSignature {
        context: parse_context(context),
    }
}

fn parse_take_signature_from(pair: Pair<Rule>) -> TakeSignatureFrom {
    let (literal, source) = pair.into_inner().collect_tuple().unwrap();
    TakeSignatureFrom {
        literal: parse_signature_literal(literal),
        source: parse_function(source),
    }
}

fn parse_take_function(pair: Pair<Rule>) -> TakeFunction {
    let (signature, literal) = pair.into_inner().collect_tuple().unwrap();
    TakeFunction {
        signature: parse_signature(signature),
        literal: parse_function_literal(literal),
    }
}

fn parse_conjure_function(pair: Pair<Rule>) -> ConjureFunction {
    let (signature, dependencies) = pair.into_inner().collect_tuple().unwrap();
    ConjureFunction {
        signature: parse_signature(signature),
        dependencies: parse_conjure_dependencies(dependencies),
    }
}

fn parse_define_function(pair: Pair<Rule>) -> DefineFunction {
    let (signature, context) = pair.into_inner().collect_tuple().unwrap();
    DefineFunction {
        signature: parse_signature(signature),
        context: parse_context(context),
    }
}

fn parse_take_function_from(pair: Pair<Rule>) -> TakeFunctionFrom {
    let (literal, source) = pair.into_inner().collect_tuple().unwrap();
    TakeFunctionFrom {
        literal: parse_function_literal(literal),
        source: parse_function(source),
    }
}

fn parse_give_signature_to(pair: Pair<Rule>) -> GiveSignatureTo {
    let (local, literal, source) = pair.into_inner().collect_tuple().unwrap();
    GiveSignatureTo {
        signature: parse_signature(local),
        literal: parse_signature_literal(literal),
        source: parse_function(source),
    }
}

fn parse_give_function_to(pair: Pair<Rule>) -> GiveFunctionTo {
    let (local, literal, source) = pair.into_inner().collect_tuple().unwrap();
    GiveFunctionTo {
        function: parse_function(local),
        literal: parse_function_literal(literal),
        source: parse_function(source),
    }
}

fn parse_signature_assignment_rhs(pair: Pair<Rule>) -> SignatureAssignmentRhs {
    match pair.as_rule() {
        Rule::take_signature => SignatureAssignmentRhs::Take(parse_take_signature(pair)),
        Rule::conjure_signature => SignatureAssignmentRhs::Conjure(parse_conjure_signature(pair)),
        Rule::define_signature => SignatureAssignmentRhs::Define(parse_define_signature(pair)),
        Rule::take_signature_from => {
            SignatureAssignmentRhs::TakeFrom(parse_take_signature_from(pair))
        }
        _ => unreachable!(),
    }
}

fn parse_function_assignment_rhs(pair: Pair<Rule>) -> FunctionAssignmentRhs {
    match pair.as_rule() {
        Rule::take_function => FunctionAssignmentRhs::Take(parse_take_function(pair)),
        Rule::conjure_function => FunctionAssignmentRhs::Conjure(parse_conjure_function(pair)),
        Rule::define_function => FunctionAssignmentRhs::Define(parse_define_function(pair)),
        Rule::take_function_from => FunctionAssignmentRhs::TakeFrom(parse_take_function_from(pair)),
        Rule::give_signature_to => {
            FunctionAssignmentRhs::GiveSignatureTo(parse_give_signature_to(pair))
        }
        Rule::give_function_to => {
            FunctionAssignmentRhs::GiveFunctionTo(parse_give_function_to(pair))
        }
        _ => unreachable!(),
    }
}

fn parse_signature_assignment(pair: Pair<Rule>) -> SignatureAssignment {
    let (lhs_pair, rhs_pair) = pair.into_inner().collect_tuple().unwrap();
    SignatureAssignment {
        lhs: parse_signature(lhs_pair),
        rhs: parse_signature_assignment_rhs(rhs_pair),
    }
}

fn parse_function_assignment(pair: Pair<Rule>) -> FunctionAssignment {
    let (lhs_pair, rhs_pair) = pair.into_inner().collect_tuple().unwrap();
    FunctionAssignment {
        lhs: parse_function(lhs_pair),
        rhs: parse_function_assignment_rhs(rhs_pair),
    }
}

fn parse_give_signature(pair: Pair<Rule>) -> GiveSignature {
    let (signature, literal) = pair.into_inner().collect_tuple().unwrap();
    GiveSignature {
        signature: parse_signature(signature),
        literal: parse_signature_literal(literal),
    }
}

fn parse_give_function(pair: Pair<Rule>) -> GiveFunction {
    let (function, literal) = pair.into_inner().collect_tuple().unwrap();
    GiveFunction {
        function: parse_function(function),
        literal: parse_function_literal(literal),
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
