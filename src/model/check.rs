use crate::model::*;
use itertools::Itertools;
use std::fmt;
use std::fmt::Formatter;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum ConjureSignatureMarker {
    Id(SignatureId),
    #[allow(unused)]
    Index(u64),
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum ConjureFunctionMarker {
    Id(FunctionId),
    #[allow(unused)]
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
    Take {
        literal: SignatureLiteralId,
    },
    Conjure {
        marker: ConjureSignatureMarker,
    },
    Define {
        context: Box<ContextValue>,
    },
    TakeFrom {
        literal: SignatureLiteralId,
        source: Box<FunctionValue>,
    },
    GiveSignatureToSignature {
        signature: Box<SignatureValue>,
        literal: SignatureLiteralId,
        source: Box<SignatureValue>,
    },
    GiveFunctionToSignature {
        function: Box<FunctionValue>,
        literal: FunctionLiteralId,
        source: Box<SignatureValue>,
    },
}

#[derive(Debug, Clone, Eq, PartialEq)]
enum FunctionValue {
    Take {
        signature: Box<SignatureValue>,
        literal: FunctionLiteralId,
    },
    Conjure {
        marker: ConjureFunctionMarker,
        signature: Box<SignatureValue>,
    },
    Define {
        context: Box<ContextValue>,
    },
    TakeFrom {
        literal: FunctionLiteralId,
        source: Box<FunctionValue>,
    },
    GiveSignatureToFunction {
        signature: Box<SignatureValue>,
        literal: SignatureLiteralId,
        source: Box<FunctionValue>,
    },
    GiveFunctionToFunction {
        function: Box<FunctionValue>,
        literal: FunctionLiteralId,
        source: Box<FunctionValue>,
    },
}

impl ContextValue {
    fn substitute_taken_signature(&mut self, id: SignatureLiteralId, value: &SignatureValue) {
        if let Ok(index) = self.taken_signatures.binary_search(&id) {
            self.taken_signatures.remove(index);
        }
        for (_, other) in &mut self.given_signatures {
            other.substitute_taken_signature(id, value);
        }
        for (_, other) in &mut self.given_functions {
            other.substitute_taken_signature(id, value);
        }
    }

    fn substitute_taken_function(&mut self, id: FunctionLiteralId, value: &FunctionValue) {
        if let Ok(index) = self.taken_functions.binary_search(&id) {
            self.taken_functions.remove(index);
        }
        for (_, other) in &mut self.given_signatures {
            other.substitute_taken_function(id, value);
        }
        for (_, other) in &mut self.given_functions {
            other.substitute_taken_function(id, value);
        }
    }
}

impl SignatureValue {
    fn substitute_taken_signature(&mut self, id: SignatureLiteralId, value: &SignatureValue) {
        match self {
            SignatureValue::Take { literal } => {
                if *literal == id {
                    *self = value.clone();
                }
            }
            SignatureValue::Conjure {
                marker,
            } => {
                _ = marker;
            }
            SignatureValue::Define { context } => {
                context.substitute_taken_signature(id, value);
            }
            SignatureValue::TakeFrom { literal, source } => {
                _ = literal;
                source.substitute_taken_signature(id, value);
            }
            SignatureValue::GiveSignatureToSignature {
                signature,
                literal,
                source,
            } => {
                signature.substitute_taken_signature(id, value);
                _ = literal;
                source.substitute_taken_signature(id, value);
            }
            SignatureValue::GiveFunctionToSignature {
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
            SignatureValue::Take { literal } => {
                _ = literal;
            }
            SignatureValue::Conjure {
                marker,
            } => {
                _ = marker;
            }
            SignatureValue::Define { context } => {
                context.substitute_taken_function(id, value);
            }
            SignatureValue::TakeFrom { literal, source } => {
                _ = literal;
                source.substitute_taken_function(id, value);
            }
            SignatureValue::GiveSignatureToSignature {
                signature,
                literal,
                source,
            } => {
                signature.substitute_taken_function(id, value);
                _ = literal;
                source.substitute_taken_function(id, value);
            }
            SignatureValue::GiveFunctionToSignature {
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

impl FunctionValue {
    fn substitute_taken_signature(&mut self, id: SignatureLiteralId, value: &SignatureValue) {
        match self {
            FunctionValue::Take { signature, literal } => {
                signature.substitute_taken_signature(id, value);
                _ = literal;
            }
            FunctionValue::Conjure {
                marker,
                signature,
            } => {
                _ = marker;
                signature.substitute_taken_signature(id, value);
            }
            FunctionValue::Define { context } => {
                context.substitute_taken_signature(id, value);
            }
            FunctionValue::TakeFrom { literal, source } => {
                _ = literal;
                source.substitute_taken_signature(id, value);
            }
            FunctionValue::GiveSignatureToFunction {
                signature,
                literal,
                source,
            } => {
                signature.substitute_taken_signature(id, value);
                _ = literal;
                source.substitute_taken_signature(id, value);
            }
            FunctionValue::GiveFunctionToFunction {
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
            FunctionValue::Take { signature, literal } => {
                signature.substitute_taken_function(id, value);
                if *literal == id {
                    *self = value.clone();
                }
            }
            FunctionValue::Conjure {
                marker,
                signature,
            } => {
                _ = marker;
                signature.substitute_taken_function(id, value);
            }
            FunctionValue::Define { context } => {
                context.substitute_taken_function(id, value);
            }
            FunctionValue::TakeFrom { literal, source } => {
                _ = literal;
                source.substitute_taken_function(id, value);
            }
            FunctionValue::GiveSignatureToFunction {
                signature,
                literal,
                source,
            } => {
                signature.substitute_taken_function(id, value);
                _ = literal;
                source.substitute_taken_function(id, value);
            }
            FunctionValue::GiveFunctionToFunction {
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
    #[allow(unused)]
    fn enumerate_conjurations(
        &mut self,
        signature_enumeration: &mut HashMap<SignatureId, u64>,
        function_enumeration: &mut HashMap<FunctionId, u64>,
    ) {
        match self {
            SignatureValue::Take { literal } => {
                _ = literal;
            }
            SignatureValue::Conjure {
                marker,
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
            }
            SignatureValue::Define { context } => {
                context.enumerate_conjurations(signature_enumeration, function_enumeration);
            }
            SignatureValue::TakeFrom { literal, source } => {
                _ = literal;
                source.enumerate_conjurations(signature_enumeration, function_enumeration);
            }
            SignatureValue::GiveSignatureToSignature {
                signature,
                literal,
                source,
            } => {
                signature.enumerate_conjurations(signature_enumeration, function_enumeration);
                _ = literal;
                source.enumerate_conjurations(signature_enumeration, function_enumeration);
            }
            SignatureValue::GiveFunctionToSignature {
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

impl FunctionValue {
    #[allow(unused)]
    fn enumerate_conjurations(
        &mut self,
        signature_enumeration: &mut HashMap<SignatureId, u64>,
        function_enumeration: &mut HashMap<FunctionId, u64>,
    ) {
        match self {
            FunctionValue::Take { signature, literal } => {
                signature.enumerate_conjurations(signature_enumeration, function_enumeration);
                _ = literal;
            }
            FunctionValue::Conjure {
                marker,
                signature,
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
            }
            FunctionValue::Define { context } => {
                context.enumerate_conjurations(signature_enumeration, function_enumeration);
            }
            FunctionValue::TakeFrom { literal, source } => {
                _ = literal;
                source.enumerate_conjurations(signature_enumeration, function_enumeration);
            }
            FunctionValue::GiveSignatureToFunction {
                signature,
                literal,
                source,
            } => {
                signature.enumerate_conjurations(signature_enumeration, function_enumeration);
                _ = literal;
                source.enumerate_conjurations(signature_enumeration, function_enumeration);
            }
            FunctionValue::GiveFunctionToFunction {
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
            SignatureValue::Take { literal } => {
                _ = literal;
            }
            SignatureValue::Conjure {
                marker,
            } => {
                _ = marker;
            }
            SignatureValue::Define { context } => {
                context.reduce();
            }
            SignatureValue::TakeFrom { literal, source } => {
                _ = literal;
                source.reduce();

                println!("reducing SignatureValue::TakeFrom, source: {:#?}", source);

                if let FunctionValue::Define {
                    context,
                } = &**source
                {
                    let (_, given_value) = context
                        .given_signatures
                        .iter()
                        .find(|&&(given_literal, _)| given_literal == *literal)
                        .unwrap();
                    // context is discarded here
                    *self = given_value.clone();
                }
            }
            SignatureValue::GiveSignatureToSignature {
                signature,
                literal,
                source,
            } => {
                signature.reduce();
                _ = literal;

                if let SignatureValue::Define {
                    context,
                } = &mut **source
                {
                    context.substitute_taken_signature(*literal, &**signature);
                    context.reduce();

                    *self = *source.clone();
                }
            }
            SignatureValue::GiveFunctionToSignature {
                function,
                literal,
                source,
            } => {
                function.reduce();
                _ = literal;

                if let SignatureValue::Define {
                    context,
                } = &mut **source
                {
                    context.substitute_taken_function(*literal, &**function);
                    context.reduce();

                    *self = *source.clone();
                }
            }
        }
    }
}

impl FunctionValue {
    fn reduce(&mut self) {
        match self {
            FunctionValue::Take { signature, literal } => {
                signature.reduce();
                _ = literal;
            }
            FunctionValue::Conjure {
                marker,
                signature,
            } => {
                _ = marker;
                signature.reduce();
            }
            FunctionValue::Define { context } => {
                context.reduce();
            }
            FunctionValue::TakeFrom { literal, source } => {
                _ = literal;
                source.reduce();

                println!("reducing FunctionValue::TakeFrom, source: {:#?}", source);

                if let FunctionValue::Define {
                    context,
                } = &**source
                {
                    let (_, given_value) = context
                        .given_functions
                        .iter()
                        .find(|&&(given_literal, _)| given_literal == *literal)
                        .unwrap();
                    // context is discarded here
                    *self = given_value.clone();
                }
            }
            FunctionValue::GiveSignatureToFunction {
                signature,
                literal,
                source,
            } => {
                signature.reduce();
                _ = literal;

                if let FunctionValue::Define {
                    context,
                } = &mut **source
                {
                    context.substitute_taken_signature(*literal, &**signature);
                    context.reduce();

                    *self = *source.clone();
                }
            }
            FunctionValue::GiveFunctionToFunction {
                function,
                literal,
                source,
            } => {
                function.reduce();
                _ = literal;

                if let FunctionValue::Define {
                    context,
                } = &mut **source
                {
                    context.substitute_taken_function(*literal, &**function);
                    context.reduce();

                    *self = *source.clone();
                }
            }
        }
    }
}

struct Values {
    context: HashMap<ContextId, ContextValue>,
    signature: HashMap<SignatureId, SignatureValue>,
    function: HashMap<FunctionId, FunctionValue>,
}

impl<'s> Model<'s> {
    fn check_context(&self, context: ContextId, values: &mut Values) {
        let contents = &self.context_contents[&context];

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

        for statement in &contents.statements {
            match *statement {
                Statement::SignatureAssignment { lhs, ref rhs } => {
                    let mut value = match *rhs {
                        SignatureAssignmentRhs::Take { literal } => {
                            SignatureValue::Take { literal }
                        }
                        SignatureAssignmentRhs::Conjure => SignatureValue::Conjure {
                            marker: ConjureSignatureMarker::Id(lhs),
                        },
                        SignatureAssignmentRhs::Define { context } => {
                            self.check_context(context, values);
                            SignatureValue::Define {
                                context: Box::new(values.context[&context].clone()),
                            }
                        }
                        SignatureAssignmentRhs::TakeFrom { literal, source } => {
                            SignatureValue::TakeFrom {
                                literal,
                                source: Box::new(values.function[&source].clone()),
                            }
                        }
                        SignatureAssignmentRhs::GiveSignatureToSignature {
                            signature,
                            literal,
                            source,
                        } => SignatureValue::GiveSignatureToSignature {
                            signature: Box::new(values.signature[&signature].clone()),
                            literal,
                            source: Box::new(values.signature[&source].clone()),
                        },
                        SignatureAssignmentRhs::GiveFunctionToSignature {
                            function,
                            literal,
                            source,
                        } => SignatureValue::GiveFunctionToSignature {
                            function: Box::new(values.function[&function].clone()),
                            literal,
                            source: Box::new(values.signature[&source].clone()),
                        },
                    };
                    value.reduce();
                    values.signature.insert(lhs, value);
                }
                Statement::FunctionAssignment { lhs, ref rhs } => {
                    let mut value = match *rhs {
                        FunctionAssignmentRhs::Take { signature, literal } => FunctionValue::Take {
                            signature: Box::new(values.signature[&signature].clone()),
                            literal,
                        },
                        FunctionAssignmentRhs::Conjure {
                            signature,
                        } => FunctionValue::Conjure {
                            signature: Box::new(values.signature[&signature].clone()),
                            marker: ConjureFunctionMarker::Id(lhs),
                        },
                        FunctionAssignmentRhs::Define { context } => {
                            self.check_context(context, values);
                            FunctionValue::Define {
                                context: Box::new(values.context[&context].clone()),
                            }
                        }
                        FunctionAssignmentRhs::TakeFrom { literal, source } => {
                            FunctionValue::TakeFrom {
                                literal,
                                source: Box::new(values.function[&source].clone()),
                            }
                        }
                        FunctionAssignmentRhs::GiveSignatureToFunction {
                            signature,
                            literal,
                            source,
                        } => FunctionValue::GiveSignatureToFunction {
                            signature: Box::new(values.signature[&signature].clone()),
                            literal,
                            source: Box::new(values.function[&source].clone()),
                        },
                        FunctionAssignmentRhs::GiveFunctionToFunction {
                            function,
                            literal,
                            source,
                        } => FunctionValue::GiveFunctionToFunction {
                            function: Box::new(values.function[&function].clone()),
                            literal,
                            source: Box::new(values.function[&source].clone()),
                        },
                    };
                    value.reduce();
                    values.function.insert(lhs, value);
                }
                _ => {}
            }
        }

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

        let context_value = ContextValue {
            taken_signatures,
            taken_functions,
            given_signatures: given_signatures
                .into_iter()
                .map(|(literal, signature)| {
                    (literal, values.signature.get(&signature).unwrap().clone())
                })
                .collect(),
            given_functions: given_functions
                .into_iter()
                .map(|(literal, function)| {
                    (literal, values.function.get(&function).unwrap().clone())
                })
                .collect(),
        };

        values.context.insert(context, context_value);
    }

    pub(crate) fn check(&self) {
        let mut values = Values {
            context: Default::default(),
            signature: Default::default(),
            function: Default::default(),
        };
        self.check_context(self.global_context, &mut values);

        let context_value = &values.context[&self.global_context];
        println!("{:#?}", fmt::from_fn(|fmt| self.debug_context_value(fmt, context_value)));
    }

    fn debug_signature_literal(
        &self,
        fmt: &mut Formatter,
        literal: SignatureLiteralId,
    ) -> fmt::Result {
        let name = self.signature_literal_names[&literal];
        write!(fmt, "'({name})'")
    }

    fn debug_function_literal(
        &self,
        fmt: &mut Formatter,
        literal: FunctionLiteralId,
    ) -> fmt::Result {
        let name = self.function_literal_names[&literal];
        write!(fmt, "'{name}'")
    }

    fn debug_signature(&self, fmt: &mut Formatter, signature: SignatureId) -> fmt::Result {
        let name = self.signature_names[&signature];
        write!(fmt, "({name})")
    }

    fn debug_function(&self, fmt: &mut Formatter, function: FunctionId) -> fmt::Result {
        let name = self.function_names[&function];
        write!(fmt, "{name}")
    }

    fn debug_conjure_signature_marker(
        &self,
        fmt: &mut Formatter,
        marker: ConjureSignatureMarker,
    ) -> fmt::Result {
        match marker {
            ConjureSignatureMarker::Id(id) => {
                write!(fmt, "Marker::Id(")?;
                self.debug_signature(fmt, id)?;
                write!(fmt, ")")
            }
            ConjureSignatureMarker::Index(index) => {
                write!(fmt, "Marker::Index({index})")
            }
        }
    }

    fn debug_conjure_function_marker(
        &self,
        fmt: &mut Formatter,
        marker: ConjureFunctionMarker,
    ) -> fmt::Result {
        match marker {
            ConjureFunctionMarker::Id(id) => {
                write!(fmt, "Marker::Id(")?;
                self.debug_function(fmt, id)?;
                write!(fmt, ")")
            }
            ConjureFunctionMarker::Index(index) => {
                write!(fmt, "Marker::Index({index})")
            }
        }
    }

    fn debug_signature_value(&self, fmt: &mut Formatter, value: &SignatureValue) -> fmt::Result {
        match *value {
            SignatureValue::Take { literal } => fmt
                .debug_struct("TakeSignature")
                .field_with("literal", |fmt| self.debug_signature_literal(fmt, literal))
                .finish(),
            SignatureValue::Conjure {
                marker,
            } => fmt
                .debug_struct("ConjureSignature")
                .field_with("marker", |fmt| {
                    self.debug_conjure_signature_marker(fmt, marker)
                })
                .finish(),
            SignatureValue::Define { ref context } => fmt
                .debug_struct("DefineSignature")
                .field_with("context", |fmt| self.debug_context_value(fmt, &**context))
                .finish(),
            SignatureValue::TakeFrom {
                literal,
                ref source,
            } => fmt
                .debug_struct("TakeSignatureFrom")
                .field_with("literal", |fmt| self.debug_signature_literal(fmt, literal))
                .field_with("source", |fmt| self.debug_function_value(fmt, &**source))
                .finish(),
            SignatureValue::GiveSignatureToSignature {
                ref signature,
                literal,
                ref source,
            } => fmt
                .debug_struct("GiveSignatureToSignature")
                .field_with("signature", |fmt| self.debug_signature_value(fmt, &**signature))
                .field_with("literal", |fmt| self.debug_signature_literal(fmt, literal))
                .field_with("source", |fmt| self.debug_signature_value(fmt, &**source))
                .finish(),
            SignatureValue::GiveFunctionToSignature {
                ref function,
                literal,
                ref source,
            } => fmt
                .debug_struct("GiveFunctionToSignature")
                .field_with("function", |fmt| self.debug_function_value(fmt, &**function))
                .field_with("literal", |fmt| self.debug_function_literal(fmt, literal))
                .field_with("source", |fmt| self.debug_signature_value(fmt, &**source))
                .finish(),
        }
    }

    fn debug_function_value(&self, fmt: &mut Formatter, value: &FunctionValue) -> fmt::Result {
        match *value {
            FunctionValue::Take {
                ref signature,
                literal,
            } => fmt
                .debug_struct("TakeFunction")
                .field_with("signature", |fmt| self.debug_signature_value(fmt, &*signature))
                .field_with("literal", |fmt| self.debug_function_literal(fmt, literal))
                .finish(),
            FunctionValue::Conjure {
                marker,
                ref signature,
            } => fmt
                .debug_struct("ConjureFunction")
                .field_with("marker", |fmt| {
                    self.debug_conjure_function_marker(fmt, marker)
                })
                .field_with("signature", |fmt| self.debug_signature_value(fmt, &*signature))
                .finish(),
            FunctionValue::Define { ref context } => fmt
                .debug_struct("DefineFunction")
                .field_with("context", |fmt| self.debug_context_value(fmt, &**context))
                .finish(),
            FunctionValue::TakeFrom {
                literal,
                ref source,
            } => fmt
                .debug_struct("TakeFunctionFrom")
                .field_with("literal", |fmt| self.debug_function_literal(fmt, literal))
                .field_with("source", |fmt| self.debug_function_value(fmt, &**source))
                .finish(),
            FunctionValue::GiveSignatureToFunction {
                ref signature,
                literal,
                ref source,
            } => fmt
                .debug_struct("GiveSignatureToFunction")
                .field_with("signature", |fmt| self.debug_signature_value(fmt, &**signature))
                .field_with("literal", |fmt| self.debug_signature_literal(fmt, literal))
                .field_with("source", |fmt| self.debug_function_value(fmt, &**source))
                .finish(),
            FunctionValue::GiveFunctionToFunction {
                ref function,
                literal,
                ref source,
            } => fmt
                .debug_struct("GiveFunctionToFunction")
                .field_with("function", |fmt| self.debug_function_value(fmt, &**function))
                .field_with("literal", |fmt| self.debug_function_literal(fmt, literal))
                .field_with("source", |fmt| self.debug_function_value(fmt, &**source))
                .finish(),
        }
    }

    fn debug_context_value(&self, fmt: &mut Formatter, value: &ContextValue) -> fmt::Result {
        fmt.debug_struct("ContextValue")
            .field_with("taken_signatures", |fmt| {
                let mut fmt = fmt.debug_list();
                for &literal in &value.taken_signatures {
                    fmt.entry_with(|fmt| self.debug_signature_literal(fmt, literal));
                }
                fmt.finish()
            })
            .field_with("taken_functions", |fmt| {
                let mut fmt = fmt.debug_list();
                for &literal in &value.taken_functions {
                    fmt.entry_with(|fmt| self.debug_function_literal(fmt, literal));
                }
                fmt.finish()
            })
            .field_with("given_signatures", |fmt| {
                let mut fmt = fmt.debug_map();
                for &(literal, ref value) in &value.given_signatures {
                    fmt.key_with(|fmt| self.debug_signature_literal(fmt, literal));
                    fmt.value_with(|fmt| self.debug_signature_value(fmt, value));
                }
                fmt.finish()
            })
            .field_with("given_functions", |fmt| {
                let mut fmt = fmt.debug_map();
                for &(literal, ref value) in &value.given_functions {
                    fmt.key_with(|fmt| self.debug_function_literal(fmt, literal));
                    fmt.value_with(|fmt| self.debug_function_value(fmt, value));
                }
                fmt.finish()
            })
            .finish()
    }
}
