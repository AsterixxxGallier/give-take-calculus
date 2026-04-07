use crate::check::context_resolver::ContextResolver;
use crate::check::format::{Format, IndentingFormatter};
use crate::parse::*;
use ordermap::{OrderMap, OrderSet};
use std::collections::HashMap;
use std::io::{stdout, Write};

mod context_resolver;
mod describes;
mod error;
mod evaluation_state;
pub(crate) mod format;
mod id;
mod lambda_dependencies;
mod resolver;
mod substitute;

use crate::check::resolver::Resolver;
pub(crate) use error::*;

macro_rules! id {
    ($name:ident) => {
        #[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
        pub(crate) struct $name(id::Id);

        impl $name {
            pub(crate) fn generate() -> Self {
                Self(id::Id::generate())
            }
        }

        impl ::std::fmt::Debug for $name {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                write!(f, concat!(stringify!($name), "({})"), self.0)
            }
        }
    };
}

id!(SignatureId);
id!(FunctionId);

#[derive(Copy, Clone, Eq, PartialEq)]
pub(crate) struct SignatureDefinition<'s> {
    signature: Signature<'s>,
    statement: SourceLocation<'s>,
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub(crate) struct FunctionDefinition<'s> {
    function: Function<'s>,
    statement: SourceLocation<'s>,
}

#[derive(Clone, Default, Eq, PartialEq)]
pub(crate) struct KnownSignatureValue<'s> {
    conjured_signatures: OrderMap<SignatureId, SignatureConjuration<'s>>,
    conjured_functions: OrderMap<FunctionId, FunctionConjuration<'s>>,
    taken_signatures: OrderSet<SignatureId>,
    taken_functions: OrderMap<FunctionId, SignatureValue<'s>>,
    resolver: ContextResolver<'s>,
}

#[derive(Clone, Default, Eq, PartialEq)]
pub(crate) struct KnownFunctionValue<'s> {
    given_signatures: OrderMap<SignatureId, SignatureLambda<'s>>,
    given_functions: OrderMap<FunctionId, FunctionLambda<'s>>,
    taken_signatures: OrderSet<SignatureId>,
    taken_functions: OrderMap<FunctionId, SignatureValue<'s>>,
    resolver: ContextResolver<'s>,
}

#[derive(Clone, Eq, PartialEq)]
pub(crate) struct SignatureConjuration<'s> {
    dependencies: LambdaDependencies<'s>,
}

#[derive(Clone, Eq, PartialEq)]
pub(crate) struct FunctionConjuration<'s> {
    signature: SignatureValue<'s>,
    // must also contain all dependencies of signature
    dependencies: LambdaDependencies<'s>,
}

#[derive(Clone, Eq, PartialEq)]
pub(crate) struct SignatureLambda<'s> {
    signature: SignatureValue<'s>,
    dependencies: LambdaDependencies<'s>,
}

#[derive(Clone, Eq, PartialEq)]
pub(crate) struct FunctionLambda<'s> {
    function: FunctionValue<'s>,
    dependencies: LambdaDependencies<'s>,
}

#[derive(Clone, Eq, PartialEq, Default)]
pub(crate) struct LambdaDependencies<'s> {
    signatures: OrderSet<SignatureId>,
    functions: OrderMap<FunctionId, SignatureValue<'s>>,
}

#[derive(Clone, Eq, PartialEq)]
pub(crate) struct LambdaDependencyValues<'s> {
    signatures: OrderMap<SignatureId, SignatureValue<'s>>,
    functions: OrderMap<FunctionId, FunctionValue<'s>>,
}

#[derive(Clone, Eq, PartialEq)]
pub(crate) struct ConjuredSignatureValue<'s> {
    unknown_function: UnknownFunctionValue<'s>,
    unknown_function_signature: KnownSignatureValue<'s>,
    conjured_signature: SignatureId,
    conjure_dependency_values: LambdaDependencyValues<'s>,
}

#[derive(Clone, Eq, PartialEq)]
pub(crate) struct ConjuredFunctionValue<'s> {
    unknown_function: UnknownFunctionValue<'s>,
    unknown_function_signature: KnownSignatureValue<'s>,
    conjured_function: FunctionId,
    conjured_function_signature: SignatureValue<'s>,
    conjure_dependency_values: LambdaDependencyValues<'s>,
}

#[derive(Clone, Eq, PartialEq)]
pub(crate) enum UnknownSignatureValue<'s> {
    Taken(SignatureId),
    Conjured(Box<ConjuredSignatureValue<'s>>),
}

#[derive(Clone, Eq, PartialEq)]
pub(crate) enum UnknownFunctionValue<'s> {
    Taken(FunctionId, SignatureValue<'s>),
    Conjured(Box<ConjuredFunctionValue<'s>>),
}

#[derive(Clone, Eq, PartialEq)]
pub(crate) enum SignatureValue<'s> {
    Known(KnownSignatureValue<'s>),
    Unknown(UnknownSignatureValue<'s>),
}

#[derive(Clone, Eq, PartialEq)]
pub(crate) enum FunctionValue<'s> {
    Known(KnownFunctionValue<'s>),
    Unknown(UnknownFunctionValue<'s>),
}

#[derive(Default)]
struct EvaluationState<'s> {
    parent: Option<Box<EvaluationState<'s>>>,
    signature_lambdas: HashMap<SignatureId, SignatureLambda<'s>>,
    function_lambdas: HashMap<FunctionId, FunctionLambda<'s>>,
    signature_ids: HashMap<&'s str, SignatureId>,
    function_ids: HashMap<&'s str, FunctionId>,
    dependencies: LambdaDependencies<'s>,
}

pub(crate) fn check_function_context(context: FunctionContext, out: &mut dyn Write) -> bool {
    let mut resolver = Resolver::default();
    let mut state = EvaluationState::default();
    if let Err(error) = state.evaluate_function_context(context, &mut resolver) {
        error.print(&resolver, out).unwrap();
        false
    } else {
        true
    }
}

impl<'s> EvaluationState<'s> {
    fn process_signature_assignment(
        &mut self,
        trace: bool,
        assignment: SignatureAssignment<'s>,
        resolver: &mut Resolver<'s>,
        register_taken_signature: impl FnOnce(Signature<'s>, SignatureId) -> CheckResult<'s, ()>,
    ) -> CheckResult<'s, ()> {
        let SignatureAssignment { location, lhs, rhs } = assignment;

        let lhs_id = SignatureId::generate();

        let lambda = match rhs {
            SignatureAssignmentRhs::Take(TakeSignature { phantom: _ }) => {
                register_taken_signature(lhs, lhs_id)?;
                SignatureLambda {
                    signature: SignatureValue::Unknown(UnknownSignatureValue::Taken(lhs_id)),
                    dependencies: LambdaDependencies {
                        signatures: OrderSet::from([lhs_id]),
                        functions: OrderMap::new(),
                    },
                }
            }
            SignatureAssignmentRhs::Define(DefineSignature { context }) => {
                let (signature, dependencies) =
                    self.do_as_child(|child| child.evaluate_signature_context(context, resolver));
                let signature = signature?;
                SignatureLambda {
                    signature: SignatureValue::Known(signature),
                    dependencies,
                }
            }
            SignatureAssignmentRhs::TakeFrom(TakeSignatureFrom { foreign, source }) => {
                let source_id = self.resolve_function(source)?;
                let FunctionLambda {
                    function: source_function,
                    dependencies: source_dependencies,
                } = self.function_lambda(source_id);

                match source_function {
                    FunctionValue::Known(source_value) => {
                        let foreign_id = if let Some(id) = source_value
                            .resolver
                            .produced_signature_id(foreign.as_str())
                        {
                            id
                        } else {
                            return Err(CheckError::CannotResolveGivenSignature {
                                statement: location,
                                source,
                                source_value,
                                foreign,
                            });
                        };

                        let foreign_lambda = source_value
                            .given_signatures
                            .get(&foreign_id)
                            .expect("could not find given signature by id");

                        if !foreign_lambda.dependencies.is_empty() {
                            return Err(CheckError::TakenSignatureDependenciesNotProvided {
                                statement: location,
                                source,
                                source_value,
                                foreign,
                            });
                        }

                        SignatureLambda {
                            signature: foreign_lambda.signature.clone(),
                            dependencies: source_dependencies,
                        }
                    }
                    FunctionValue::Unknown(source_value) => {
                        return Err(CheckError::CannotTakeSignatureFromUnknownFunction {
                            statement: location,
                            source,
                            source_value,
                        });
                    }
                }
            }
            SignatureAssignmentRhs::GiveSignatureToSignature(GiveSignatureToSignature {
                                                                 signature,
                                                                 foreign,
                                                                 source,
                                                             }) => {
                let signature_id = self.resolve_signature(signature)?;
                let SignatureLambda {
                    signature: signature_value,
                    dependencies: signature_dependencies,
                } = self.signature_lambda(signature_id);

                let source_id = self.resolve_signature(source)?;
                let SignatureLambda {
                    signature: source_value,
                    dependencies: source_dependencies,
                } = self.signature_lambda(source_id);

                match source_value {
                    SignatureValue::Known(mut source_value) => {
                        let foreign_id = if let Some(id) =
                            source_value.resolver.taken_signature_id(foreign.as_str())
                        {
                            id
                        } else {
                            return Err(CheckError::CannotResolveTakenSignatureOfSignature {
                                statement: location,
                                source,
                                source_value,
                                foreign,
                            });
                        };

                        source_value.taken_signatures.remove(&foreign_id);
                        source_value.resolver.remove_taken_signature(foreign_id);

                        for signature in source_value.taken_functions.values_mut() {
                            signature.substitute_taken_signature(
                                foreign_id,
                                &signature_value,
                                resolver,
                            );
                        }

                        for conjuration in source_value.conjured_signatures.values_mut() {
                            // foreign won't be a dependency of _all_ conjurations,
                            // so it's okay if it's not in the set here
                            _ = conjuration.dependencies.signatures.remove(&foreign_id);
                        }

                        for conjuration in source_value.conjured_functions.values_mut() {
                            _ = conjuration.dependencies.signatures.remove(&foreign_id);
                            conjuration.signature.substitute_taken_signature(
                                foreign_id,
                                &signature_value,
                                resolver,
                            );
                        }

                        let mut dependencies = signature_dependencies;
                        dependencies.add(&source_dependencies);

                        SignatureLambda {
                            signature: SignatureValue::Known(source_value),
                            dependencies,
                        }
                    }
                    SignatureValue::Unknown(source_value) => {
                        return Err(CheckError::CannotGiveSignatureToUnknownSignature {
                            statement: location,
                            source,
                            source_value,
                        });
                    }
                }
            }
            SignatureAssignmentRhs::GiveFunctionToSignature(GiveFunctionToSignature {
                                                                function,
                                                                foreign,
                                                                source,
                                                            }) => {
                let function_id = self.resolve_function(function)?;
                let FunctionLambda {
                    function: function_value,
                    dependencies: function_dependencies,
                } = self.function_lambda(function_id);

                let source_id = self.resolve_signature(source)?;
                let SignatureLambda {
                    signature: source_value,
                    dependencies: source_dependencies,
                } = self.signature_lambda(source_id);

                match source_value {
                    SignatureValue::Known(mut source_value) => {
                        let foreign_id = if let Some(id) =
                            source_value.resolver.taken_function_id(foreign.as_str())
                        {
                            id
                        } else {
                            return Err(CheckError::CannotResolveTakenFunctionOfSignature {
                                statement: location,
                                source,
                                source_value,
                                foreign,
                            });
                        };

                        let expected_signature = &source_value.taken_functions[&foreign_id];
                        if !expected_signature.describes(&function_value) {
                            return Err(
                                CheckError::FunctionGivenToSignatureDoesNotHaveExpectedSignature {
                                    statement: location,
                                    function,
                                    function_value,
                                    foreign,
                                    source,
                                    expected_signature_value: expected_signature.clone(),
                                    source_value,
                                },
                            );
                        }

                        source_value.resolver.remove_taken_function(foreign_id);
                        source_value.taken_functions.remove(&foreign_id);

                        for signature in source_value.taken_functions.values_mut() {
                            signature.substitute_taken_function(
                                foreign_id,
                                &function_value,
                                resolver,
                            );
                        }

                        for conjuration in source_value.conjured_signatures.values_mut() {
                            // foreign won't be a dependency of _all_ conjurations,
                            // so it's okay if it's not in the set here
                            _ = conjuration.dependencies.functions.remove(&foreign_id);
                        }

                        for conjuration in source_value.conjured_functions.values_mut() {
                            _ = conjuration.dependencies.functions.remove(&foreign_id);
                            conjuration.signature.substitute_taken_function(
                                foreign_id,
                                &function_value,
                                resolver,
                            );
                        }

                        let mut dependencies = function_dependencies;
                        dependencies.add(&source_dependencies);

                        SignatureLambda {
                            signature: SignatureValue::Known(source_value),
                            dependencies,
                        }
                    }
                    SignatureValue::Unknown(source_value) => {
                        return Err(CheckError::CannotGiveFunctionToUnknownSignature {
                            statement: location,
                            source,
                            source_value,
                        });
                    }
                }
            }
        };

        resolver.insert_signature(lhs_id, lhs, assignment.location);

        if trace {
            print!("{:?} := ", lhs.with_parens);
            lambda
                .format(resolver, &mut IndentingFormatter::new(&mut stdout()))
                .unwrap();
            println!();
            println!();
        }

        self.signature_ids.insert(lhs.as_str(), lhs_id);
        self.signature_lambdas.insert(lhs_id, lambda);

        Ok(())
    }

    fn process_function_assignment(
        &mut self,
        trace: bool,
        assignment: FunctionAssignment<'s>,
        resolver: &mut Resolver<'s>,
        register_taken_function: impl FnOnce(
            Function<'s>,
            FunctionId,
            &SignatureValue<'s>,
        ) -> CheckResult<'s, ()>,
    ) -> CheckResult<'s, ()> {
        let FunctionAssignment { lhs, rhs, location } = assignment;

        let lhs_id = FunctionId::generate();

        let lambda = match rhs {
            FunctionAssignmentRhs::Take(TakeFunction { signature }) => {
                let signature_id = self.resolve_signature(signature)?;
                let SignatureLambda {
                    signature,
                    dependencies: signature_dependencies,
                } = self.signature_lambda(signature_id);

                register_taken_function(lhs, lhs_id, &signature)?;

                let unknown_function = UnknownFunctionValue::Taken(lhs_id, signature.clone());

                let mut dependencies = LambdaDependencies {
                    signatures: OrderSet::new(),
                    functions: OrderMap::from([(lhs_id, signature.clone())]),
                };
                dependencies.add(&signature_dependencies);

                let function = match signature {
                    SignatureValue::Known(signature) => FunctionValue::Known(KnownFunctionValue {
                        given_signatures: signature
                            .conjured_signatures
                            .iter()
                            .map(|(&id, conjuration)| {
                                let value = UnknownSignatureValue::Conjured(Box::new(
                                    ConjuredSignatureValue {
                                        unknown_function: unknown_function.clone(),
                                        unknown_function_signature: signature.clone(),
                                        conjured_signature: id,
                                        conjure_dependency_values: conjuration
                                            .dependencies
                                            .as_values(),
                                    },
                                ));
                                let lambda = SignatureLambda {
                                    signature: SignatureValue::Unknown(value),
                                    dependencies: conjuration.dependencies.clone(),
                                };
                                (id, lambda)
                            })
                            .collect(),
                        given_functions: signature
                            .conjured_functions
                            .iter()
                            .map(|(&id, conjuration)| {
                                let value = UnknownFunctionValue::Conjured(Box::new(
                                    ConjuredFunctionValue {
                                        unknown_function: unknown_function.clone(),
                                        unknown_function_signature: signature.clone(),
                                        conjured_function: id,
                                        conjured_function_signature: conjuration.signature.clone(),
                                        conjure_dependency_values: conjuration
                                            .dependencies
                                            .as_values(),
                                    },
                                ));
                                let lambda = FunctionLambda {
                                    function: FunctionValue::Unknown(value),
                                    dependencies: conjuration.dependencies.clone(),
                                };
                                (id, lambda)
                            })
                            .collect(),
                        taken_signatures: signature.taken_signatures,
                        taken_functions: signature.taken_functions,
                        resolver: signature.resolver.clone(),
                    }),
                    SignatureValue::Unknown(_) => FunctionValue::Unknown(unknown_function),
                };

                FunctionLambda {
                    function,
                    dependencies,
                }
            }
            FunctionAssignmentRhs::Define(DefineFunction { context }) => {
                let (function, dependencies) =
                    self.do_as_child(|child| child.evaluate_function_context(context, resolver));
                let function = function?;
                FunctionLambda {
                    function: FunctionValue::Known(function),
                    dependencies,
                }
            }
            FunctionAssignmentRhs::TakeFrom(TakeFunctionFrom { foreign, source }) => {
                let source_id = self.resolve_function(source)?;
                let FunctionLambda {
                    function: source_function,
                    dependencies: source_dependencies,
                } = self.function_lambda(source_id);

                match source_function {
                    FunctionValue::Known(source_value) => {
                        let foreign_id = if let Some(id) =
                            source_value.resolver.produced_function_id(foreign.as_str())
                        {
                            id
                        } else {
                            return Err(CheckError::CannotResolveGivenFunction {
                                statement: location,
                                source,
                                source_value,
                                foreign,
                            });
                        };

                        let foreign_lambda = source_value
                            .given_functions
                            .get(&foreign_id)
                            .expect("could not find given function by id");

                        if !foreign_lambda.dependencies.is_empty() {
                            return Err(CheckError::TakenFunctionDependenciesNotProvided {
                                statement: location,
                                source,
                                source_value,
                                foreign,
                            });
                        }

                        FunctionLambda {
                            function: foreign_lambda.function.clone(),
                            dependencies: source_dependencies,
                        }
                    }
                    FunctionValue::Unknown(source_value) => {
                        return Err(CheckError::CannotTakeFunctionFromUnknownFunction {
                            statement: location,
                            source,
                            source_value,
                        });
                    }
                }
            }
            FunctionAssignmentRhs::GiveSignatureToFunction(GiveSignatureToFunction {
                                                               signature,
                                                               foreign,
                                                               source,
                                                           }) => {
                let signature_id = self.resolve_signature(signature)?;
                let SignatureLambda {
                    signature: signature_value,
                    dependencies: signature_dependencies,
                } = self.signature_lambda(signature_id);

                let source_id = self.resolve_function(source)?;
                let FunctionLambda {
                    function: source_value,
                    dependencies: source_dependencies,
                } = self.function_lambda(source_id);

                match source_value {
                    FunctionValue::Known(mut source_value) => {
                        let foreign_id = if let Some(id) =
                            source_value.resolver.taken_signature_id(foreign.as_str())
                        {
                            id
                        } else {
                            return Err(CheckError::CannotResolveTakenSignatureOfFunction {
                                statement: location,
                                source,
                                source_value,
                                foreign,
                            });
                        };

                        source_value.taken_signatures.remove(&foreign_id);
                        source_value.resolver.remove_taken_signature(foreign_id);

                        for signature in source_value.taken_functions.values_mut() {
                            signature.substitute_taken_signature(
                                foreign_id,
                                &signature_value,
                                resolver,
                            );
                        }

                        for lambda in source_value.given_signatures.values_mut() {
                            _ = lambda.dependencies.signatures.remove(&foreign_id);
                            lambda.signature.substitute_taken_signature(
                                foreign_id,
                                &signature_value,
                                resolver,
                            );
                        }

                        for lambda in source_value.given_functions.values_mut() {
                            _ = lambda.dependencies.signatures.remove(&foreign_id);
                            lambda.function.substitute_taken_signature(
                                foreign_id,
                                &signature_value,
                                resolver,
                            );
                        }

                        let mut dependencies = signature_dependencies;
                        dependencies.add(&source_dependencies);

                        FunctionLambda {
                            function: FunctionValue::Known(source_value),
                            dependencies,
                        }
                    }
                    FunctionValue::Unknown(source_value) => {
                        return Err(CheckError::CannotGiveSignatureToUnknownFunction {
                            statement: location,
                            source,
                            source_value,
                        });
                    }
                }
            }
            FunctionAssignmentRhs::GiveFunctionToFunction(GiveFunctionToFunction {
                                                              function,
                                                              foreign,
                                                              source,
                                                          }) => {
                let function_id = self.resolve_function(function)?;
                let FunctionLambda {
                    function: function_value,
                    dependencies: function_dependencies,
                } = self.function_lambda(function_id);

                let source_id = self.resolve_function(source)?;
                let FunctionLambda {
                    function: source_value,
                    dependencies: source_dependencies,
                } = self.function_lambda(source_id);

                match source_value {
                    FunctionValue::Known(mut source_value) => {
                        let foreign_id = if let Some(id) =
                            source_value.resolver.taken_function_id(foreign.as_str())
                        {
                            id
                        } else {
                            return Err(CheckError::CannotResolveTakenFunctionOfFunction {
                                statement: location,
                                source,
                                source_value,
                                foreign,
                            });
                        };

                        let expected_signature = &source_value.taken_functions[&foreign_id];
                        if !expected_signature.describes(&function_value) {
                            return Err(
                                CheckError::FunctionGivenToFunctionDoesNotHaveExpectedSignature {
                                    statement: location,
                                    function,
                                    function_value,
                                    foreign,
                                    source,
                                    expected_signature_value: expected_signature.clone(),
                                    source_value,
                                },
                            );
                        }

                        println!("\n\n===== give function to function =====\n");
                        function.symbol.dump("function");
                        source_value
                            .resolver
                            .taken_function(foreign_id)
                            .symbol
                            .dump("foreign");
                        source.symbol.dump("source");

                        let mut stdout = stdout();
                        let mut formatter = IndentingFormatter::new(&mut stdout);
                        write!(formatter, "source_value: ").unwrap();
                        source_value.format(resolver, &mut formatter).unwrap();
                        formatter.new_line().unwrap();
                        write!(formatter, "function_value: ").unwrap();
                        function_value.format(resolver, &mut formatter).unwrap();
                        formatter.new_line().unwrap();

                        source_value.resolver.remove_taken_function(foreign_id);
                        source_value.taken_functions.remove(&foreign_id);

                        for signature in source_value.taken_functions.values_mut() {
                            signature.substitute_taken_function(
                                foreign_id,
                                &function_value,
                                resolver,
                            );
                        }

                        for lambda in source_value.given_signatures.values_mut() {
                            _ = lambda.dependencies.functions.remove(&foreign_id);
                            lambda.signature.substitute_taken_function(
                                foreign_id,
                                &function_value,
                                resolver,
                            );
                        }

                        for lambda in source_value.given_functions.values_mut() {
                            _ = lambda.dependencies.functions.remove(&foreign_id);
                            lambda.function.substitute_taken_function(
                                foreign_id,
                                &function_value,
                                resolver,
                            );
                        }

                        let mut dependencies = function_dependencies;
                        dependencies.add(&source_dependencies);

                        FunctionLambda {
                            function: FunctionValue::Known(source_value),
                            dependencies,
                        }
                    }
                    FunctionValue::Unknown(source_value) => {
                        return Err(CheckError::CannotGiveFunctionToUnknownFunction {
                            statement: location,
                            source,
                            source_value,
                        });
                    }
                }
            }
        };

        resolver.insert_function(lhs_id, lhs, assignment.location);

        if trace {
            print!("{:?} := ", lhs.symbol);
            lambda
                .format(resolver, &mut IndentingFormatter::new(&mut stdout()))
                .unwrap();
            println!();
            println!();
        }

        self.function_ids.insert(lhs.as_str(), lhs_id);
        self.function_lambdas.insert(lhs_id, lambda);

        Ok(())
    }

    fn evaluate_signature_context(
        &mut self,
        context: SignatureContext<'s>,
        resolver: &mut Resolver<'s>,
    ) -> CheckResult<'s, KnownSignatureValue<'s>> {
        let trace = context.trace;

        let mut context_value = KnownSignatureValue::default();

        for statement in context.statements {
            match statement {
                SignatureStatement::SignatureAssignment(assignment) => {
                    let statement = assignment.location;
                    self.process_signature_assignment(
                        trace,
                        assignment,
                        resolver,
                        |signature, id| {
                            context_value.taken_signatures.insert(id);
                            context_value
                                .resolver
                                .insert_taken_signature(id, signature, statement)
                        },
                    )?;
                }
                SignatureStatement::FunctionAssignment(assignment) => {
                    let statement = assignment.location;
                    self.process_function_assignment(
                        trace,
                        assignment,
                        resolver,
                        |function, id, signature| {
                            context_value.taken_functions.insert(id, signature.clone());
                            context_value
                                .resolver
                                .insert_taken_function(id, function, statement)
                        },
                    )?;
                }
                SignatureStatement::ConjureSignature(ConjureSignature {
                                                         signature,
                                                         dependencies,
                                                         location,
                                                     }) => {
                    let signature_id = SignatureId::generate();

                    if let Some(_other_signature_id) = context_value
                        .resolver
                        .produced_signature_id(signature.as_str())
                    {
                        // TODO: Show the other conjuration.
                        return Err(CheckError::CannotConjureTwoSignaturesWithIdenticalName {
                            signature,
                            statement: location,
                        });
                    }

                    let mut lambda_dependencies = LambdaDependencies::default();

                    for dependency in dependencies.0 {
                        match dependency {
                            ConjureDependency::Signature(signature) => {
                                let signature_id = self.resolve_signature(signature)?;
                                let lambda = self.signature_lambda(signature_id);
                                lambda_dependencies.add(&lambda.dependencies);
                            }
                            ConjureDependency::Function(function) => {
                                let function_id = self.resolve_function(function)?;
                                let lambda = self.function_lambda(function_id);
                                lambda_dependencies.add(&lambda.dependencies);
                            }
                        }
                    }

                    let conjuration = SignatureConjuration {
                        dependencies: lambda_dependencies,
                    };

                    context_value
                        .conjured_signatures
                        .insert(signature_id, conjuration);
                    context_value.resolver.insert_produced_signature(
                        signature_id,
                        signature,
                        location,
                    );

                    resolver.insert_signature(signature_id, signature, location);
                }
                SignatureStatement::ConjureFunction(ConjureFunction {
                                                        function,
                                                        signature,
                                                        dependencies,
                                                        location,
                                                    }) => {
                    let function_id = FunctionId::generate();

                    if let Some(_other_function_id) = context_value
                        .resolver
                        .produced_function_id(function.as_str())
                    {
                        return Err(CheckError::CannotConjureTwoFunctionsWithIdenticalName {
                            function,
                            statement: location,
                        });
                    }

                    let mut lambda_dependencies = LambdaDependencies::default();

                    for dependency in dependencies.0 {
                        match dependency {
                            ConjureDependency::Signature(signature) => {
                                let signature_id = self.resolve_signature(signature)?;
                                let lambda = self.signature_lambda(signature_id);
                                lambda_dependencies.add(&lambda.dependencies);
                            }
                            ConjureDependency::Function(function) => {
                                let function_id = self.resolve_function(function)?;
                                let lambda = self.function_lambda(function_id);
                                lambda_dependencies.add(&lambda.dependencies);
                            }
                        }
                    }

                    let signature_id = self.resolve_signature(signature)?;
                    let signature_lambda = self.signature_lambda(signature_id);
                    lambda_dependencies.add(&signature_lambda.dependencies);

                    let conjuration = FunctionConjuration {
                        signature: signature_lambda.signature,
                        dependencies: lambda_dependencies,
                    };

                    context_value
                        .conjured_functions
                        .insert(function_id, conjuration);
                    context_value.resolver.insert_produced_function(
                        function_id,
                        function,
                        location,
                    );

                    resolver.insert_function(function_id, function, location);
                }
            }
        }

        Ok(context_value)
    }

    fn evaluate_function_context(
        &mut self,
        context: FunctionContext<'s>,
        resolver: &mut Resolver<'s>,
    ) -> CheckResult<'s, KnownFunctionValue<'s>> {
        let trace = context.trace;

        let mut context_value = KnownFunctionValue::default();

        for statement in context.statements {
            match statement {
                FunctionStatement::SignatureAssignment(assignment) => {
                    let statement = assignment.location;
                    self.process_signature_assignment(
                        trace,
                        assignment,
                        resolver,
                        |signature, id| {
                            context_value.taken_signatures.insert(id);
                            context_value
                                .resolver
                                .insert_taken_signature(id, signature, statement)
                        },
                    )?;
                }
                FunctionStatement::FunctionAssignment(assignment) => {
                    let statement = assignment.location;
                    self.process_function_assignment(
                        trace,
                        assignment,
                        resolver,
                        |function, id, signature| {
                            context_value.taken_functions.insert(id, signature.clone());
                            context_value
                                .resolver
                                .insert_taken_function(id, function, statement)
                        },
                    )?;
                }
                FunctionStatement::GiveSignature(GiveSignature {
                                                     signature,
                                                     location,
                                                 }) => {
                    let signature_id = self.resolve_signature(signature)?;
                    let lambda = self.signature_lambda(signature_id);

                    if let Some(other_signature_id) = context_value
                        .resolver
                        .produced_signature_id(signature.as_str())
                    {
                        // TODO: Show the other give statement.
                        return if other_signature_id == signature_id {
                            Err(CheckError::CannotGiveSignatureTwice {
                                signature,
                                statement: location,
                            })
                        } else {
                            // TODO: Show the definitions of the two signatures.
                            Err(CheckError::CannotGiveTwoSignaturesWithIdenticalName {
                                signature,
                                statement: location,
                            })
                        };
                    }

                    context_value.given_signatures.insert(signature_id, lambda);
                    context_value.resolver.insert_produced_signature(
                        signature_id,
                        signature,
                        location,
                    );
                }
                FunctionStatement::GiveFunction(GiveFunction { function, location }) => {
                    let function_id = self.resolve_function(function)?;
                    let lambda = self.function_lambda(function_id);

                    if let Some(other_function_id) = context_value
                        .resolver
                        .produced_function_id(function.as_str())
                    {
                        return if other_function_id == function_id {
                            Err(CheckError::CannotGiveFunctionTwice {
                                function,
                                statement: location,
                            })
                        } else {
                            Err(CheckError::CannotGiveTwoFunctionsWithIdenticalName {
                                function,
                                statement: location,
                            })
                        };
                    }

                    context_value.given_functions.insert(function_id, lambda);
                    context_value.resolver.insert_produced_function(
                        function_id,
                        function,
                        location,
                    );
                }
            }
        }

        Ok(context_value)
    }
}
