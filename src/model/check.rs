use crate::model::*;
use itertools::Itertools;

#[derive(Debug, Clone, Eq, PartialEq)]
enum ConjureSignatureMarker {
    Id(SignatureId),
    Index(u64),
}

#[derive(Debug, Clone, Eq, PartialEq)]
enum ConjureFunctionMarker {
    Id(FunctionId),
    Index(u64),
}

#[derive(Debug, Clone, Eq, PartialEq)]
enum SignatureValue {
    Variable {
        signature: SignatureId,
    },
    Use {
        literal: SignatureLiteralId,
    },
    Take {
        literal: SignatureLiteralId,
    },
    Conjure {
        marker: ConjureSignatureMarker,
        signature_dependencies: Vec<SignatureValue>,
        function_dependencies: Vec<FunctionValue>,
    },
    // TODO Define,
    TakeFrom {
        literal: SignatureLiteralId,
        source: Box<FunctionValue>,
    },
}

#[derive(Debug, Clone, Eq, PartialEq)]
enum FunctionValue {
    Variable {
        function: FunctionId,
    },
    Use {
        signature: Box<SignatureValue>,
        literal: FunctionLiteralId,
    },
    Take {
        signature: Box<SignatureValue>,
        literal: FunctionLiteralId,
    },
    Conjure {
        marker: ConjureFunctionMarker,
        signature: Box<SignatureValue>,
        signature_dependencies: Vec<SignatureValue>,
        function_dependencies: Vec<FunctionValue>,
    },
    // TODO Define,
    TakeFrom {
        literal: FunctionLiteralId,
        source: Box<FunctionValue>,
    },
    GiveSignatureTo {
        signature: Box<SignatureValue>,
        literal: SignatureLiteralId,
        source: Box<FunctionValue>,
    },
    GiveFunctionTo {
        function: Box<FunctionValue>,
        literal: FunctionLiteralId,
        source: Box<FunctionValue>,
    },
}

impl SignatureValue {
    fn substitute_signature(&mut self, id: SignatureId, value: &SignatureValue) {
        match self {
            SignatureValue::Variable { signature } => {
                if *signature == id {
                    *self = value.clone();
                }
            }
            SignatureValue::Use { literal } => {
                _ = literal;
            }
            SignatureValue::Take { literal } => {
                _ = literal;
            }
            SignatureValue::Conjure {
                marker,
                signature_dependencies,
                function_dependencies,
            } => {
                _ = marker;
                for dependency in signature_dependencies {
                    dependency.substitute_signature(id, value);
                }
                for dependency in function_dependencies {
                    dependency.substitute_signature(id, value);
                }
            }
            SignatureValue::TakeFrom { literal, source } => {
                _ = literal;
                source.substitute_signature(id, value);
            }
        }
    }

    fn substitute_function(&mut self, id: FunctionId, value: &FunctionValue) {
        match self {
            SignatureValue::Variable { signature } => {
                _ = signature;
            }
            SignatureValue::Use { literal } => {
                _ = literal;
            }
            SignatureValue::Take { literal } => {
                _ = literal;
            }
            SignatureValue::Conjure {
                marker,
                signature_dependencies,
                function_dependencies,
            } => {
                _ = marker;
                for dependency in signature_dependencies {
                    dependency.substitute_function(id, value);
                }
                for dependency in function_dependencies {
                    dependency.substitute_function(id, value);
                }
            }
            SignatureValue::TakeFrom { literal, source } => {
                _ = literal;
                source.substitute_function(id, value);
            }
        }
    }
}

impl FunctionValue {
    fn substitute_signature(&mut self, id: SignatureId, value: &SignatureValue) {
        match self {
            FunctionValue::Variable { function } => {
                _ = function;
            }
            FunctionValue::Use { signature, literal } => {
                signature.substitute_signature(id, value);
                _ = literal;
            }
            FunctionValue::Take { signature, literal } => {
                signature.substitute_signature(id, value);
                _ = literal;
            }
            FunctionValue::Conjure {
                marker,
                signature,
                signature_dependencies,
                function_dependencies,
            } => {
                _ = marker;
                signature.substitute_signature(id, value);
                for dependency in signature_dependencies {
                    dependency.substitute_signature(id, value);
                }
                for dependency in function_dependencies {
                    dependency.substitute_signature(id, value);
                }
            }
            FunctionValue::TakeFrom { literal, source } => {
                _ = literal;
                source.substitute_signature(id, value);
            }
            FunctionValue::GiveSignatureTo {
                signature,
                literal,
                source,
            } => {
                signature.substitute_signature(id, value);
                _ = literal;
                source.substitute_signature(id, value);
            }
            FunctionValue::GiveFunctionTo {
                function,
                literal,
                source,
            } => {
                function.substitute_signature(id, value);
                _ = literal;
                source.substitute_signature(id, value);
            }
        }
    }

    fn substitute_function(&mut self, id: FunctionId, value: &FunctionValue) {
        match self {
            FunctionValue::Variable { function } => {
                if *function == id {
                    *self = value.clone();
                }
            }
            FunctionValue::Use { signature, literal } => {
                signature.substitute_function(id, value);
                _ = literal;
            }
            FunctionValue::Take { signature, literal } => {
                signature.substitute_function(id, value);
                _ = literal;
            }
            FunctionValue::Conjure {
                marker,
                signature,
                signature_dependencies,
                function_dependencies,
            } => {
                _ = marker;
                signature.substitute_function(id, value);
                for dependency in signature_dependencies {
                    dependency.substitute_function(id, value);
                }
                for dependency in function_dependencies {
                    dependency.substitute_function(id, value);
                }
            }
            FunctionValue::TakeFrom { literal, source } => {
                _ = literal;
                source.substitute_function(id, value);
            }
            FunctionValue::GiveSignatureTo {
                signature,
                literal,
                source,
            } => {
                signature.substitute_function(id, value);
                _ = literal;
                source.substitute_function(id, value);
            }
            FunctionValue::GiveFunctionTo {
                function,
                literal,
                source,
            } => {
                function.substitute_function(id, value);
                _ = literal;
                source.substitute_function(id, value);
            }
        }
    }
}

impl SignatureValue {
    fn enumerate_conjurations(
        &mut self,
        signature_enumeration: &mut HashMap<SignatureId, u64>,
        function_enumeration: &mut HashMap<FunctionId, u64>,
    ) {
        match self {
            SignatureValue::Variable { .. } => {
                // there shouldn't be not-substituted variables left during conjuration enumeration
                unreachable!()
            }
            SignatureValue::Use { literal } => {
                _ = literal;
            }
            SignatureValue::Take { literal } => {
                _ = literal;
            }
            SignatureValue::Conjure {
                marker,
                signature_dependencies,
                function_dependencies,
            } => {
                let ConjureSignatureMarker::Id(id) = marker else {
                    // shouldn't be enumerated yet
                    unreachable!()
                };
                let next_index = signature_enumeration.len() as u64;
                match signature_enumeration.entry(*id) {
                    Entry::Occupied(entry) => {
                        *marker = ConjureSignatureMarker::Index(*entry.get());
                    }
                    Entry::Vacant(entry) => {
                        *marker = ConjureSignatureMarker::Index(next_index);
                        entry.insert(next_index);
                    }
                }

                for dependency in signature_dependencies {
                    dependency.enumerate_conjurations(signature_enumeration, function_enumeration);
                }
                for dependency in function_dependencies {
                    dependency.enumerate_conjurations(signature_enumeration, function_enumeration);
                }
            }
            SignatureValue::TakeFrom { literal, source } => {
                _ = literal;
                source.enumerate_conjurations(signature_enumeration, function_enumeration);
            }
        }
    }
}

impl FunctionValue {
    fn enumerate_conjurations(
        &mut self,
        signature_enumeration: &mut HashMap<SignatureId, u64>,
        function_enumeration: &mut HashMap<FunctionId, u64>,
    ) {
        match self {
            FunctionValue::Variable { .. } => {
                // there shouldn't be not-substituted variables left during conjuration enumeration
                unreachable!();
            }
            FunctionValue::Use { signature, literal } => {
                signature.enumerate_conjurations(signature_enumeration, function_enumeration);
                _ = literal;
            }
            FunctionValue::Take { signature, literal } => {
                signature.enumerate_conjurations(signature_enumeration, function_enumeration);
                _ = literal;
            }
            FunctionValue::Conjure {
                marker,
                signature,
                signature_dependencies,
                function_dependencies,
            } => {
                let ConjureFunctionMarker::Id(id) = marker else {
                    // shouldn't be enumerated yet
                    unreachable!()
                };
                let next_index = function_enumeration.len() as u64;
                match function_enumeration.entry(*id) {
                    Entry::Occupied(entry) => {
                        *marker = ConjureFunctionMarker::Index(*entry.get());
                    }
                    Entry::Vacant(entry) => {
                        *marker = ConjureFunctionMarker::Index(next_index);
                        entry.insert(next_index);
                    }
                }

                signature.enumerate_conjurations(signature_enumeration, function_enumeration);
                for dependency in signature_dependencies {
                    dependency.enumerate_conjurations(signature_enumeration, function_enumeration);
                }
                for dependency in function_dependencies {
                    dependency.enumerate_conjurations(signature_enumeration, function_enumeration);
                }
            }
            FunctionValue::TakeFrom { literal, source } => {
                _ = literal;
                source.enumerate_conjurations(signature_enumeration, function_enumeration);
            }
            FunctionValue::GiveSignatureTo {
                signature,
                literal,
                source,
            } => {
                signature.enumerate_conjurations(signature_enumeration, function_enumeration);
                _ = literal;
                source.enumerate_conjurations(signature_enumeration, function_enumeration);
            }
            FunctionValue::GiveFunctionTo {
                function,
                literal,
                source,
            } => {
                function.enumerate_conjurations(signature_enumeration, function_enumeration);
                _ = literal;
                source.enumerate_conjurations(signature_enumeration, function_enumeration);
            }
        }
    }
}

impl<'s> Model<'s> {
    pub(crate) fn check(&self) {
        let context = self.contexts[0];
        let contents = &self.context_contents[&context];

        println!("{contents:#?}");

        let mut taken_signatures = contents
            .statements
            .iter()
            .filter_map(|statement| {
                if let Statement::SignatureAssignment {
                    lhs: _,
                    rhs: SignatureAssignmentRhs::Take { literal },
                } = *statement
                {
                    Some(literal)
                } else {
                    None
                }
            })
            .collect_vec();
        taken_signatures.sort();
        let mut taken_functions = contents
            .statements
            .iter()
            .filter_map(|statement| {
                if let Statement::FunctionAssignment {
                    lhs: _,
                    rhs:
                        FunctionAssignmentRhs::Take {
                            signature: _,
                            literal,
                        },
                } = *statement
                {
                    Some(literal)
                } else {
                    None
                }
            })
            .collect_vec();
        taken_functions.sort();

        dbg!(&taken_signatures);
        dbg!(&taken_functions);

        let mut given_signatures = contents
            .statements
            .iter()
            .filter_map(|statement| {
                if let Statement::GiveSignature { signature, literal } = *statement {
                    Some((literal, signature))
                } else {
                    None
                }
            })
            .collect_vec();
        given_signatures.sort();
        let mut given_functions = contents
            .statements
            .iter()
            .filter_map(|statement| {
                if let Statement::GiveFunction { function, literal } = *statement {
                    Some((literal, function))
                } else {
                    None
                }
            })
            .collect_vec();
        given_functions.sort();

        dbg!(&given_signatures);
        dbg!(&given_functions);

        let mut signature_values: HashMap<SignatureId, SignatureValue> = contents
            .statements
            .iter()
            .filter_map(|statement| {
                if let Statement::SignatureAssignment { lhs, ref rhs } = *statement {
                    let value = match *rhs {
                        SignatureAssignmentRhs::Use { literal } => SignatureValue::Use { literal },
                        SignatureAssignmentRhs::Take { literal } => {
                            SignatureValue::Take { literal }
                        }
                        SignatureAssignmentRhs::Conjure {
                            ref signature_dependencies,
                            ref function_dependencies,
                        } => SignatureValue::Conjure {
                            marker: ConjureSignatureMarker::Id(lhs),
                            signature_dependencies: signature_dependencies
                                .iter()
                                .map(|&signature| SignatureValue::Variable { signature })
                                .collect(),
                            function_dependencies: function_dependencies
                                .iter()
                                .map(|&function| FunctionValue::Variable { function })
                                .collect(),
                        },
                        SignatureAssignmentRhs::Define { .. } => {
                            // for now, we only care about the first context, which does not have
                            // sub-contexts
                            unreachable!()
                        }
                        SignatureAssignmentRhs::TakeFrom { literal, source } => {
                            SignatureValue::TakeFrom {
                                literal,
                                source: Box::new(FunctionValue::Variable { function: source }),
                            }
                        }
                    };
                    Some((lhs, value))
                } else {
                    None
                }
            })
            .collect();

        let mut function_values: HashMap<FunctionId, FunctionValue> = contents
            .statements
            .iter()
            .filter_map(|statement| {
                if let Statement::FunctionAssignment { lhs, ref rhs } = *statement {
                    let value = match *rhs {
                        FunctionAssignmentRhs::Use { signature, literal } => FunctionValue::Use {
                            signature: Box::new(SignatureValue::Variable { signature }),
                            literal,
                        },
                        FunctionAssignmentRhs::Take { signature, literal } => FunctionValue::Take {
                            signature: Box::new(SignatureValue::Variable { signature }),
                            literal,
                        },
                        FunctionAssignmentRhs::Conjure {
                            signature,
                            ref signature_dependencies,
                            ref function_dependencies,
                        } => FunctionValue::Conjure {
                            signature: Box::new(SignatureValue::Variable { signature }),
                            marker: ConjureFunctionMarker::Id(lhs),
                            signature_dependencies: signature_dependencies
                                .iter()
                                .map(|&signature| SignatureValue::Variable { signature })
                                .collect(),
                            function_dependencies: function_dependencies
                                .iter()
                                .map(|&function| FunctionValue::Variable { function })
                                .collect(),
                        },
                        FunctionAssignmentRhs::Define { .. } => {
                            // for now, we only care about the first context, which does not have
                            // sub-contexts
                            unreachable!()
                        }
                        FunctionAssignmentRhs::TakeFrom { literal, source } => {
                            FunctionValue::TakeFrom {
                                literal,
                                source: Box::new(FunctionValue::Variable { function: source }),
                            }
                        }
                        FunctionAssignmentRhs::GiveSignatureTo {
                            signature,
                            literal,
                            source,
                        } => FunctionValue::GiveSignatureTo {
                            signature: Box::new(SignatureValue::Variable { signature }),
                            literal,
                            source: Box::new(FunctionValue::Variable { function: source }),
                        },
                        FunctionAssignmentRhs::GiveFunctionTo {
                            function,
                            literal,
                            source,
                        } => FunctionValue::GiveFunctionTo {
                            function: Box::new(FunctionValue::Variable { function }),
                            literal,
                            source: Box::new(FunctionValue::Variable { function: source }),
                        },
                    };
                    Some((lhs, value))
                } else {
                    None
                }
            })
            .collect();

        dbg!(&signature_values);
        dbg!(&function_values);

        for id in signature_values.keys().copied().collect_vec() {
            let value = signature_values[&id].clone();
            for other in signature_values.values_mut() {
                other.substitute_signature(id, &value);
            }
            for other in function_values.values_mut() {
                other.substitute_signature(id, &value);
            }
        }
        for id in function_values.keys().copied().collect_vec() {
            let value = function_values[&id].clone();
            for other in signature_values.values_mut() {
                other.substitute_function(id, &value);
            }
            for other in function_values.values_mut() {
                other.substitute_function(id, &value);
            }
        }

        /*dbg!(&signature_values);
        dbg!(&function_values);

        for &(_literal, signature) in &given_signatures {
            let value = signature_values.get_mut(&signature).unwrap();
            value.reduce();
        }
        for &(_literal, function) in &given_functions {
            let value = function_values.get_mut(&function).unwrap();
            value.reduce();
        }*/

        dbg!(&signature_values);
        dbg!(&function_values);

        let mut signature_enumeration = HashMap::new();
        let mut function_enumeration = HashMap::new();
        for &(_literal, signature) in &given_signatures {
            let value = signature_values.get_mut(&signature).unwrap();
            value.enumerate_conjurations(&mut signature_enumeration, &mut function_enumeration);
        }
        for &(_literal, function) in &given_functions {
            let value = function_values.get_mut(&function).unwrap();
            value.enumerate_conjurations(&mut signature_enumeration, &mut function_enumeration);
        }

        // Four stages:
        // 0. rewrite assignments as values
        // 1. substitute assignments
        // 2. reduce definitions, take-from's and give-to's
        // 3. enumerate conjurations
    }
}
