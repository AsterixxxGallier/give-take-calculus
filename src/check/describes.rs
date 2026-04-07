use crate::check::{
    FunctionConjuration, FunctionValue, KnownSignatureValue, SignatureConjuration, SignatureValue,
    UnknownFunctionValue, UnknownSignatureValue,
};

impl<'s> SignatureValue<'s> {
    pub(super) fn describes(&self, function: &FunctionValue<'s>) -> bool {
        self.can_be_used_in_place_of(&function.signature())
    }

    /// Returns `true` iff all functions of this signature can be used wherever a function of
    /// signature `other` is required.
    pub(super) fn can_be_used_in_place_of(&self, other: &SignatureValue<'s>) -> bool {
        match self {
            SignatureValue::Known(known_this) => match other {
                SignatureValue::Known(known_other) => {
                    // First condition:
                    // If a signature can be given to OTHER, THIS should also take a signature with
                    // the same name.

                    for &other_id in &known_other.taken_signatures {
                        let name = known_other.resolver.taken_signature_name(other_id);
                        let this_id = known_this.resolver.taken_signature_id(name);
                        if this_id.is_none() {
                            // THIS does not take a signature with the same name.
                            return false;
                        }
                    }

                    // Second condition:
                    // If a function can be given to OTHER, THIS should also take a function with
                    // the same name. Additionally, a function that may be given to OTHER must also
                    // be accepted by THIS.

                    for (&other_id, other_signature) in &known_other.taken_functions {
                        let name = known_other.resolver.taken_function_name(other_id);
                        let this_id = known_this.resolver.taken_function_id(name);
                        if let Some(this_id) = this_id {
                            let this_signature = &known_this.taken_functions[&this_id];
                            if !other_signature.can_be_used_in_place_of(this_signature) {
                                // There are functions that would be accepted by OTHER, but are not
                                // accepted by THIS.
                                return false;
                            }
                        } else {
                            // THIS does not take a function with the same name.
                            return false;
                        }
                    }

                    // Third condition:
                    // If a signature can be taken from OTHER, it must also be produced by THIS,
                    // using at most the dependencies that OTHER permits in the calculation of the
                    // produced signature.

                    for (&other_id, other_conjuration) in &known_other.conjured_signatures {
                        let name = known_other.resolver.produced_signature_name(other_id);
                        let this_id = known_this.resolver.produced_signature_id(name);
                        if let Some(this_id) = this_id {
                            let this_conjuration = &known_this.conjured_signatures[&this_id];

                            for &this_dependency_id in &this_conjuration.dependencies.signatures {
                                let dependency_name =
                                    known_this.resolver.taken_signature_name(this_dependency_id);
                                let other_dependency_id =
                                    known_other.resolver.taken_signature_id(dependency_name);
                                if let Some(other_dependency_id) = other_dependency_id {
                                    let is_dependency_of_other = other_conjuration
                                        .dependencies
                                        .signatures
                                        .contains(&other_dependency_id);
                                    if !is_dependency_of_other {
                                        // The dependency used by THIS is not used by OTHER in the
                                        // calculation of the produced signature.
                                        return false;
                                    }
                                } else {
                                    // OTHER does not even take a signature with the same name, so
                                    // it cannot be a dependency of this conjuration.
                                    return false;
                                }
                            }

                            // TODO explain why the signatures can be ignored
                            for &this_dependency_id in
                                this_conjuration.dependencies.functions.keys()
                            {
                                let dependency_name =
                                    known_this.resolver.taken_function_name(this_dependency_id);
                                let other_dependency_id =
                                    known_other.resolver.taken_function_id(dependency_name);
                                if let Some(other_dependency_id) = other_dependency_id {
                                    let is_dependency_of_other = other_conjuration
                                        .dependencies
                                        .functions
                                        .contains_key(&other_dependency_id);
                                    if !is_dependency_of_other {
                                        // The dependency used by THIS is not used by OTHER in the
                                        // calculation of the produced signature.
                                        return false;
                                    }
                                } else {
                                    // OTHER does not even take a function with the same name, so
                                    // it cannot be a dependency of this conjuration.
                                    return false;
                                }
                            }
                        } else {
                            // THIS does not produce a signature with the same name.
                            return false;
                        }
                    }

                    // Fourth condition:
                    // If a function can be taken from OTHER, it must also be produced by THIS,
                    // using at most the dependencies that OTHER permits in the calculation of the
                    // produced signature. Additionally, the function produced by THIS must match
                    // the signature expected in the conjuration in OTHER.

                    for (&other_id, other_conjuration) in &known_other.conjured_functions {
                        let name = known_other.resolver.produced_function_name(other_id);
                        let this_id = known_this.resolver.produced_function_id(name);
                        if let Some(this_id) = this_id {
                            let this_conjuration = &known_this.conjured_functions[&this_id];

                            for &this_dependency_id in &this_conjuration.dependencies.signatures {
                                let dependency_name =
                                    known_this.resolver.taken_signature_name(this_dependency_id);
                                let other_dependency_id =
                                    known_other.resolver.taken_signature_id(dependency_name);
                                if let Some(other_dependency_id) = other_dependency_id {
                                    let is_dependency_of_other = other_conjuration
                                        .dependencies
                                        .signatures
                                        .contains(&other_dependency_id);
                                    if !is_dependency_of_other {
                                        // The dependency used by THIS is not used by OTHER in the
                                        // calculation of the produced function.
                                        return false;
                                    }
                                } else {
                                    // OTHER does not even take a signature with the same name, so
                                    // it cannot be a dependency of this conjuration.
                                    return false;
                                }
                            }

                            for &this_dependency_id in
                                this_conjuration.dependencies.functions.keys()
                            {
                                let dependency_name =
                                    known_this.resolver.taken_function_name(this_dependency_id);
                                let other_dependency_id =
                                    known_other.resolver.taken_function_id(dependency_name);
                                if let Some(other_dependency_id) = other_dependency_id {
                                    let is_dependency_of_other = other_conjuration
                                        .dependencies
                                        .functions
                                        .contains_key(&other_dependency_id);
                                    if !is_dependency_of_other {
                                        // The dependency used by THIS is not used by OTHER in the
                                        // calculation of the produced function.
                                        return false;
                                    }
                                } else {
                                    // OTHER does not even take a function with the same name, so
                                    // it cannot be a dependency of this conjuration.
                                    return false;
                                }
                            }

                            let this_signature = &this_conjuration.signature;
                            let other_signature = &other_conjuration.signature;

                            if !this_signature.can_be_used_in_place_of(other_signature) {
                                // Some of the functions produced by THIS cannot be used in the way
                                // that is guaranteed by OTHER.
                                return false;
                            }
                        } else {
                            // THIS does not produce a function with the same name.
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
