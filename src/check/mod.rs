use crate::parse::*;
use ordermap::{OrderMap, OrderSet};
use std::collections::HashMap;
use std::mem;

mod error;
mod id;

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

#[derive(Clone, Default, Eq, PartialEq)]
struct KnownSignatureValue<'s> {
    conjured_signatures: OrderMap<SignatureId, SignatureConjuration<'s>>,
    conjured_functions: OrderMap<FunctionId, FunctionConjuration<'s>>,
    conjured_signature_ids: HashMap<&'s str, SignatureId>,
    conjured_function_ids: HashMap<&'s str, FunctionId>,
    conjured_signature_names: HashMap<SignatureId, &'s str>,
    conjured_function_names: HashMap<FunctionId, &'s str>,
    taken_signature_ids: HashMap<&'s str, SignatureId>,
    taken_function_ids: HashMap<&'s str, FunctionId>,
}

#[derive(Clone, Default, Eq, PartialEq)]
struct KnownFunctionValue<'s> {
    given_signatures: OrderMap<SignatureId, SignatureLambda<'s>>,
    given_functions: OrderMap<FunctionId, FunctionLambda<'s>>,
    given_signature_ids: HashMap<&'s str, SignatureId>,
    given_function_ids: HashMap<&'s str, FunctionId>,
    given_signature_names: HashMap<SignatureId, &'s str>,
    given_function_names: HashMap<FunctionId, &'s str>,
    taken_signature_ids: HashMap<&'s str, SignatureId>,
    taken_function_ids: HashMap<&'s str, FunctionId>,
}

#[derive(Clone, Eq, PartialEq)]
struct SignatureConjuration<'s> {
    dependencies: LambdaDependencies<'s>,
}

#[derive(Clone, Eq, PartialEq)]
struct FunctionConjuration<'s> {
    signature: SignatureValue<'s>,
    // must also contain all dependencies of signature
    dependencies: LambdaDependencies<'s>,
}

#[derive(Clone, Eq, PartialEq)]
struct SignatureLambda<'s> {
    signature: SignatureValue<'s>,
    dependencies: LambdaDependencies<'s>,
}

#[derive(Clone, Eq, PartialEq)]
struct FunctionLambda<'s> {
    function: FunctionValue<'s>,
    dependencies: LambdaDependencies<'s>,
}

#[derive(Clone, Eq, PartialEq, Default)]
struct LambdaDependencies<'s> {
    signatures: OrderSet<SignatureId>,
    functions: OrderMap<FunctionId, SignatureValue<'s>>,
}

#[derive(Clone, Eq, PartialEq)]
struct LambdaDependencyValues<'s> {
    signatures: OrderMap<SignatureId, SignatureValue<'s>>,
    functions: OrderMap<FunctionId, FunctionValue<'s>>,
}

#[derive(Clone, Eq, PartialEq)]
struct ConjuredSignatureValue<'s> {
    unknown_function: UnknownFunctionValue<'s>,
    unknown_function_signature: KnownSignatureValue<'s>,
    conjured_signature: SignatureId,
    conjure_dependency_values: LambdaDependencyValues<'s>,
}

#[derive(Clone, Eq, PartialEq)]
struct ConjuredFunctionValue<'s> {
    unknown_function: UnknownFunctionValue<'s>,
    unknown_function_signature: KnownSignatureValue<'s>,
    conjured_function: FunctionId,
    conjured_function_signature: SignatureValue<'s>,
    conjure_dependency_values: LambdaDependencyValues<'s>,
}

#[derive(Clone, Eq, PartialEq)]
enum UnknownSignatureValue<'s> {
    Taken(SignatureId),
    Conjured(Box<ConjuredSignatureValue<'s>>),
}

#[derive(Clone, Eq, PartialEq)]
enum UnknownFunctionValue<'s> {
    Taken(FunctionId, SignatureValue<'s>),
    Conjured(Box<ConjuredFunctionValue<'s>>),
}

#[derive(Clone, Eq, PartialEq)]
enum SignatureValue<'s> {
    Known(KnownSignatureValue<'s>),
    Unknown(UnknownSignatureValue<'s>),
}

#[derive(Clone, Eq, PartialEq)]
enum FunctionValue<'s> {
    Known(KnownFunctionValue<'s>),
    Unknown(UnknownFunctionValue<'s>),
}

impl<'s> LambdaDependencies<'s> {
    pub(crate) fn is_empty(&self) -> bool {
        self.signatures.is_empty() && self.functions.is_empty()
    }

    pub(crate) fn is_subset_of(&self, other: &LambdaDependencies) -> bool {
        self.signatures.is_subset(&other.signatures)
            && self.functions.iter().all(|(&function_id, signature)| {
            if let Some(other_signature) = other.functions.get(&function_id) {
                // TODO check if signature is a sub-signature of other_signature
                _ = signature;
                _ = other_signature;
                true
            } else {
                false
            }
        })
    }

    pub(crate) fn as_values(&self) -> LambdaDependencyValues<'s> {
        LambdaDependencyValues {
            signatures: self
                .signatures
                .iter()
                .map(|&dependency| {
                    (
                        dependency,
                        SignatureValue::Unknown(UnknownSignatureValue::Taken(dependency)),
                    )
                })
                .collect(),
            functions: self
                .functions
                .iter()
                .map(|(&dependency, signature)| {
                    (
                        dependency,
                        FunctionValue::Unknown(UnknownFunctionValue::Taken(
                            dependency,
                            signature.clone(),
                        )),
                    )
                })
                .collect(),
        }
    }
}

impl<'s> SignatureValue<'s> {
    pub(crate) fn describes(&self, function: &FunctionValue<'s>) -> bool {
        match self {
            SignatureValue::Known(known_signature) => match function {
                FunctionValue::Known(known_function) => {
                    for (&conjured_id, conjuration) in &known_signature.conjured_signatures {
                        let conjured_name = known_signature.conjured_signature_names[&conjured_id];
                        let Some(&given_id) = known_function.given_signature_ids.get(conjured_name)
                        else {
                            return false;
                        };
                        let given_lambda = &known_function.given_signatures[&given_id];
                        let subset = given_lambda
                            .dependencies
                            .is_subset_of(&conjuration.dependencies);
                        if !subset {
                            return false;
                        }
                    }

                    for (&conjured_id, conjuration) in &known_signature.conjured_functions {
                        let conjured_name = known_signature.conjured_function_names[&conjured_id];
                        let Some(&given_id) = known_function.given_function_ids.get(conjured_name)
                        else {
                            return false;
                        };
                        let given_lambda = &known_function.given_functions[&given_id];
                        let subset = given_lambda
                            .dependencies
                            .is_subset_of(&conjuration.dependencies);
                        if !subset {
                            return false;
                        }
                    }

                    true
                }
                // unknown function with known signature would have been converted to known function
                // immediately
                FunctionValue::Unknown(_) => false,
            },
            SignatureValue::Unknown(unknown) => match *unknown {
                UnknownSignatureValue::Taken(signature_id) => match function {
                    FunctionValue::Known(_) => false,
                    FunctionValue::Unknown(unknown) => match *unknown {
                        UnknownFunctionValue::Taken(function, ref signature) => {
                            _ = function;
                            matches!(
                                signature,
                                &SignatureValue::Unknown(UnknownSignatureValue::Taken(other_id))
                                if other_id == signature_id
                            )
                        }
                        UnknownFunctionValue::Conjured(_) => false,
                    },
                },
                UnknownSignatureValue::Conjured(ref conjured) => match function {
                    FunctionValue::Known(_) => false,
                    FunctionValue::Unknown(unknown) => match *unknown {
                        UnknownFunctionValue::Taken(function, ref signature) => {
                            _ = function;
                            matches!(
                                signature,
                                SignatureValue::Unknown(UnknownSignatureValue::Conjured(other_conjured))
                                if conjured == other_conjured
                            )
                        }
                        UnknownFunctionValue::Conjured(_) => false,
                    },
                },
            },
        }
    }
}

impl<'s> KnownSignatureValue<'s> {
    pub(crate) fn substitute_taken_signature(
        &mut self,
        id: SignatureId,
        value: &SignatureValue<'s>,
    ) {
        for conjuration in self.conjured_functions.values_mut() {
            conjuration.signature.substitute_taken_signature(id, value);
        }
    }

    pub(crate) fn substitute_taken_function(&mut self, id: FunctionId, value: &FunctionValue<'s>) {
        for conjuration in self.conjured_functions.values_mut() {
            conjuration.signature.substitute_taken_function(id, value);
        }
    }
}

impl<'s> KnownFunctionValue<'s> {
    pub(crate) fn substitute_taken_signature(
        &mut self,
        id: SignatureId,
        value: &SignatureValue<'s>,
    ) {
        for conjuration in self.given_signatures.values_mut() {
            conjuration.signature.substitute_taken_signature(id, value);
        }

        for conjuration in self.given_functions.values_mut() {
            conjuration.function.substitute_taken_signature(id, value);
        }
    }

    pub(crate) fn substitute_taken_function(&mut self, id: FunctionId, value: &FunctionValue<'s>) {
        for conjuration in self.given_signatures.values_mut() {
            conjuration.signature.substitute_taken_function(id, value);
        }

        for conjuration in self.given_functions.values_mut() {
            conjuration.function.substitute_taken_function(id, value);
        }
    }
}

impl<'s> LambdaDependencyValues<'s> {
    pub(crate) fn substitute_taken_signature(
        &mut self,
        id: SignatureId,
        value: &SignatureValue<'s>,
    ) {
        for signature in self.signatures.values_mut() {
            signature.substitute_taken_signature(id, value);
        }

        for function in self.functions.values_mut() {
            function.substitute_taken_signature(id, value);
        }
    }

    pub(crate) fn substitute_taken_function(&mut self, id: FunctionId, value: &FunctionValue<'s>) {
        for signature in self.signatures.values_mut() {
            signature.substitute_taken_function(id, value);
        }

        for function in self.functions.values_mut() {
            function.substitute_taken_function(id, value);
        }
    }
}

/*
#[derive(Clone, Eq, PartialEq)]
struct ConjuredSignatureValue<'s> {
    unknown_function: UnknownFunctionValue<'s>,
    unknown_function_signature: KnownSignatureValue<'s>,
    conjured_signature: SignatureId,
    conjure_dependency_values: LambdaDependencyValues<'s>,
}

#[derive(Clone, Eq, PartialEq)]
struct ConjuredFunctionValue<'s> {
    unknown_function: UnknownFunctionValue<'s>,
    unknown_function_signature: KnownSignatureValue<'s>,
    conjured_function: FunctionId,
    conjured_function_signature: SignatureValue<'s>,
    conjure_dependency_values: LambdaDependencyValues<'s>,
}
 */

impl<'s> ConjuredSignatureValue<'s> {
    pub(crate) fn substitute_taken_signature(
        mut self,
        id: SignatureId,
        value: &SignatureValue<'s>,
    ) -> SignatureValue<'s> {
        let mut unknown_function = FunctionValue::Unknown(self.unknown_function.clone());
        unknown_function.substitute_taken_signature(id, value);
        self.unknown_function_signature
            .substitute_taken_signature(id, value);
        self.conjure_dependency_values
            .substitute_taken_signature(id, value);

        match unknown_function {
            FunctionValue::Known(known) => {
                let name = self.unknown_function_signature.conjured_signature_names
                    [&self.conjured_signature];
                let signature_id = *known
                    .given_signature_ids
                    .get(name)
                    .expect("function should give a signature with this name");
                let SignatureLambda {
                    signature: mut signature_value,
                    dependencies: mut signature_dependencies,
                } = known.given_signatures[&signature_id].clone();

                for (&dependency, value) in &self.conjure_dependency_values.signatures {
                    signature_value.substitute_taken_signature(dependency, value);
                    let was_present = signature_dependencies.signatures.remove(&dependency);
                    assert!(was_present);
                }

                for (&dependency, value) in &self.conjure_dependency_values.functions {
                    signature_value.substitute_taken_function(dependency, value);
                    // TODO check if value matches expected_signature?
                    let expected_signature = signature_dependencies.functions.remove(&dependency);
                    assert!(expected_signature.is_some());
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

    pub(crate) fn substitute_taken_function(
        mut self,
        id: FunctionId,
        value: &FunctionValue<'s>,
    ) -> SignatureValue<'s> {
        let mut unknown_function = FunctionValue::Unknown(self.unknown_function.clone());
        unknown_function.substitute_taken_function(id, value);
        self.unknown_function_signature
            .substitute_taken_function(id, value);
        self.conjure_dependency_values
            .substitute_taken_function(id, value);

        match unknown_function {
            FunctionValue::Known(known) => {
                let name = self.unknown_function_signature.conjured_signature_names
                    [&self.conjured_signature];
                let signature_id = *known
                    .given_signature_ids
                    .get(name)
                    .expect("function should give a signature with this name");
                let SignatureLambda {
                    signature: mut signature_value,
                    dependencies: mut signature_dependencies,
                } = known.given_signatures[&signature_id].clone();

                for (&dependency, value) in &self.conjure_dependency_values.signatures {
                    signature_value.substitute_taken_signature(dependency, value);
                    let was_present = signature_dependencies.signatures.remove(&dependency);
                    assert!(was_present);
                }

                for (&dependency, value) in &self.conjure_dependency_values.functions {
                    signature_value.substitute_taken_function(dependency, value);
                    // TODO check if value matches expected_signature?
                    let expected_signature = signature_dependencies.functions.remove(&dependency);
                    assert!(expected_signature.is_some());
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
    pub(crate) fn substitute_taken_signature(
        mut self,
        id: SignatureId,
        value: &SignatureValue<'s>,
    ) -> FunctionValue<'s> {
        let mut unknown_function = FunctionValue::Unknown(self.unknown_function.clone());
        unknown_function.substitute_taken_signature(id, value);
        self.unknown_function_signature
            .substitute_taken_signature(id, value);
        self.conjure_dependency_values
            .substitute_taken_signature(id, value);

        match unknown_function {
            FunctionValue::Known(known) => {
                let name = self.unknown_function_signature.conjured_function_names
                    [&self.conjured_function];
                let function_id = *known
                    .given_function_ids
                    .get(name)
                    .expect("function should give a function with this name");
                let FunctionLambda {
                    function: mut function_value,
                    dependencies: mut function_dependencies,
                } = known.given_functions[&function_id].clone();

                for (&dependency, value) in &self.conjure_dependency_values.signatures {
                    function_value.substitute_taken_signature(dependency, value);
                    let was_present = function_dependencies.signatures.remove(&dependency);
                    assert!(was_present);
                }

                for (&dependency, value) in &self.conjure_dependency_values.functions {
                    function_value.substitute_taken_function(dependency, value);
                    let expected_signature = function_dependencies.functions.remove(&dependency);
                    assert!(expected_signature.is_some());
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

    pub(crate) fn substitute_taken_function(
        mut self,
        id: FunctionId,
        value: &FunctionValue<'s>,
    ) -> FunctionValue<'s> {
        let mut unknown_function = FunctionValue::Unknown(self.unknown_function.clone());
        unknown_function.substitute_taken_function(id, value);
        self.unknown_function_signature
            .substitute_taken_function(id, value);
        self.conjure_dependency_values
            .substitute_taken_function(id, value);

        match unknown_function {
            FunctionValue::Known(known) => {
                let name = self.unknown_function_signature.conjured_function_names
                    [&self.conjured_function];
                let function_id = *known
                    .given_function_ids
                    .get(name)
                    .expect("function should give a function with this name");
                let FunctionLambda {
                    function: mut function_value,
                    dependencies: mut function_dependencies,
                } = known.given_functions[&function_id].clone();

                for (&dependency, value) in &self.conjure_dependency_values.signatures {
                    function_value.substitute_taken_signature(dependency, value);
                    let was_present = function_dependencies.signatures.remove(&dependency);
                    assert!(was_present);
                }

                for (&dependency, value) in &self.conjure_dependency_values.functions {
                    function_value.substitute_taken_function(dependency, value);
                    let expected_signature = function_dependencies.functions.remove(&dependency);
                    assert!(expected_signature.is_some());
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
    pub(crate) fn substitute_taken_signature(
        &mut self,
        id: SignatureId,
        value: &SignatureValue<'s>,
    ) {
        match self {
            SignatureValue::Known(known) => {
                known.substitute_taken_signature(id, value);
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
                    .substitute_taken_signature(id, value);
            }
        }
    }

    pub(crate) fn substitute_taken_function(&mut self, id: FunctionId, value: &FunctionValue<'s>) {
        match self {
            SignatureValue::Known(known) => {
                known.substitute_taken_function(id, value);
            }
            SignatureValue::Unknown(UnknownSignatureValue::Taken(signature)) => {
                _ = signature;
            }
            SignatureValue::Unknown(UnknownSignatureValue::Conjured(conjured)) => {
                *self = conjured
                    .as_ref()
                    .clone()
                    .substitute_taken_function(id, value);
            }
        }
    }
}

impl<'s> FunctionValue<'s> {
    pub(crate) fn substitute_taken_signature(
        &mut self,
        id: SignatureId,
        value: &SignatureValue<'s>,
    ) {
        match self {
            FunctionValue::Known(known) => {
                known.substitute_taken_signature(id, value);
            }
            FunctionValue::Unknown(UnknownFunctionValue::Taken(function, signature)) => {
                _ = function;
                signature.substitute_taken_signature(id, value);
            }
            FunctionValue::Unknown(UnknownFunctionValue::Conjured(conjured)) => {
                *self = conjured
                    .as_ref()
                    .clone()
                    .substitute_taken_signature(id, value);
            }
        }
    }

    pub(crate) fn substitute_taken_function(&mut self, id: FunctionId, value: &FunctionValue<'s>) {
        match self {
            FunctionValue::Known(known) => {
                known.substitute_taken_function(id, value);
            }
            FunctionValue::Unknown(UnknownFunctionValue::Taken(function, signature)) => {
                if *function == id {
                    *self = value.clone();
                } else {
                    signature.substitute_taken_function(id, value);
                }
            }
            FunctionValue::Unknown(UnknownFunctionValue::Conjured(conjured)) => {
                *self = conjured
                    .as_ref()
                    .clone()
                    .substitute_taken_function(id, value);
            }
        }
    }
}

impl<'s> LambdaDependencies<'s> {
    fn add(&mut self, other: &Self) {
        for &signature in &other.signatures {
            self.signatures.insert(signature);
        }

        for (&function, signature) in &other.functions {
            if self.functions.contains_key(&function) {
                // can this ever happen?
                // if so: should we allow one function to be constrained by multiple signatures?
                assert!(&self.functions[&function] == signature);
            } else {
                self.functions.insert(function, signature.clone());
            }
        }
    }
}

// I don't think this ever made sense
/*impl<'s> KnownSignatureValue<'s> {
    fn collect_dependencies(&self, dependencies: &mut LambdaDependencies<'s>) {
        for conjuration in self.conjured_signatures.values() {
            dependencies.add(&conjuration.dependencies);
        }

        for conjuration in self.conjured_functions.values() {
            dependencies.add(&conjuration.dependencies);
        }
    }
}

impl<'s> KnownFunctionValue<'s> {
    fn collect_dependencies(&self, dependencies: &mut LambdaDependencies<'s>) {
        for lambda in self.given_signatures.values() {
            dependencies.add(&lambda.dependencies);
        }

        for lambda in self.given_functions.values() {
            dependencies.add(&lambda.dependencies);
        }
    }
}*/

pub(crate) fn check_function_context(context: FunctionContext) -> Result<(), CheckError> {
    let mut state = EvaluationState::default();
    state.evaluate_function_context(context)?;
    Ok(())
}

type CheckResult<'s, T> = Result<T, CheckError<'s>>;

#[derive(Default)]
struct EvaluationState<'s> {
    parent: Option<Box<EvaluationState<'s>>>,
    signature_lambdas: HashMap<SignatureId, SignatureLambda<'s>>,
    function_lambdas: HashMap<FunctionId, FunctionLambda<'s>>,
    signature_ids: HashMap<&'s str, SignatureId>,
    function_ids: HashMap<&'s str, FunctionId>,
    dependencies: LambdaDependencies<'s>,
}

impl<'s> EvaluationState<'s> {
    // yes i know how fucked up this looks but i've been fighting the borrow checker for half an
    // hour now so please forgive me
    fn do_as_child<R>(
        &mut self,
        as_child: impl FnOnce(&mut Self) -> R,
    ) -> (R, LambdaDependencies<'s>) {
        let parent = mem::take(self);
        let mut child = Self {
            parent: Some(Box::new(parent)),
            ..Self::default()
        };
        let result = as_child(&mut child);
        let dependencies = child.dependencies;
        *self = *child.parent.take().unwrap();
        (result, dependencies)
    }

    fn resolve_signature(&self, signature: Signature<'s>) -> CheckResult<'s, SignatureId> {
        let mut this = Some(self);
        while let Some(current) = this {
            if let Some(&id) = current.signature_ids.get(signature.as_str()) {
                return Ok(id);
            }
            this = current.parent.as_deref();
        }
        Err(CheckError::CannotResolveSignature { signature })
    }

    fn resolve_function(&self, function: Function<'s>) -> CheckResult<'s, FunctionId> {
        let mut this = Some(self);
        while let Some(current) = this {
            if let Some(&id) = current.function_ids.get(function.as_str()) {
                return Ok(id);
            }
            this = current.parent.as_deref();
        }
        Err(CheckError::CannotResolveFunction { function })
    }

    fn signature_lambda(&mut self, signature: SignatureId) -> SignatureLambda<'s> {
        if let Some(lambda) = self.signature_lambdas.get(&signature) {
            return lambda.clone();
        }

        let mut this = self.parent.as_mut();
        while let Some(current) = this {
            if let Some(lambda) = current.signature_lambdas.get(&signature) {
                current.dependencies.add(&lambda.dependencies);
                return SignatureLambda {
                    signature: lambda.signature.clone(),
                    dependencies: Default::default(),
                };
            }
            this = current.parent.as_mut();
        }

        panic!("could not get lambda for signature");
    }

    fn function_lambda(&mut self, function: FunctionId) -> FunctionLambda<'s> {
        if let Some(lambda) = self.function_lambdas.get(&function) {
            return lambda.clone();
        }

        let mut this = self.parent.as_mut();
        while let Some(current) = this {
            if let Some(lambda) = current.function_lambdas.get(&function) {
                current.dependencies.add(&lambda.dependencies);
                return FunctionLambda {
                    function: lambda.function.clone(),
                    dependencies: Default::default(),
                };
            }
            this = current.parent.as_mut();
        }

        panic!("could not get lambda for function");
    }

    fn process_signature_assignment(
        &mut self,
        assignment: SignatureAssignment<'s>,
        mut register_taken_signature: impl FnMut(Signature<'s>, SignatureId),
    ) -> CheckResult<'s, ()> {
        let SignatureAssignment { location, lhs, rhs } = assignment;

        let lhs_id = SignatureId::generate();

        let lambda = match rhs {
            SignatureAssignmentRhs::Take(TakeSignature { phantom: _ }) => {
                register_taken_signature(lhs, lhs_id);
                SignatureLambda {
                    signature: SignatureValue::Unknown(UnknownSignatureValue::Taken(lhs_id)),
                    dependencies: LambdaDependencies {
                        signatures: OrderSet::from([lhs_id]),
                        functions: OrderMap::new(),
                    },
                }
            },
            SignatureAssignmentRhs::Define(DefineSignature { context }) => {
                let (signature, dependencies) =
                    self.do_as_child(|child| child.evaluate_signature_context(context));
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
                    FunctionValue::Known(KnownFunctionValue {
                                             given_signatures,
                                             given_functions: _,
                                             given_signature_ids,
                                             given_function_ids: _,
                                             given_signature_names: _,
                                             given_function_names: _,
                                             taken_signature_ids: _,
                                             taken_function_ids: _,
                                         }) => {
                        let foreign_id =
                            if let Some(&id) = given_signature_ids.get(foreign.as_str()) {
                                id
                            } else {
                                return Err(CheckError::CannotResolveGivenSignature {
                                    statement: location,
                                    source,
                                    foreign,
                                });
                            };

                        let foreign_lambda = given_signatures
                            .get(&foreign_id)
                            .expect("could not find given signature by id");

                        if !foreign_lambda.dependencies.is_empty() {
                            return Err(CheckError::GiveSignatureDependenciesNotProvided {
                                statement: location,
                                source,
                                foreign,
                            });
                        }

                        SignatureLambda {
                            signature: foreign_lambda.signature.clone(),
                            dependencies: source_dependencies,
                        }
                    }
                    FunctionValue::Unknown(_) => {
                        return Err(CheckError::CannotTakeSignatureFromUnknownFunction {
                            statement: location,
                            source,
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
                    signature,
                    dependencies: signature_dependencies,
                } = self.signature_lambda(signature_id);

                let source_id = self.resolve_signature(source)?;
                let SignatureLambda {
                    signature: source_value,
                    dependencies: source_dependencies,
                } = self.signature_lambda(source_id);

                match source_value {
                    SignatureValue::Known(mut source_value) => {
                        let foreign_id = if let Some(&id) =
                            source_value.taken_signature_ids.get(foreign.as_str())
                        {
                            id
                        } else {
                            return Err(CheckError::CannotResolveTakenSignatureOfSignature {
                                statement: location,
                                source,
                                foreign,
                            });
                        };

                        for conjuration in source_value.conjured_signatures.values_mut() {
                            // foreign won't be a dependency of _all_ conjurations,
                            // so it's okay if it's not in the set here
                            _ = conjuration.dependencies.signatures.remove(&foreign_id);
                        }

                        for conjuration in source_value.conjured_functions.values_mut() {
                            _ = conjuration.dependencies.signatures.remove(&foreign_id);
                            conjuration
                                .signature
                                .substitute_taken_signature(foreign_id, &signature);
                        }

                        let mut dependencies = signature_dependencies;
                        dependencies.add(&source_dependencies);

                        SignatureLambda {
                            signature: SignatureValue::Known(source_value),
                            dependencies,
                        }
                    }
                    SignatureValue::Unknown(_) => {
                        return Err(CheckError::CannotGiveSignatureToUnknownSignature {
                            statement: location,
                            source,
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
                        let foreign_id = if let Some(&id) =
                            source_value.taken_function_ids.get(foreign.as_str())
                        {
                            id
                        } else {
                            return Err(CheckError::CannotResolveTakenFunctionOfSignature {
                                statement: location,
                                source,
                                foreign,
                            });
                        };

                        for conjuration in source_value.conjured_signatures.values_mut() {
                            // foreign won't be a dependency of _all_ conjurations,
                            // so it's okay if it's not in the set here
                            if let Some(expected_signature) =
                                conjuration.dependencies.functions.get(&foreign_id)
                            {
                                if !expected_signature.describes(&function_value) {
                                    return Err(CheckError::FunctionGivenToSignatureDoesNotHaveExpectedSignature {
                                        statement: location,
                                        function,
                                        foreign,
                                        source,
                                    });
                                }
                            }
                            _ = conjuration.dependencies.functions.remove(&foreign_id);
                        }

                        for conjuration in source_value.conjured_functions.values_mut() {
                            if let Some(expected_signature) =
                                conjuration.dependencies.functions.get(&foreign_id)
                            {
                                if !expected_signature.describes(&function_value) {
                                    return Err(CheckError::FunctionGivenToSignatureDoesNotHaveExpectedSignature {
                                        statement: location,
                                        function,
                                        foreign,
                                        source,
                                    });
                                }
                            }
                            _ = conjuration.dependencies.functions.remove(&foreign_id);
                            conjuration
                                .signature
                                .substitute_taken_function(foreign_id, &function_value);
                        }

                        let mut dependencies = function_dependencies;
                        dependencies.add(&source_dependencies);

                        SignatureLambda {
                            signature: SignatureValue::Known(source_value),
                            dependencies,
                        }
                    }
                    SignatureValue::Unknown(_) => {
                        return Err(CheckError::CannotGiveSignatureToUnknownSignature {
                            statement: location,
                            source,
                        });
                    }
                }
            }
        };

        self.signature_ids.insert(lhs.as_str(), lhs_id);
        self.signature_lambdas.insert(lhs_id, lambda);

        Ok(())
    }

    fn process_function_assignment(
        &mut self,
        assignment: FunctionAssignment<'s>,
        mut register_taken_function: impl FnMut(Function<'s>, FunctionId),
    ) -> CheckResult<'s, ()> {
        let FunctionAssignment { lhs, rhs, location } = assignment;

        let lhs_id = FunctionId::generate();

        let lambda = match rhs {
            FunctionAssignmentRhs::Take(TakeFunction { signature }) => {
                register_taken_function(lhs, lhs_id);

                let signature_id = self.resolve_signature(signature)?;
                let SignatureLambda {
                    signature,
                    dependencies: signature_dependencies,
                } = self.signature_lambda(signature_id);

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
                        given_signature_ids: signature.conjured_signature_ids,
                        given_function_ids: signature.conjured_function_ids,
                        given_signature_names: signature.conjured_signature_names,
                        given_function_names: signature.conjured_function_names,
                        taken_signature_ids: signature.taken_signature_ids,
                        taken_function_ids: signature.taken_function_ids,
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
                    self.do_as_child(|child| child.evaluate_function_context(context));
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
                    FunctionValue::Known(KnownFunctionValue {
                                             given_signatures: _,
                                             given_functions,
                                             given_signature_ids: _,
                                             given_function_ids,
                                             given_signature_names: _,
                                             given_function_names: _,
                                             taken_signature_ids: _,
                                             taken_function_ids: _,
                                         }) => {
                        let foreign_id = if let Some(&id) = given_function_ids.get(foreign.as_str())
                        {
                            id
                        } else {
                            return Err(CheckError::CannotResolveGivenFunction {
                                statement: location,
                                source,
                                foreign,
                            });
                        };

                        let foreign_lambda = given_functions
                            .get(&foreign_id)
                            .expect("could not find given function by id");

                        if !foreign_lambda.dependencies.is_empty() {
                            return Err(CheckError::GiveFunctionDependenciesNotProvided {
                                statement: location,
                                source,
                                foreign,
                            });
                        }

                        FunctionLambda {
                            function: foreign_lambda.function.clone(),
                            dependencies: source_dependencies,
                        }
                    }
                    FunctionValue::Unknown(_) => {
                        return Err(CheckError::CannotTakeSignatureFromUnknownFunction {
                            statement: location,
                            source,
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
                    signature,
                    dependencies: signature_dependencies,
                } = self.signature_lambda(signature_id);

                let source_id = self.resolve_function(source)?;
                let FunctionLambda {
                    function: source_value,
                    dependencies: source_dependencies,
                } = self.function_lambda(source_id);

                match source_value {
                    FunctionValue::Known(mut source_value) => {
                        let foreign_id = if let Some(&id) =
                            source_value.taken_signature_ids.get(foreign.as_str())
                        {
                            id
                        } else {
                            return Err(CheckError::CannotResolveTakenSignatureOfFunction {
                                statement: location,
                                source,
                                foreign,
                            });
                        };

                        for conjuration in source_value.given_signatures.values_mut() {
                            _ = conjuration.dependencies.signatures.remove(&foreign_id);
                            conjuration
                                .signature
                                .substitute_taken_signature(foreign_id, &signature);
                        }

                        for conjuration in source_value.given_functions.values_mut() {
                            _ = conjuration.dependencies.signatures.remove(&foreign_id);
                            conjuration
                                .function
                                .substitute_taken_signature(foreign_id, &signature);
                        }

                        let mut dependencies = signature_dependencies;
                        dependencies.add(&source_dependencies);

                        FunctionLambda {
                            function: FunctionValue::Known(source_value),
                            dependencies,
                        }
                    }
                    FunctionValue::Unknown(_) => {
                        return Err(CheckError::CannotGiveSignatureToUnknownFunction {
                            statement: location,
                            source,
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
                    function,
                    dependencies: function_dependencies,
                } = self.function_lambda(function_id);

                let source_id = self.resolve_function(source)?;
                let FunctionLambda {
                    function: source_value,
                    dependencies: source_dependencies,
                } = self.function_lambda(source_id);

                match source_value {
                    FunctionValue::Known(mut source_value) => {
                        let foreign_id = if let Some(&id) =
                            source_value.taken_function_ids.get(foreign.as_str())
                        {
                            id
                        } else {
                            return Err(CheckError::CannotResolveTakenFunctionOfFunction {
                                statement: location,
                                source,
                                foreign,
                            });
                        };

                        for conjuration in source_value.given_functions.values_mut() {
                            _ = conjuration.dependencies.functions.remove(&foreign_id);
                            conjuration
                                .function
                                .substitute_taken_function(foreign_id, &function);
                        }

                        for conjuration in source_value.given_functions.values_mut() {
                            _ = conjuration.dependencies.functions.remove(&foreign_id);
                            conjuration
                                .function
                                .substitute_taken_function(foreign_id, &function);
                        }

                        let mut dependencies = function_dependencies;
                        dependencies.add(&source_dependencies);

                        FunctionLambda {
                            function: FunctionValue::Known(source_value),
                            dependencies,
                        }
                    }
                    FunctionValue::Unknown(_) => {
                        return Err(CheckError::CannotGiveFunctionToUnknownFunction {
                            statement: location,
                            source,
                        });
                    }
                }
            }
        };

        self.function_ids.insert(lhs.as_str(), lhs_id);
        self.function_lambdas.insert(lhs_id, lambda);

        Ok(())
    }

    fn evaluate_signature_context(
        &mut self,
        context: SignatureContext<'s>,
    ) -> CheckResult<'s, KnownSignatureValue<'s>> {
        let mut context_value = KnownSignatureValue::default();

        for statement in context.0 {
            match statement {
                SignatureStatement::SignatureAssignment(assignment) => {
                    self.process_signature_assignment(assignment, |signature, id| {
                        context_value.taken_signature_ids.insert(signature.as_str(), id);
                    })?;
                }
                SignatureStatement::FunctionAssignment(assignment) => {
                    self.process_function_assignment(assignment, |function, id| {
                        context_value.taken_function_ids.insert(function.as_str(), id);
                    })?;
                }
                SignatureStatement::ConjureSignature(ConjureSignature {
                                                         signature,
                                                         dependencies,
                                                         location,
                                                     }) => {
                    let signature_id = SignatureId::generate();

                    if let Some(&_other_signature_id) =
                        context_value.conjured_signature_ids.get(signature.as_str())
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
                    context_value
                        .conjured_signature_ids
                        .insert(signature.as_str(), signature_id);
                    context_value
                        .conjured_signature_names
                        .insert(signature_id, signature.as_str());
                }
                SignatureStatement::ConjureFunction(ConjureFunction {
                                                        function,
                                                        signature,
                                                        dependencies,
                                                        location,
                                                    }) => {
                    let function_id = FunctionId::generate();

                    if let Some(&_other_function_id) =
                        context_value.conjured_function_ids.get(function.as_str())
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
                    context_value
                        .conjured_function_ids
                        .insert(function.as_str(), function_id);
                    context_value
                        .conjured_function_names
                        .insert(function_id, function.as_str());
                }
            }
        }

        Ok(context_value)
    }

    fn evaluate_function_context(
        &mut self,
        context: FunctionContext<'s>,
    ) -> CheckResult<'s, KnownFunctionValue<'s>> {
        let mut context_value = KnownFunctionValue::default();

        for statement in context.0 {
            match statement {
                FunctionStatement::SignatureAssignment(assignment) => {
                    self.process_signature_assignment(assignment, |signature, id| {
                        context_value.taken_signature_ids.insert(signature.as_str(), id);
                    })?;
                }
                FunctionStatement::FunctionAssignment(assignment) => {
                    self.process_function_assignment(assignment, |function, id| {
                        context_value.taken_function_ids.insert(function.as_str(), id);
                    })?;
                }
                FunctionStatement::GiveSignature(GiveSignature {
                                                     signature,
                                                     location,
                                                 }) => {
                    let signature_id = self.resolve_signature(signature)?;
                    let lambda = self.signature_lambda(signature_id);

                    if let Some(&other_signature_id) =
                        context_value.given_signature_ids.get(signature.as_str())
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
                    context_value
                        .given_signature_ids
                        .insert(signature.as_str(), signature_id);
                    context_value
                        .given_signature_names
                        .insert(signature_id, signature.as_str());
                }
                FunctionStatement::GiveFunction(GiveFunction { function, location }) => {
                    let function_id = self.resolve_function(function)?;
                    let lambda = self.function_lambda(function_id);

                    if let Some(&other_function_id) =
                        context_value.given_function_ids.get(function.as_str())
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
                    context_value
                        .given_function_ids
                        .insert(function.as_str(), function_id);
                    context_value
                        .given_function_names
                        .insert(function_id, function.as_str());
                }
            }
        }

        Ok(context_value)
    }
}
