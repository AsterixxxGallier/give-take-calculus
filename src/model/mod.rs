use crate::parse;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::fmt::Debug;

// mod check;
mod id;

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

id!(ContextId);
id!(SignatureId);
id!(FunctionId);
id!(SignatureLiteralId);
id!(FunctionLiteralId);

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub(crate) enum ConjureDependency {
    Signature(SignatureId),
    Function(FunctionId),
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) enum SignatureAssignmentRhs {
    Take {
        literal: SignatureLiteralId,
    },
    Conjure {
        dependencies: Vec<ConjureDependency>,
    },
    Define {
        context: ContextId,
    },
    TakeFrom {
        literal: SignatureLiteralId,
        source: FunctionId,
    },
    GiveSignatureToSignature {
        signature: SignatureId,
        literal: SignatureLiteralId,
        source: SignatureId,
    },
    GiveFunctionToSignature {
        function: FunctionId,
        literal: FunctionLiteralId,
        source: SignatureId,
    },
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) enum FunctionAssignmentRhs {
    Take {
        signature: SignatureId,
        literal: FunctionLiteralId,
    },
    Conjure {
        signature: SignatureId,
        dependencies: Vec<ConjureDependency>,
    },
    Define {
        context: ContextId,
    },
    TakeFrom {
        literal: FunctionLiteralId,
        source: FunctionId,
    },
    GiveSignatureToFunction {
        signature: SignatureId,
        literal: SignatureLiteralId,
        source: FunctionId,
    },
    GiveFunctionToFunction {
        function: FunctionId,
        literal: FunctionLiteralId,
        source: FunctionId,
    },
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) enum Statement {
    SignatureAssignment {
        lhs: SignatureId,
        rhs: SignatureAssignmentRhs,
    },
    FunctionAssignment {
        lhs: FunctionId,
        rhs: FunctionAssignmentRhs,
    },
    GiveSignature {
        signature: SignatureId,
        literal: SignatureLiteralId,
    },
    GiveFunction {
        function: FunctionId,
        literal: FunctionLiteralId,
    },
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct ContextContents<'s> {
    statements: Vec<Statement>,
    name_to_signature: HashMap<&'s str, SignatureId>,
    name_to_function: HashMap<&'s str, FunctionId>,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub(crate) enum ContextLocation {
    // the global context
    Global,
    DefineSignature(SignatureId),
    DefineFunction(FunctionId),
}

trait ResolveNames<'s> {
    fn resolve_signature(&self, name: &'s str) -> Option<SignatureId>;

    fn resolve_function(&self, name: &'s str) -> Option<FunctionId>;

    fn resolve_signature_unwrap(&self, name: &'s str) -> SignatureId {
        self.resolve_signature(name)
            .expect(&format!("could not find signature {name}"))
    }

    fn resolve_function_unwrap(&self, name: &'s str) -> FunctionId {
        self.resolve_function(name)
            .expect(&format!("could not find function {name}"))
    }
}

impl<'s> ResolveNames<'s> for ContextContents<'s> {
    fn resolve_signature(&self, name: &'s str) -> Option<SignatureId> {
        self.name_to_signature.get(name).copied()
    }

    fn resolve_function(&self, name: &'s str) -> Option<FunctionId> {
        self.name_to_function.get(name).copied()
    }
}

struct DeepResolver<'s, 'm: 's, 'p> {
    model: &'m Model<'s>,
    path: &'p Vec<ContextId>,
}

impl<'s, 'm: 's, 'p> ResolveNames<'s> for DeepResolver<'s, 'm, 'p> {
    fn resolve_signature(&self, name: &'s str) -> Option<SignatureId> {
        self.path
            .iter()
            .filter_map(|&context| {
                let contents = &self.model.context_contents[&context];
                contents.resolve_signature(name)
            })
            .rev()
            .next()
    }

    fn resolve_function(&self, name: &'s str) -> Option<FunctionId> {
        self.path
            .iter()
            .filter_map(|&context| {
                let contents = &self.model.context_contents[&context];
                contents.resolve_function(name)
            })
            .rev()
            .next()
    }
}

fn resolve_conjure_dependencies<'s>(
    resolver: &impl ResolveNames<'s>,
    dependencies: parse::ConjureDependencies<'s>,
) -> Vec<ConjureDependency> {
    dependencies
        .0
        .into_iter()
        .map(|dependency| match dependency {
            parse::ConjureDependency::Signature(signature) => {
                ConjureDependency::Signature(resolver.resolve_signature_unwrap(signature.as_str()))
            }
            parse::ConjureDependency::Function(function) => {
                ConjureDependency::Function(resolver.resolve_function_unwrap(function.as_str()))
            }
        })
        .collect()
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct Model<'s> {
    signature_literal_names: HashMap<SignatureLiteralId, &'s str>,
    function_literal_names: HashMap<FunctionLiteralId, &'s str>,
    name_to_signature_literal: HashMap<&'s str, SignatureLiteralId>,
    name_to_function_literal: HashMap<&'s str, FunctionLiteralId>,
    signature_names: HashMap<SignatureId, &'s str>,
    function_names: HashMap<FunctionId, &'s str>,
    context_contents: HashMap<ContextId, ContextContents<'s>>,
    context_locations: HashMap<ContextId, ContextLocation>,
    signature_locations: HashMap<SignatureId, ContextId>,
    function_locations: HashMap<FunctionId, ContextId>,
    global_context: ContextId,
}

impl<'s> Model<'s> {
    pub(crate) fn build(global_context: parse::Context<'s>) -> Self {
        let global_context_id = ContextId::generate();
        let mut this = Self {
            signature_literal_names: Default::default(),
            function_literal_names: Default::default(),
            name_to_signature_literal: Default::default(),
            name_to_function_literal: Default::default(),
            signature_names: Default::default(),
            function_names: Default::default(),
            context_contents: Default::default(),
            context_locations: Default::default(),
            signature_locations: Default::default(),
            function_locations: Default::default(),
            global_context: global_context_id,
        };
        this.build_global_context(global_context);
        this
    }

    fn build_global_context(&mut self, parsed: parse::Context<'s>) {
        self.context_locations
            .insert(self.global_context, ContextLocation::Global);
        self.build_context(self.global_context, parsed, &mut Vec::new());
    }

    fn signature_literal(&mut self, name: &'s str) -> SignatureLiteralId {
        match self.name_to_signature_literal.entry(name) {
            Entry::Occupied(entry) => *entry.get(),
            Entry::Vacant(entry) => {
                let id = SignatureLiteralId::generate();
                entry.insert(id);
                self.signature_literal_names.insert(id, name);
                id
            }
        }
    }

    fn function_literal(&mut self, name: &'s str) -> FunctionLiteralId {
        match self.name_to_function_literal.entry(name) {
            Entry::Occupied(entry) => *entry.get(),
            Entry::Vacant(entry) => {
                let id = FunctionLiteralId::generate();
                entry.insert(id);
                self.function_literal_names.insert(id, name);
                id
            }
        }
    }

    fn deep_resolver<'p>(&self, path: &'p Vec<ContextId>) -> DeepResolver<'s, '_, 'p> {
        DeepResolver { model: self, path }
    }

    fn build_context(
        &mut self,
        context: ContextId,
        parsed: parse::Context<'s>,
        path: &mut Vec<ContextId>,
    ) {
        path.push(context);

        let contents = ContextContents {
            statements: Vec::new(),
            name_to_signature: Default::default(),
            name_to_function: Default::default(),
        };

        self.context_contents.insert(context, contents);

        for statement in parsed.0 {
            match statement {
                parse::Statement::SignatureAssignment(signature_assignment) => {
                    let lhs = SignatureId::generate();

                    let rhs = match signature_assignment.rhs {
                        parse::SignatureAssignmentRhs::Take(take_signature) => {
                            let literal = self.signature_literal(take_signature.foreign.as_str());
                            SignatureAssignmentRhs::Take { literal }
                        }
                        parse::SignatureAssignmentRhs::Conjure(conjure_signature) => {
                            let contents = &self.context_contents[&context];
                            SignatureAssignmentRhs::Conjure {
                                dependencies: resolve_conjure_dependencies(
                                    contents,
                                    conjure_signature.dependencies,
                                ),
                            }
                        }
                        parse::SignatureAssignmentRhs::Define(define_signature) => {
                            let define_context = ContextId::generate();
                            self.context_locations
                                .insert(define_context, ContextLocation::DefineSignature(lhs));
                            self.build_context(define_context, define_signature.context, path);
                            SignatureAssignmentRhs::Define {
                                context: define_context,
                            }
                        }
                        parse::SignatureAssignmentRhs::TakeFrom(take_signature_from) => {
                            let source = self
                                .deep_resolver(path)
                                .resolve_function_unwrap(take_signature_from.source.as_str());
                            let literal =
                                self.signature_literal(take_signature_from.foreign.as_str());
                            SignatureAssignmentRhs::TakeFrom { literal, source }
                        }
                        parse::SignatureAssignmentRhs::GiveSignatureToSignature(
                            give_signature_to,
                        ) => {
                            let signature = self
                                .deep_resolver(path)
                                .resolve_signature_unwrap(give_signature_to.signature.as_str());
                            let literal =
                                self.signature_literal(give_signature_to.foreign.as_str());
                            let source = self
                                .deep_resolver(path)
                                .resolve_signature_unwrap(give_signature_to.source.as_str());
                            SignatureAssignmentRhs::GiveSignatureToSignature {
                                signature,
                                literal,
                                source,
                            }
                        }
                        parse::SignatureAssignmentRhs::GiveFunctionToSignature(
                            give_function_to,
                        ) => {
                            let function = self
                                .deep_resolver(path)
                                .resolve_function_unwrap(give_function_to.function.as_str());
                            let literal = self.function_literal(give_function_to.foreign.as_str());
                            let source = self
                                .deep_resolver(path)
                                .resolve_signature_unwrap(give_function_to.source.as_str());
                            SignatureAssignmentRhs::GiveFunctionToSignature {
                                function,
                                literal,
                                source,
                            }
                        }
                    };

                    let lhs_name = signature_assignment.lhs.as_str();
                    let contents = self.context_contents.get_mut(&context).unwrap();
                    contents.name_to_signature.insert(lhs_name, lhs);
                    contents
                        .statements
                        .push(Statement::SignatureAssignment { lhs, rhs });
                    self.signature_names.insert(lhs, lhs_name);
                    self.signature_locations.insert(lhs, context);
                }
                parse::Statement::FunctionAssignment(function_assignment) => {
                    let lhs = FunctionId::generate();

                    let rhs = match function_assignment.rhs {
                        parse::FunctionAssignmentRhs::Take(take_function) => {
                            let signature = self
                                .deep_resolver(path)
                                .resolve_signature_unwrap(take_function.signature.as_str());
                            let literal = self.function_literal(take_function.literal.as_str());
                            FunctionAssignmentRhs::Take { signature, literal }
                        }
                        parse::FunctionAssignmentRhs::Conjure(conjure_function) => {
                            let signature = self
                                .deep_resolver(path)
                                .resolve_signature_unwrap(conjure_function.signature.as_str());
                            let contents = &self.context_contents[&context];
                            FunctionAssignmentRhs::Conjure {
                                signature,
                                dependencies: resolve_conjure_dependencies(
                                    contents,
                                    conjure_function.dependencies,
                                ),
                            }
                        }
                        parse::FunctionAssignmentRhs::Define(define_function) => {
                            let define_context = ContextId::generate();
                            self.context_locations
                                .insert(define_context, ContextLocation::DefineFunction(lhs));
                            self.build_context(define_context, define_function.context, path);
                            FunctionAssignmentRhs::Define {
                                context: define_context,
                            }
                        }
                        parse::FunctionAssignmentRhs::TakeFrom(take_function_from) => {
                            let source = self
                                .deep_resolver(path)
                                .resolve_function_unwrap(take_function_from.source.as_str());
                            let literal =
                                self.function_literal(take_function_from.foreign.as_str());
                            FunctionAssignmentRhs::TakeFrom { literal, source }
                        }
                        parse::FunctionAssignmentRhs::GiveSignatureToFunction(
                            give_signature_to,
                        ) => {
                            let signature = self
                                .deep_resolver(path)
                                .resolve_signature_unwrap(give_signature_to.signature.as_str());
                            let literal =
                                self.signature_literal(give_signature_to.foreign.as_str());
                            let source = self
                                .deep_resolver(path)
                                .resolve_function_unwrap(give_signature_to.source.as_str());
                            FunctionAssignmentRhs::GiveSignatureToFunction {
                                signature,
                                literal,
                                source,
                            }
                        }
                        parse::FunctionAssignmentRhs::GiveFunctionToFunction(give_function_to) => {
                            let function = self
                                .deep_resolver(path)
                                .resolve_function_unwrap(give_function_to.function.as_str());
                            let literal = self.function_literal(give_function_to.foreign.as_str());
                            let source = self
                                .deep_resolver(path)
                                .resolve_function_unwrap(give_function_to.source.as_str());
                            FunctionAssignmentRhs::GiveFunctionToFunction {
                                function,
                                literal,
                                source,
                            }
                        }
                    };

                    let lhs_name = function_assignment.lhs.as_str();
                    let contents = self.context_contents.get_mut(&context).unwrap();
                    contents.name_to_function.insert(lhs_name, lhs);
                    contents
                        .statements
                        .push(Statement::FunctionAssignment { lhs, rhs });
                    self.function_names.insert(lhs, lhs_name);
                    self.function_locations.insert(lhs, context);
                }
                parse::Statement::GiveSignature(give_signature) => {
                    let literal = self.signature_literal(give_signature.literal.as_str());
                    let signature = self
                        .deep_resolver(path)
                        .resolve_signature_unwrap(give_signature.signature.as_str());
                    let contents = self.context_contents.get_mut(&context).unwrap();
                    contents
                        .statements
                        .push(Statement::GiveSignature { signature, literal });
                }
                parse::Statement::GiveFunction(give_function) => {
                    let literal = self.function_literal(give_function.literal.as_str());
                    let function = self
                        .deep_resolver(path)
                        .resolve_function_unwrap(give_function.function.as_str());
                    let contents = self.context_contents.get_mut(&context).unwrap();
                    contents
                        .statements
                        .push(Statement::GiveFunction { function, literal });
                }
            }
        }

        path.pop();
    }
}
