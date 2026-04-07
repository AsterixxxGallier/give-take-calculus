use crate::check::{
    FunctionConjuration, FunctionValue, KnownSignatureValue, SignatureConjuration, SignatureValue,
    UnknownFunctionValue, UnknownSignatureValue,
};

impl<'s> SignatureValue<'s> {
    pub(super) fn describes(&self, function: &FunctionValue<'s>) -> bool {
        self.describes_all(&function.signature())
    }

    pub(super) fn describes_all(&self, other: &SignatureValue<'s>) -> bool {
        match self {
            SignatureValue::Known(known_this) => match other {
                SignatureValue::Known(known_other) => {
                    for (&this_id, this_conjuration) in &known_this.conjured_signatures {
                        let name = known_this.resolver.produced_signature_name(this_id);
                        let Some(other_id) = known_other.resolver.produced_signature_id(name)
                        else {
                            return false;
                        };
                        let other_conjuration = &known_other.conjured_signatures[&other_id];
                        let signature_dependencies_subset = other_conjuration
                            .dependencies
                            .signatures
                            .iter()
                            .map(|&id| {
                                known_this.resolver.taken_signature_id(
                                    known_other.resolver.taken_signature_name(id),
                                )
                            })
                            .all(|id| {
                                id.is_some_and(|id| {
                                    this_conjuration.dependencies.signatures.contains(&id)
                                })
                            });
                        let function_dependencies_subset = other_conjuration
                            .dependencies
                            .functions
                            .iter()
                            .all(|(&id, signature)| {
                                known_this
                                    .resolver
                                    .taken_function_id(known_other.resolver.taken_function_name(id))
                                    .is_some_and(|id| {
                                        this_conjuration
                                            .dependencies
                                            .functions
                                            .get(&id)
                                            .is_some_and(|other_signature| {
                                                // TODO or the other way around?
                                                other_signature.describes_all(signature)
                                            })
                                    })
                            });
                        let subset = signature_dependencies_subset && function_dependencies_subset;
                        if !subset {
                            return false;
                        }
                    }

                    for (&this_id, this_conjuration) in &known_this.conjured_functions {
                        let name = known_this.resolver.produced_function_name(this_id);
                        let Some(other_id) = known_other.resolver.produced_function_id(name) else {
                            return false;
                        };
                        let other_conjuration = &known_other.conjured_functions[&other_id];
                        let signature_dependencies_subset = other_conjuration
                            .dependencies
                            .signatures
                            .iter()
                            .map(|&id| {
                                known_this.resolver.taken_signature_id(
                                    known_other.resolver.taken_signature_name(id),
                                )
                            })
                            .all(|id| {
                                id.is_some_and(|id| {
                                    this_conjuration.dependencies.signatures.contains(&id)
                                })
                            });
                        let function_dependencies_subset = other_conjuration
                            .dependencies
                            .functions
                            .iter()
                            .all(|(&id, signature)| {
                                known_this
                                    .resolver
                                    .taken_function_id(known_other.resolver.taken_function_name(id))
                                    .is_some_and(|id| {
                                        this_conjuration
                                            .dependencies
                                            .functions
                                            .get(&id)
                                            .is_some_and(|other_signature| {
                                                // TODO or the other way around?
                                                other_signature.describes_all(signature)
                                            })
                                    })
                            });
                        let subset = signature_dependencies_subset && function_dependencies_subset;
                        if !subset {
                            return false;
                        }
                    }

                    true
                }
                SignatureValue::Unknown(_) => false,
            },
            SignatureValue::Unknown(this_unknown) => match this_unknown {
                &UnknownSignatureValue::Taken(this_id) => match other {
                    SignatureValue::Known(_) => false,
                    SignatureValue::Unknown(other_unknown) => match other_unknown {
                        &UnknownSignatureValue::Taken(other_id) => this_id == other_id,
                        UnknownSignatureValue::Conjured(_) => false,
                    },
                },
                UnknownSignatureValue::Conjured(this_conjured) => match other {
                    SignatureValue::Known(_) => false,
                    SignatureValue::Unknown(other_unknown) => match other_unknown {
                        UnknownSignatureValue::Taken(_) => false,
                        UnknownSignatureValue::Conjured(other_conjured) => {
                            this_conjured == other_conjured
                        }
                    },
                },
            },
        }
    }
}

impl<'s> FunctionValue<'s> {
    pub(crate) fn signature(&self) -> SignatureValue<'s> {
        match self {
            FunctionValue::Known(known) => SignatureValue::Known(KnownSignatureValue {
                conjured_signatures: known
                    .given_signatures
                    .iter()
                    .map(|(&id, lambda)| {
                        (
                            id,
                            SignatureConjuration {
                                dependencies: lambda.dependencies.clone(),
                            },
                        )
                    })
                    .collect(),
                conjured_functions: known
                    .given_functions
                    .iter()
                    .map(|(&id, lambda)| {
                        (
                            id,
                            FunctionConjuration {
                                signature: lambda.function.signature(),
                                dependencies: lambda.dependencies.clone(),
                            },
                        )
                    })
                    .collect(),
                taken_signatures: known.taken_signatures.clone(),
                taken_functions: known.taken_functions.clone(),
                resolver: known.resolver.clone(),
            }),
            FunctionValue::Unknown(unknown) => match unknown {
                UnknownFunctionValue::Taken(_, signature) => signature.clone(),
                UnknownFunctionValue::Conjured(conjured) => {
                    conjured.conjured_function_signature.clone()
                }
            },
        }
    }
}
