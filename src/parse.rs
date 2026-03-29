use itertools::Itertools;
use pest::iterators::Pair;
use pest::Parser;
use pest_derive::Parser;
use std::marker::PhantomData;

#[derive(Parser)]
#[grammar = "syntax.pest"]
struct MyParser;

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct Signature<'s>(&'s str);

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct Function<'s>(&'s str);

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) enum SignatureOrFunction<'s> {
    Signature(Signature<'s>),
    Function(Function<'s>),
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct ConjureDependencies<'s>(Vec<SignatureOrFunction<'s>>);

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct TakeSignature<'s>(PhantomData<&'s ()>);

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct ConjureSignature<'s> {
    dependencies: ConjureDependencies<'s>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct DefineSignature<'s> {
    context: Statements<'s>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct TakeSignatureFrom<'s> {
    // name of the taken signature in the source function context
    remote: Signature<'s>,
    source: Function<'s>,
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
    lhs: Signature<'s>,
    rhs: SignatureAssignmentRhs<'s>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct TakeFunction<'s> {
    signature: Signature<'s>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct ConjureFunction<'s> {
    signature: Signature<'s>,
    dependencies: ConjureDependencies<'s>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct DefineFunction<'s> {
    signature: Signature<'s>,
    context: Statements<'s>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct TakeFunctionFrom<'s> {
    // name of the taken function in the source function context
    remote: Function<'s>,
    source: Function<'s>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct GiveSignatureTo<'s> {
    // name of the given signature in this context
    local: Signature<'s>,
    // name of the given signature in source function context
    remote: Signature<'s>,
    source: Function<'s>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct GiveFunctionTo<'s> {
    // name of the given function in this context
    local: Function<'s>,
    // name of the given function in source function context
    remote: Function<'s>,
    source: Function<'s>,
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
    lhs: Function<'s>,
    rhs: FunctionAssignmentRhs<'s>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct GiveSignature<'s> {
    signature: Signature<'s>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct GiveFunction<'s> {
    function: Function<'s>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) enum Statement<'s> {
    SignatureAssignment(SignatureAssignment<'s>),
    FunctionAssignment(FunctionAssignment<'s>),
    GiveSignature(GiveSignature<'s>),
    GiveFunction(GiveFunction<'s>),
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct Statements<'s>(Vec<Statement<'s>>);

fn parse_signature(pair: Pair<Rule>) -> Signature {
    let (symbol,) = pair.into_inner().collect_tuple().unwrap();
    Signature(symbol.as_str())
}

fn parse_function(pair: Pair<Rule>) -> Function {
    let (symbol,) = pair.into_inner().collect_tuple().unwrap();
    Function(symbol.as_str())
}

fn parse_signature_or_function(pair: Pair<Rule>) -> SignatureOrFunction {
    let rule = pair.as_rule();
    let (symbol,) = pair.into_inner().collect_tuple().unwrap();
    match rule {
        Rule::signature => SignatureOrFunction::Signature(Signature(symbol.as_str())),
        Rule::function => SignatureOrFunction::Function(Function(symbol.as_str())),
        _ => unreachable!(),
    }
}

fn parse_conjure_dependencies(pair: Pair<Rule>) -> ConjureDependencies {
    let dependencies = pair
        .into_inner()
        .map(|dependency| parse_signature_or_function(dependency))
        .collect();
    ConjureDependencies(dependencies)
}

fn parse_take_signature(pair: Pair<Rule>) -> TakeSignature {
    TakeSignature(PhantomData)
}

fn parse_conjure_signature(pair: Pair<Rule>) -> ConjureSignature {
    if let Some(dependencies) = pair.into_inner().next() {
        ConjureSignature {
            dependencies: parse_conjure_dependencies(dependencies),
        }
    } else {
        ConjureSignature {
            dependencies: ConjureDependencies(Vec::new()),
        }
    }
}

fn parse_define_signature(pair: Pair<Rule>) -> DefineSignature {
    if let Some(context) = pair.into_inner().next() {
        DefineSignature {
            context: parse_context(context),
        }
    } else {
        DefineSignature {
            context: Statements(Vec::new()),
        }
    }
}

fn parse_take_signature_from(pair: Pair<Rule>) -> TakeSignatureFrom {
    let (remote, source) = pair.into_inner().collect_tuple().unwrap();
    TakeSignatureFrom {
        remote: parse_signature(remote),
        source: parse_function(source),
    }
}

fn parse_take_function(pair: Pair<Rule>) -> TakeFunction {
    let (signature, ) = pair.into_inner().collect_tuple().unwrap();
    TakeFunction {
        signature: parse_signature(signature),
    }
}

fn parse_conjure_function(pair: Pair<Rule>) -> ConjureFunction {
    let mut pairs = pair.into_inner();
    let signature = pairs.next().unwrap();
    if let Some(dependencies) = pairs.next() {
        ConjureFunction {
            signature: parse_signature(signature),
            dependencies: parse_conjure_dependencies(dependencies),
        }
    } else {
        ConjureFunction {
            signature: parse_signature(signature),
            dependencies: ConjureDependencies(Vec::new()),
        }
    }
}

fn parse_define_function(pair: Pair<Rule>) -> DefineFunction {
    let mut pairs = pair.into_inner();
    let signature = pairs.next().unwrap();
    if let Some(context) = pairs.next() {
        DefineFunction {
            signature: parse_signature(signature),
            context: parse_context(context),
        }
    } else {
        DefineFunction {
            signature: parse_signature(signature),
            context: Statements(Vec::new()),
        }
    }
}

fn parse_take_function_from(pair: Pair<Rule>) -> TakeFunctionFrom {
    let (remote, source) = pair.into_inner().collect_tuple().unwrap();
    TakeFunctionFrom {
        remote: parse_function(remote),
        source: parse_function(source),
    }
}

fn parse_give_signature_to(pair: Pair<Rule>) -> GiveSignatureTo {
    let (local, remote, source) = pair.into_inner().collect_tuple().unwrap();
    GiveSignatureTo {
        local: parse_signature(local),
        remote: parse_signature(remote),
        source: parse_function(source),
    }
}

fn parse_give_function_to(pair: Pair<Rule>) -> GiveFunctionTo {
    let (local, remote, source) = pair.into_inner().collect_tuple().unwrap();
    GiveFunctionTo {
        local: parse_function(local),
        remote: parse_function(remote),
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
    let (signature,) = pair.into_inner().collect_tuple().unwrap();
    GiveSignature {
        signature: parse_signature(signature),
    }
}

fn parse_give_function(pair: Pair<Rule>) -> GiveFunction {
    let (function,) = pair.into_inner().collect_tuple().unwrap();
    GiveFunction {
        function: parse_function(function),
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

fn parse_context(pair: Pair<Rule>) -> Statements {
    let statements = pair
        .into_inner()
        .map(|statement| parse_statement(statement))
        .collect();
    Statements(statements)
}

pub(crate) fn parse(text: &'_ str) -> Statements<'_> {
    let pairs = MyParser::parse(Rule::file, text).unwrap();
    let statements = pairs.filter_map(|statement| {
        if let Rule::EOI = statement.as_rule() {
            None
        } else {
            Some(parse_statement(statement))
        }
    }).collect();
    Statements(statements)
}
