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
struct ContextValue {
    taken_signatures: Vec<SignatureLiteralId>,
    taken_functions: Vec<FunctionLiteralId>,
    given_signatures: Vec<(SignatureLiteralId, SignatureValue)>,
    given_functions: Vec<(FunctionLiteralId, FunctionValue)>,
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
    Define {
        signature_dependencies: Vec<(SignatureLiteralId, SignatureValue)>,
        function_dependencies: Vec<(FunctionLiteralId, FunctionValue)>,
        context: Box<ContextValue>,
    },
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
    Define {
        signature: Box<SignatureValue>,
        signature_dependencies: Vec<(SignatureLiteralId, SignatureValue)>,
        function_dependencies: Vec<(FunctionLiteralId, FunctionValue)>,
        context: Box<ContextValue>,
    },
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

impl ContextValue {
    fn substitute_signature(&mut self, id: SignatureId, value: &SignatureValue) {
        for (_, other) in &mut self.given_signatures {
            other.substitute_signature(id, value);
        }
        for (_, other) in &mut self.given_functions {
            other.substitute_signature(id, value);
        }
    }

    fn substitute_function(&mut self, id: FunctionId, value: &FunctionValue) {
        for (_, other) in &mut self.given_signatures {
            other.substitute_function(id, value);
        }
        for (_, other) in &mut self.given_functions {
            other.substitute_function(id, value);
        }
    }

    fn substitute_used_signature(&mut self, id: SignatureLiteralId, value: &SignatureValue) {
        for (_, other) in &mut self.given_signatures {
            other.substitute_used_signature(id, value);
        }
        for (_, other) in &mut self.given_functions {
            other.substitute_used_signature(id, value);
        }
    }

    fn substitute_used_function(&mut self, id: FunctionLiteralId, value: &FunctionValue) {
        for (_, other) in &mut self.given_signatures {
            other.substitute_used_function(id, value);
        }
        for (_, other) in &mut self.given_functions {
            other.substitute_used_function(id, value);
        }
    }

    fn substitute_taken_signature(&mut self, id: SignatureLiteralId, value: &SignatureValue) {
        for (_, other) in &mut self.given_signatures {
            other.substitute_taken_signature(id, value);
        }
        for (_, other) in &mut self.given_functions {
            other.substitute_taken_signature(id, value);
        }
    }

    fn substitute_taken_function(&mut self, id: FunctionLiteralId, value: &FunctionValue) {
        for (_, other) in &mut self.given_signatures {
            other.substitute_taken_function(id, value);
        }
        for (_, other) in &mut self.given_functions {
            other.substitute_taken_function(id, value);
        }
    }
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
            SignatureValue::Define {
                signature_dependencies,
                function_dependencies,
                context,
            } => {
                for (_, dependency) in signature_dependencies {
                    dependency.substitute_signature(id, value);
                }
                for (_, dependency) in function_dependencies {
                    dependency.substitute_signature(id, value);
                }
                context.substitute_signature(id, value);
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
            SignatureValue::Define {
                signature_dependencies,
                function_dependencies,
                context,
            } => {
                for (_, dependency) in signature_dependencies {
                    dependency.substitute_function(id, value);
                }
                for (_, dependency) in function_dependencies {
                    dependency.substitute_function(id, value);
                }
                context.substitute_function(id, value);
            }
            SignatureValue::TakeFrom { literal, source } => {
                _ = literal;
                source.substitute_function(id, value);
            }
        }
    }

    fn substitute_used_signature(&mut self, id: SignatureLiteralId, value: &SignatureValue) {
        match self {
            SignatureValue::Variable { signature } => {
                _ = signature;
            }
            SignatureValue::Use { literal } => {
                if *literal == id {
                    *self = value.clone();
                }
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
                    dependency.substitute_used_signature(id, value);
                }
                for dependency in function_dependencies {
                    dependency.substitute_used_signature(id, value);
                }
            }
            SignatureValue::Define {
                signature_dependencies,
                function_dependencies,
                context,
            } => {
                for (_, dependency) in signature_dependencies {
                    dependency.substitute_used_signature(id, value);
                }
                for (_, dependency) in function_dependencies {
                    dependency.substitute_used_signature(id, value);
                }
                context.substitute_used_signature(id, value);
            }
            SignatureValue::TakeFrom { literal, source } => {
                _ = literal;
                source.substitute_used_signature(id, value);
            }
        }
    }

    fn substitute_used_function(&mut self, id: FunctionLiteralId, value: &FunctionValue) {
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
                    dependency.substitute_used_function(id, value);
                }
                for dependency in function_dependencies {
                    dependency.substitute_used_function(id, value);
                }
            }
            SignatureValue::Define {
                signature_dependencies,
                function_dependencies,
                context,
            } => {
                for (_, dependency) in signature_dependencies {
                    dependency.substitute_used_function(id, value);
                }
                for (_, dependency) in function_dependencies {
                    dependency.substitute_used_function(id, value);
                }
                context.substitute_used_function(id, value);
            }
            SignatureValue::TakeFrom { literal, source } => {
                _ = literal;
                source.substitute_used_function(id, value);
            }
        }
    }

    fn substitute_taken_signature(&mut self, id: SignatureLiteralId, value: &SignatureValue) {
        match self {
            SignatureValue::Variable { signature } => {
                _ = signature;
            }
            SignatureValue::Use { literal } => {
                _ = literal;
            }
            SignatureValue::Take { literal } => {
                if *literal == id {
                    *self = value.clone();
                }
            }
            SignatureValue::Conjure {
                marker,
                signature_dependencies,
                function_dependencies,
            } => {
                _ = marker;
                for dependency in signature_dependencies {
                    dependency.substitute_taken_signature(id, value);
                }
                for dependency in function_dependencies {
                    dependency.substitute_taken_signature(id, value);
                }
            }
            SignatureValue::Define {
                signature_dependencies,
                function_dependencies,
                context,
            } => {
                for (_, dependency) in signature_dependencies {
                    dependency.substitute_taken_signature(id, value);
                }
                for (_, dependency) in function_dependencies {
                    dependency.substitute_taken_signature(id, value);
                }
                context.substitute_taken_signature(id, value);
            }
            SignatureValue::TakeFrom { literal, source } => {
                _ = literal;
                source.substitute_taken_signature(id, value);
            }
        }
    }

    fn substitute_taken_function(&mut self, id: FunctionLiteralId, value: &FunctionValue) {
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
                    dependency.substitute_taken_function(id, value);
                }
                for dependency in function_dependencies {
                    dependency.substitute_taken_function(id, value);
                }
            }
            SignatureValue::Define {
                signature_dependencies,
                function_dependencies,
                context,
            } => {
                for (_, dependency) in signature_dependencies {
                    dependency.substitute_taken_function(id, value);
                }
                for (_, dependency) in function_dependencies {
                    dependency.substitute_taken_function(id, value);
                }
                context.substitute_taken_function(id, value);
            }
            SignatureValue::TakeFrom { literal, source } => {
                _ = literal;
                source.substitute_taken_function(id, value);
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
            FunctionValue::Define {
                signature,
                signature_dependencies,
                function_dependencies,
                context,
            } => {
                signature.substitute_signature(id, value);
                for (_, dependency) in signature_dependencies {
                    dependency.substitute_signature(id, value);
                }
                for (_, dependency) in function_dependencies {
                    dependency.substitute_signature(id, value);
                }
                context.substitute_signature(id, value);
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
            FunctionValue::Define {
                signature,
                signature_dependencies,
                function_dependencies,
                context,
            } => {
                signature.substitute_function(id, value);
                for (_, dependency) in signature_dependencies {
                    dependency.substitute_function(id, value);
                }
                for (_, dependency) in function_dependencies {
                    dependency.substitute_function(id, value);
                }
                context.substitute_function(id, value);
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

    fn substitute_used_signature(&mut self, id: SignatureLiteralId, value: &SignatureValue) {
        match self {
            FunctionValue::Variable { function } => {
                _ = function;
            }
            FunctionValue::Use { signature, literal } => {
                signature.substitute_used_signature(id, value);
                _ = literal;
            }
            FunctionValue::Take { signature, literal } => {
                signature.substitute_used_signature(id, value);
                _ = literal;
            }
            FunctionValue::Conjure {
                marker,
                signature,
                signature_dependencies,
                function_dependencies,
            } => {
                _ = marker;
                signature.substitute_used_signature(id, value);
                for dependency in signature_dependencies {
                    dependency.substitute_used_signature(id, value);
                }
                for dependency in function_dependencies {
                    dependency.substitute_used_signature(id, value);
                }
            }
            FunctionValue::Define {
                signature,
                signature_dependencies,
                function_dependencies,
                context,
            } => {
                signature.substitute_used_signature(id, value);
                for (_, dependency) in signature_dependencies {
                    dependency.substitute_used_signature(id, value);
                }
                for (_, dependency) in function_dependencies {
                    dependency.substitute_used_signature(id, value);
                }
                context.substitute_used_signature(id, value);
            }
            FunctionValue::TakeFrom { literal, source } => {
                _ = literal;
                source.substitute_used_signature(id, value);
            }
            FunctionValue::GiveSignatureTo {
                signature,
                literal,
                source,
            } => {
                signature.substitute_used_signature(id, value);
                _ = literal;
                source.substitute_used_signature(id, value);
            }
            FunctionValue::GiveFunctionTo {
                function,
                literal,
                source,
            } => {
                function.substitute_used_signature(id, value);
                _ = literal;
                source.substitute_used_signature(id, value);
            }
        }
    }

    fn substitute_used_function(&mut self, id: FunctionLiteralId, value: &FunctionValue) {
        match self {
            FunctionValue::Variable { function } => {
                _ = function;
            }
            FunctionValue::Use { signature, literal } => {
                signature.substitute_used_function(id, value);
                if *literal == id {
                    *self = value.clone();
                }
            }
            FunctionValue::Take { signature, literal } => {
                signature.substitute_used_function(id, value);
                _ = literal;
            }
            FunctionValue::Conjure {
                marker,
                signature,
                signature_dependencies,
                function_dependencies,
            } => {
                _ = marker;
                signature.substitute_used_function(id, value);
                for dependency in signature_dependencies {
                    dependency.substitute_used_function(id, value);
                }
                for dependency in function_dependencies {
                    dependency.substitute_used_function(id, value);
                }
            }
            FunctionValue::Define {
                signature,
                signature_dependencies,
                function_dependencies,
                context,
            } => {
                signature.substitute_used_function(id, value);
                for (_, dependency) in signature_dependencies {
                    dependency.substitute_used_function(id, value);
                }
                for (_, dependency) in function_dependencies {
                    dependency.substitute_used_function(id, value);
                }
                context.substitute_used_function(id, value);
            }
            FunctionValue::TakeFrom { literal, source } => {
                _ = literal;
                source.substitute_used_function(id, value);
            }
            FunctionValue::GiveSignatureTo {
                signature,
                literal,
                source,
            } => {
                signature.substitute_used_function(id, value);
                _ = literal;
                source.substitute_used_function(id, value);
            }
            FunctionValue::GiveFunctionTo {
                function,
                literal,
                source,
            } => {
                function.substitute_used_function(id, value);
                _ = literal;
                source.substitute_used_function(id, value);
            }
        }
    }

    fn substitute_taken_signature(&mut self, id: SignatureLiteralId, value: &SignatureValue) {
        match self {
            FunctionValue::Variable { function } => {
                _ = function;
            }
            FunctionValue::Use { signature, literal } => {
                signature.substitute_taken_signature(id, value);
                _ = literal;
            }
            FunctionValue::Take { signature, literal } => {
                signature.substitute_taken_signature(id, value);
                _ = literal;
            }
            FunctionValue::Conjure {
                marker,
                signature,
                signature_dependencies,
                function_dependencies,
            } => {
                _ = marker;
                signature.substitute_taken_signature(id, value);
                for dependency in signature_dependencies {
                    dependency.substitute_taken_signature(id, value);
                }
                for dependency in function_dependencies {
                    dependency.substitute_taken_signature(id, value);
                }
            }
            FunctionValue::Define {
                signature,
                signature_dependencies,
                function_dependencies,
                context,
            } => {
                signature.substitute_taken_signature(id, value);
                for (_, dependency) in signature_dependencies {
                    dependency.substitute_taken_signature(id, value);
                }
                for (_, dependency) in function_dependencies {
                    dependency.substitute_taken_signature(id, value);
                }
                context.substitute_taken_signature(id, value);
            }
            FunctionValue::TakeFrom { literal, source } => {
                _ = literal;
                source.substitute_taken_signature(id, value);
            }
            FunctionValue::GiveSignatureTo {
                signature,
                literal,
                source,
            } => {
                signature.substitute_taken_signature(id, value);
                _ = literal;
                source.substitute_taken_signature(id, value);
            }
            FunctionValue::GiveFunctionTo {
                function,
                literal,
                source,
            } => {
                function.substitute_taken_signature(id, value);
                _ = literal;
                source.substitute_taken_signature(id, value);
            }
        }
    }

    fn substitute_taken_function(&mut self, id: FunctionLiteralId, value: &FunctionValue) {
        match self {
            FunctionValue::Variable { function } => {
                _ = function;
            }
            FunctionValue::Use { signature, literal } => {
                signature.substitute_taken_function(id, value);
                _ = literal;
            }
            FunctionValue::Take { signature, literal } => {
                signature.substitute_taken_function(id, value);
                if *literal == id {
                    *self = value.clone();
                }
            }
            FunctionValue::Conjure {
                marker,
                signature,
                signature_dependencies,
                function_dependencies,
            } => {
                _ = marker;
                signature.substitute_taken_function(id, value);
                for dependency in signature_dependencies {
                    dependency.substitute_taken_function(id, value);
                }
                for dependency in function_dependencies {
                    dependency.substitute_taken_function(id, value);
                }
            }
            FunctionValue::Define {
                signature,
                signature_dependencies,
                function_dependencies,
                context,
            } => {
                signature.substitute_taken_function(id, value);
                for (_, dependency) in signature_dependencies {
                    dependency.substitute_taken_function(id, value);
                }
                for (_, dependency) in function_dependencies {
                    dependency.substitute_taken_function(id, value);
                }
                context.substitute_taken_function(id, value);
            }
            FunctionValue::TakeFrom { literal, source } => {
                _ = literal;
                source.substitute_taken_function(id, value);
            }
            FunctionValue::GiveSignatureTo {
                signature,
                literal,
                source,
            } => {
                signature.substitute_taken_function(id, value);
                _ = literal;
                source.substitute_taken_function(id, value);
            }
            FunctionValue::GiveFunctionTo {
                function,
                literal,
                source,
            } => {
                function.substitute_taken_function(id, value);
                _ = literal;
                source.substitute_taken_function(id, value);
            }
        }
    }
}

impl ContextValue {
    fn enumerate_conjurations(
        &mut self,
        signature_enumeration: &mut HashMap<SignatureId, u64>,
        function_enumeration: &mut HashMap<FunctionId, u64>,
    ) {
        for (_, other) in &mut self.given_signatures {
            other.enumerate_conjurations(signature_enumeration, function_enumeration);
        }
        for (_, other) in &mut self.given_functions {
            other.enumerate_conjurations(signature_enumeration, function_enumeration);
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
            SignatureValue::Define {
                signature_dependencies,
                function_dependencies,
                context,
            } => {
                for (_, dependency) in signature_dependencies {
                    dependency.enumerate_conjurations(signature_enumeration, function_enumeration);
                }
                for (_, dependency) in function_dependencies {
                    dependency.enumerate_conjurations(signature_enumeration, function_enumeration);
                }
                context.enumerate_conjurations(signature_enumeration, function_enumeration);
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
            FunctionValue::Define {
                signature,
                signature_dependencies,
                function_dependencies,
                context,
            } => {
                signature.enumerate_conjurations(signature_enumeration, function_enumeration);
                for (_, dependency) in signature_dependencies {
                    dependency.enumerate_conjurations(signature_enumeration, function_enumeration);
                }
                for (_, dependency) in function_dependencies {
                    dependency.enumerate_conjurations(signature_enumeration, function_enumeration);
                }
                context.enumerate_conjurations(signature_enumeration, function_enumeration);
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

impl ContextValue {
    fn reduce(&mut self) {
        for (_, other) in &mut self.given_signatures {
            other.reduce();
        }
        for (_, other) in &mut self.given_functions {
            other.reduce();
        }
    }
}

impl SignatureValue {
    fn reduce(&mut self) {
        match self {
            SignatureValue::Variable { .. } => {
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
                _ = marker;
                for dependency in signature_dependencies {
                    dependency.reduce();
                }
                for dependency in function_dependencies {
                    dependency.reduce();
                }
            }
            SignatureValue::Define {
                signature_dependencies,
                function_dependencies,
                context,
            } => {
                for (_, dependency) in signature_dependencies.iter_mut() {
                    dependency.reduce();
                }
                for (_, dependency) in function_dependencies.iter_mut() {
                    dependency.reduce();
                }

                for (id, value) in signature_dependencies.drain(..) {
                    context.substitute_used_signature(id, &value);
                }
                for (id, value) in function_dependencies.drain(..) {
                    context.substitute_used_function(id, &value);
                }

                context.reduce();
            }
            SignatureValue::TakeFrom { literal, source } => {
                _ = literal;
                source.reduce();

                if let FunctionValue::Define {
                    signature: _,
                    signature_dependencies: _,
                    function_dependencies: _,
                    context,
                } = &**source
                {
                    let (_, given_value) = context
                        .given_signatures
                        .iter()
                        .find(|&&(given_literal, _)| given_literal == *literal)
                        .unwrap();
                    *self = given_value.clone();
                }
            }
        }
    }
}

impl FunctionValue {
    fn reduce(&mut self) {
        match self {
            FunctionValue::Variable { .. } => {
                unreachable!()
            }
            FunctionValue::Use { signature, literal } => {
                signature.reduce();
                _ = literal;
            }
            FunctionValue::Take { signature, literal } => {
                signature.reduce();
                _ = literal;
            }
            FunctionValue::Conjure {
                marker,
                signature,
                signature_dependencies,
                function_dependencies,
            } => {
                _ = marker;
                signature.reduce();
                for dependency in signature_dependencies {
                    dependency.reduce();
                }
                for dependency in function_dependencies {
                    dependency.reduce();
                }
            }
            FunctionValue::Define {
                signature,
                signature_dependencies,
                function_dependencies,
                context,
            } => {
                signature.reduce();
                for (_, dependency) in signature_dependencies.iter_mut() {
                    dependency.reduce();
                }
                for (_, dependency) in function_dependencies.iter_mut() {
                    dependency.reduce();
                }

                for (id, value) in signature_dependencies.drain(..) {
                    context.substitute_used_signature(id, &value);
                }
                for (id, value) in function_dependencies.drain(..) {
                    context.substitute_used_function(id, &value);
                }

                context.reduce();
            }
            FunctionValue::TakeFrom { literal, source } => {
                _ = literal;
                source.reduce();

                if let FunctionValue::Define {
                    signature: _,
                    signature_dependencies: _,
                    function_dependencies: _,
                    context,
                } = &**source
                {
                    let (_, given_value) = context
                        .given_functions
                        .iter()
                        .find(|&&(given_literal, _)| given_literal == *literal)
                        .unwrap();
                    *self = given_value.clone();
                }
            }
            FunctionValue::GiveSignatureTo {
                signature,
                literal,
                source,
            } => {
                signature.reduce();
                _ = literal;

                source.substitute_taken_signature(*literal, &**signature);

                source.reduce();
            }
            FunctionValue::GiveFunctionTo {
                function,
                literal,
                source,
            } => {
                function.reduce();
                _ = literal;

                source.substitute_taken_function(*literal, &**function);

                source.reduce();
            }
        }
    }
}

impl<'s> Model<'s> {
    pub(crate) fn check(&self) {
        let mut context_values: HashMap<ContextId, ContextValue> = HashMap::new();

        for &context in &self.contexts {
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
                            SignatureAssignmentRhs::Define {
                                ref signature_dependencies,
                                ref function_dependencies,
                                context,
                            } => SignatureValue::Define {
                                signature_dependencies: signature_dependencies
                                    .iter()
                                    .map(|dependency| {
                                        (
                                            dependency.literal,
                                            SignatureValue::Variable {
                                                signature: dependency.signature,
                                            },
                                        )
                                    })
                                    .collect(),
                                function_dependencies: function_dependencies
                                    .iter()
                                    .map(|dependency| {
                                        (
                                            dependency.literal,
                                            FunctionValue::Variable {
                                                function: dependency.function,
                                            },
                                        )
                                    })
                                    .collect(),
                                context: Box::new(context_values[&context].clone()),
                            },
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
                            FunctionAssignmentRhs::Define {
                                signature,
                                ref signature_dependencies,
                                ref function_dependencies,
                                context,
                            } => FunctionValue::Define {
                                signature: Box::new(SignatureValue::Variable { signature }),
                                signature_dependencies: signature_dependencies
                                    .iter()
                                    .map(|dependency| {
                                        (
                                            dependency.literal,
                                            SignatureValue::Variable {
                                                signature: dependency.signature,
                                            },
                                        )
                                    })
                                    .collect(),
                                function_dependencies: function_dependencies
                                    .iter()
                                    .map(|dependency| {
                                        (
                                            dependency.literal,
                                            FunctionValue::Variable {
                                                function: dependency.function,
                                            },
                                        )
                                    })
                                    .collect(),
                                context: Box::new(context_values[&context].clone()),
                            },
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

            dbg!(&signature_values);
            dbg!(&function_values);

            for &(_literal, signature) in &given_signatures {
                let value = signature_values.get_mut(&signature).unwrap();
                value.reduce();
            }
            for &(_literal, function) in &given_functions {
                let value = function_values.get_mut(&function).unwrap();
                value.reduce();
            }

            dbg!(&signature_values);
            dbg!(&function_values);

            /*let mut signature_enumeration = HashMap::new();
            let mut function_enumeration = HashMap::new();
            for &(_literal, signature) in &given_signatures {
                let value = signature_values.get_mut(&signature).unwrap();
                value.enumerate_conjurations(&mut signature_enumeration, &mut function_enumeration);
            }
            for &(_literal, function) in &given_functions {
                let value = function_values.get_mut(&function).unwrap();
                value.enumerate_conjurations(&mut signature_enumeration, &mut function_enumeration);
            }*/

            context_values.insert(context, ContextValue {
                taken_signatures,
                taken_functions,
                given_signatures: given_signatures.into_iter().map(|(literal, signature)|
                    (literal, signature_values.remove(&signature).unwrap())
                ).collect(),
                given_functions: given_functions.into_iter().map(|(literal, function)|
                    (literal, function_values.remove(&function).unwrap())
                ).collect(),
            });

            println!("\n=====\n=====\n=====\n");

            // Four stages:
            // 0. rewrite assignments as values
            // 1. substitute assignments
            // 2. reduce definitions, take-from's and give-to's
            // 3. enumerate conjurations
        }
    }
}
