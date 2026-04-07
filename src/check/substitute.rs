use crate::check::resolver::Resolver;
use crate::check::{
    ConjuredFunctionValue, ConjuredSignatureValue, FunctionId, FunctionLambda, FunctionValue,
    KnownFunctionValue, KnownSignatureValue, LambdaDependencyValues, SignatureId, SignatureLambda,
    SignatureValue, UnknownFunctionValue, UnknownSignatureValue,
};

// Note: `resolver: &Resolver<'s>` is passed around as an argument here because it's sometimes
// useful for debugging, as without it all we have are opaque IDs. Usually, the argument remains
// unused. We don't need #[allow(unused)] because it's passed around in a recursive manner here.

impl<'s> KnownSignatureValue<'s> {
    pub(super) fn substitute_taken_signature(
        &mut self,
        id: SignatureId,
        value: &SignatureValue<'s>,
        resolver: &Resolver<'s>,
    ) {
        for signature in self.taken_functions.values_mut() {
            signature.substitute_taken_signature(id, value, resolver);
        }

        for conjuration in self.conjured_functions.values_mut() {
            conjuration
                .signature
                .substitute_taken_signature(id, value, resolver);
        }
    }

    pub(super) fn substitute_taken_function(
        &mut self,
        id: FunctionId,
        value: &FunctionValue<'s>,
        resolver: &Resolver<'s>,
    ) {
        for signature in self.taken_functions.values_mut() {
            signature.substitute_taken_function(id, value, resolver);
        }

        for conjuration in self.conjured_functions.values_mut() {
            conjuration
                .signature
                .substitute_taken_function(id, value, resolver);
        }
    }
}

impl<'s> KnownFunctionValue<'s> {
    pub(super) fn substitute_taken_signature(
        &mut self,
        id: SignatureId,
        value: &SignatureValue<'s>,
        resolver: &Resolver<'s>,
    ) {
        for signature in self.taken_functions.values_mut() {
            signature.substitute_taken_signature(id, value, resolver);
        }

        for conjuration in self.given_signatures.values_mut() {
            conjuration
                .signature
                .substitute_taken_signature(id, value, resolver);
        }

        for conjuration in self.given_functions.values_mut() {
            conjuration
                .function
                .substitute_taken_signature(id, value, resolver);
        }
    }

    pub(super) fn substitute_taken_function(
        &mut self,
        id: FunctionId,
        value: &FunctionValue<'s>,
        resolver: &Resolver<'s>,
    ) {
        for signature in self.taken_functions.values_mut() {
            signature.substitute_taken_function(id, value, resolver);
        }

        for conjuration in self.given_signatures.values_mut() {
            conjuration
                .signature
                .substitute_taken_function(id, value, resolver);
        }

        for conjuration in self.given_functions.values_mut() {
            conjuration
                .function
                .substitute_taken_function(id, value, resolver);
        }
    }
}

impl<'s> LambdaDependencyValues<'s> {
    pub(super) fn substitute_taken_signature(
        &mut self,
        id: SignatureId,
        value: &SignatureValue<'s>,
        resolver: &Resolver<'s>,
    ) {
        for signature in self.signatures.values_mut() {
            signature.substitute_taken_signature(id, value, resolver);
        }

        for function in self.functions.values_mut() {
            function.substitute_taken_signature(id, value, resolver);
        }
    }

    pub(super) fn substitute_taken_function(
        &mut self,
        id: FunctionId,
        value: &FunctionValue<'s>,
        resolver: &Resolver<'s>,
    ) {
        for signature in self.signatures.values_mut() {
            signature.substitute_taken_function(id, value, resolver);
        }

        for function in self.functions.values_mut() {
            function.substitute_taken_function(id, value, resolver);
        }
    }
}

impl<'s> ConjuredSignatureValue<'s> {
    pub(super) fn substitute_taken_signature(
        mut self,
        id: SignatureId,
        value: &SignatureValue<'s>,
        resolver: &Resolver<'s>,
    ) -> SignatureValue<'s> {
        let mut unknown_function = FunctionValue::Unknown(self.unknown_function.clone());
        unknown_function.substitute_taken_signature(id, value, resolver);
        self.unknown_function_signature
            .substitute_taken_signature(id, value, resolver);
        self.conjure_dependency_values
            .substitute_taken_signature(id, value, resolver);

        match unknown_function {
            FunctionValue::Known(known) => {
                let name = self
                    .unknown_function_signature
                    .resolver
                    .produced_signature_name(self.conjured_signature);
                let signature_id = known
                    .resolver
                    .produced_signature_id(name)
                    .expect("function should give a signature with this name");
                let SignatureLambda {
                    signature: mut signature_value,
                    dependencies: mut signature_dependencies,
                } = known.given_signatures[&signature_id].clone();

                for (&dependency, value) in &self.conjure_dependency_values.signatures {
                    let name = self
                        .unknown_function_signature
                        .resolver
                        .taken_signature_name(dependency);
                    let dependency = known.resolver.taken_signature_id(name).unwrap();
                    let name = self
                        .unknown_function_signature
                        .resolver
                        .taken_signature_name(dependency);
                    let dependency = known.resolver.taken_signature_id(name).unwrap();
                    let was_present = signature_dependencies.signatures.remove(&dependency);
                    if was_present {
                        signature_value.substitute_taken_signature(dependency, value, resolver);
                    }
                }

                for (&dependency, value) in &self.conjure_dependency_values.functions {
                    let name = self
                        .unknown_function_signature
                        .resolver
                        .taken_function_name(dependency);
                    let dependency = known.resolver.taken_function_id(name).unwrap();
                    let expected_signature = signature_dependencies.functions.remove(&dependency);
                    if expected_signature.is_some() {
                        // TODO check if value matches expected_signature?
                        signature_value.substitute_taken_function(dependency, value, resolver);
                    }
                }

                assert!(signature_dependencies.is_empty());

                signature_value
            }
            FunctionValue::Unknown(unknown) => {
                self.unknown_function = unknown;
                SignatureValue::Unknown(UnknownSignatureValue::Conjured(Box::new(self)))
            }
        }
    }

    pub(super) fn substitute_taken_function(
        mut self,
        id: FunctionId,
        value: &FunctionValue<'s>,
        resolver: &Resolver<'s>,
    ) -> SignatureValue<'s> {
        let mut unknown_function = FunctionValue::Unknown(self.unknown_function.clone());
        unknown_function.substitute_taken_function(id, value, resolver);
        self.unknown_function_signature
            .substitute_taken_function(id, value, resolver);
        self.conjure_dependency_values
            .substitute_taken_function(id, value, resolver);

        match unknown_function {
            FunctionValue::Known(known) => {
                let name = self
                    .unknown_function_signature
                    .resolver
                    .produced_signature_name(self.conjured_signature);
                let signature_id = known
                    .resolver
                    .produced_signature_id(name)
                    .expect("function should give a signature with this name");
                let SignatureLambda {
                    signature: mut signature_value,
                    dependencies: mut signature_dependencies,
                } = known.given_signatures[&signature_id].clone();

                for (&dependency, value) in &self.conjure_dependency_values.signatures {
                    let name = self
                        .unknown_function_signature
                        .resolver
                        .taken_signature_name(dependency);
                    let dependency = known.resolver.taken_signature_id(name).unwrap();
                    let was_present = signature_dependencies.signatures.remove(&dependency);
                    if was_present {
                        signature_value.substitute_taken_signature(dependency, value, resolver);
                    }
                }

                for (&dependency, value) in &self.conjure_dependency_values.functions {
                    let name = self
                        .unknown_function_signature
                        .resolver
                        .taken_function_name(dependency);
                    let dependency = known.resolver.taken_function_id(name).unwrap();
                    let expected_signature = signature_dependencies.functions.remove(&dependency);
                    if expected_signature.is_some() {
                        // TODO check if value matches expected_signature?
                        signature_value.substitute_taken_function(dependency, value, resolver);
                    }
                }

                assert!(signature_dependencies.is_empty());

                signature_value
            }
            FunctionValue::Unknown(unknown) => {
                self.unknown_function = unknown;
                SignatureValue::Unknown(UnknownSignatureValue::Conjured(Box::new(self)))
            }
        }
    }
}

impl<'s> ConjuredFunctionValue<'s> {
    pub(super) fn substitute_taken_signature(
        mut self,
        id: SignatureId,
        value: &SignatureValue<'s>,
        resolver: &Resolver<'s>,
    ) -> FunctionValue<'s> {
        let mut unknown_function = FunctionValue::Unknown(self.unknown_function.clone());
        unknown_function.substitute_taken_signature(id, value, resolver);
        self.conjured_function_signature
            .substitute_taken_signature(id, value, resolver);
        self.unknown_function_signature
            .substitute_taken_signature(id, value, resolver);
        self.conjure_dependency_values
            .substitute_taken_signature(id, value, resolver);

        match unknown_function {
            FunctionValue::Known(known) => {
                let name = self
                    .unknown_function_signature
                    .resolver
                    .produced_function_name(self.conjured_function);
                let function_id = known
                    .resolver
                    .produced_function_id(name)
                    .expect("function should give a function with this name");
                let FunctionLambda {
                    function: mut function_value,
                    dependencies: mut function_dependencies,
                } = known.given_functions[&function_id].clone();

                for (&dependency, value) in &self.conjure_dependency_values.signatures {
                    let name = self
                        .unknown_function_signature
                        .resolver
                        .taken_signature_name(dependency);
                    let dependency = known.resolver.taken_signature_id(name).unwrap();
                    let was_present = function_dependencies.signatures.remove(&dependency);
                    if was_present {
                        function_value.substitute_taken_signature(dependency, value, resolver);
                    }
                }

                for (&dependency, value) in &self.conjure_dependency_values.functions {
                    let name = self
                        .unknown_function_signature
                        .resolver
                        .taken_function_name(dependency);
                    let dependency = known.resolver.taken_function_id(name).unwrap();
                    let expected_signature = function_dependencies.functions.remove(&dependency);
                    if expected_signature.is_some() {
                        function_value.substitute_taken_function(dependency, value, resolver);
                    }
                }

                assert!(function_dependencies.is_empty());

                function_value
            }
            FunctionValue::Unknown(unknown) => {
                self.unknown_function = unknown;
                FunctionValue::Unknown(UnknownFunctionValue::Conjured(Box::new(self)))
            }
        }
    }

    pub(super) fn substitute_taken_function(
        mut self,
        id: FunctionId,
        value: &FunctionValue<'s>,
        resolver: &Resolver<'s>,
    ) -> FunctionValue<'s> {
        let mut unknown_function = FunctionValue::Unknown(self.unknown_function.clone());
        unknown_function.substitute_taken_function(id, value, resolver);
        self.conjured_function_signature
            .substitute_taken_function(id, value, resolver);
        self.unknown_function_signature
            .substitute_taken_function(id, value, resolver);
        self.conjure_dependency_values
            .substitute_taken_function(id, value, resolver);

        match unknown_function {
            FunctionValue::Known(known) => {
                let name = self
                    .unknown_function_signature
                    .resolver
                    .produced_function_name(self.conjured_function);
                let function_id = known
                    .resolver
                    .produced_function_id(name)
                    .expect("function should give a function with this name");
                let FunctionLambda {
                    function: mut function_value,
                    dependencies: mut function_dependencies,
                } = known.given_functions[&function_id].clone();

                for (&dependency, value) in &self.conjure_dependency_values.signatures {
                    let name = self
                        .unknown_function_signature
                        .resolver
                        .taken_signature_name(dependency);
                    let dependency = known.resolver.taken_signature_id(name).unwrap();
                    let was_present = function_dependencies.signatures.remove(&dependency);
                    if was_present {
                        function_value.substitute_taken_signature(dependency, value, resolver);
                    }
                }

                for (&dependency, value) in &self.conjure_dependency_values.functions {
                    let name = self
                        .unknown_function_signature
                        .resolver
                        .taken_function_name(dependency);
                    let dependency = known.resolver.taken_function_id(name).unwrap();
                    let expected_signature = function_dependencies.functions.remove(&dependency);
                    if expected_signature.is_some() {
                        function_value.substitute_taken_function(dependency, value, resolver);
                    }
                }

                assert!(function_dependencies.is_empty());

                function_value
            }
            FunctionValue::Unknown(unknown) => {
                self.unknown_function = unknown;
                FunctionValue::Unknown(UnknownFunctionValue::Conjured(Box::new(self)))
            }
        }
    }
}

impl<'s> SignatureValue<'s> {
    pub(super) fn substitute_taken_signature(
        &mut self,
        id: SignatureId,
        value: &SignatureValue<'s>,
        resolver: &Resolver<'s>,
    ) {
        match self {
            SignatureValue::Known(known) => {
                known.substitute_taken_signature(id, value, resolver);
            }
            SignatureValue::Unknown(UnknownSignatureValue::Taken(signature)) => {
                if *signature == id {
                    *self = value.clone();
                }
            }
            SignatureValue::Unknown(UnknownSignatureValue::Conjured(conjured)) => {
                *self = conjured
                    .as_ref()
                    .clone()
                    .substitute_taken_signature(id, value, resolver);
            }
        }
    }

    pub(super) fn substitute_taken_function(
        &mut self,
        id: FunctionId,
        value: &FunctionValue<'s>,
        resolver: &Resolver<'s>,
    ) {
        match self {
            SignatureValue::Known(known) => {
                known.substitute_taken_function(id, value, resolver);
            }
            SignatureValue::Unknown(UnknownSignatureValue::Taken(signature)) => {
                _ = signature;
            }
            SignatureValue::Unknown(UnknownSignatureValue::Conjured(conjured)) => {
                *self = conjured
                    .as_ref()
                    .clone()
                    .substitute_taken_function(id, value, resolver);
            }
        }
    }
}

impl<'s> FunctionValue<'s> {
    pub(super) fn substitute_taken_signature(
        &mut self,
        id: SignatureId,
        value: &SignatureValue<'s>,
        resolver: &Resolver<'s>,
    ) {
        match self {
            FunctionValue::Known(known) => {
                known.substitute_taken_signature(id, value, resolver);
            }
            FunctionValue::Unknown(UnknownFunctionValue::Taken(function, signature)) => {
                _ = function;
                signature.substitute_taken_signature(id, value, resolver);
            }
            FunctionValue::Unknown(UnknownFunctionValue::Conjured(conjured)) => {
                *self = conjured
                    .as_ref()
                    .clone()
                    .substitute_taken_signature(id, value, resolver);
            }
        }
    }

    pub(super) fn substitute_taken_function(
        &mut self,
        id: FunctionId,
        value: &FunctionValue<'s>,
        resolver: &Resolver<'s>,
    ) {
        match self {
            FunctionValue::Known(known) => {
                known.substitute_taken_function(id, value, resolver);
            }
            FunctionValue::Unknown(UnknownFunctionValue::Taken(function, signature)) => {
                if *function == id {
                    *self = value.clone();
                } else {
                    signature.substitute_taken_function(id, value, resolver);
                }
            }
            FunctionValue::Unknown(UnknownFunctionValue::Conjured(conjured)) => {
                *self = conjured
                    .as_ref()
                    .clone()
                    .substitute_taken_function(id, value, resolver);
            }
        }
    }
}
