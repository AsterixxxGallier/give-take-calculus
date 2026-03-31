use crate::parse;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::fmt::{Debug, Display};

mod id;

macro_rules! id {
    ($name:ident) => {
        #[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
        pub(crate) struct $name(id::Id);

        impl $name {
            pub(crate) fn generate() -> Self {
                Self(id::Id::generate())
            }
        }
    };
}

id!(ContextId);
id!(SignatureId);
id!(FunctionId);
id!(SignatureLiteralId);
id!(FunctionLiteralId);

pub(crate) struct SignatureDefineDependency {
    signature: SignatureId,
    literal: SignatureLiteralId,
}

pub(crate) struct FunctionDefineDependency {
    function: FunctionId,
    literal: FunctionLiteralId,
}

pub(crate) enum SignatureAssignmentRhs {
    Use {
        literal: SignatureLiteralId,
    },
    Take {
        literal: SignatureLiteralId,
    },
    Conjure {
        signature_dependencies: Vec<SignatureId>,
        function_dependencies: Vec<FunctionId>,
    },
    Define {
        signature_dependencies: Vec<SignatureDefineDependency>,
        function_dependencies: Vec<FunctionDefineDependency>,
        context: ContextId,
    },
    TakeFrom {
        literal: SignatureLiteralId,
        source: FunctionId,
    },
}

pub(crate) enum FunctionAssignmentRhs {
    Use {
        signature: SignatureId,
        literal: FunctionLiteralId,
    },
    Take {
        signature: SignatureId,
        literal: FunctionLiteralId,
    },
    Conjure {
        signature: SignatureId,
        signature_dependencies: Vec<SignatureId>,
        function_dependencies: Vec<FunctionId>,
    },
    Define {
        signature: SignatureId,
        signature_dependencies: Vec<SignatureDefineDependency>,
        function_dependencies: Vec<FunctionDefineDependency>,
        context: ContextId,
    },
    TakeFrom {
        literal: FunctionLiteralId,
        source: FunctionId,
    },
    GiveSignatureTo {
        signature: SignatureId,
        literal: SignatureLiteralId,
        source: FunctionId,
    },
    GiveFunctionTo {
        function: FunctionId,
        literal: FunctionLiteralId,
        source: FunctionId,
    },
}

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

pub(crate) struct ContextContents<'s> {
    statements: Vec<Statement>,
    name_to_signature: HashMap<&'s str, SignatureId>,
    name_to_function: HashMap<&'s str, FunctionId>,
}

#[derive(Copy, Clone)]
pub(crate) enum ContextLocation {
    // the global context
    Global,
    DefineSignature(SignatureId),
    DefineFunction(FunctionId),
}

trait ResolveNames<'s> {
    fn resolve_signature(&self, name: &'s str) -> Option<SignatureId>;

    fn resolve_function(&self, name: &'s str) -> Option<FunctionId>;
}

impl<'s> ResolveNames<'s> for ContextContents<'s> {
    fn resolve_signature(&self, name: &'s str) -> Option<SignatureId> {
        self.name_to_signature.get(name).copied()
    }

    fn resolve_function(&self, name: &'s str) -> Option<FunctionId> {
        self.name_to_function.get(name).copied()
    }
}

fn resolve_conjure_dependencies<'s>(
    resolver: &impl ResolveNames<'s>,
    conjure_dependencies: parse::ConjureDependencies<'s>,
) -> (Vec<SignatureId>, Vec<FunctionId>) {
    let (signatures, functions): (Vec<_>, Vec<_>) =
        conjure_dependencies
            .0
            .into_iter()
            .partition(|conjure_dependency| {
                matches!(conjure_dependency, parse::ConjureDependency::Signature(_))
            });
    let signatures = signatures
        .into_iter()
        .map(|conjure_dependency| {
            let parse::ConjureDependency::Signature(signature) = conjure_dependency else {
                unreachable!()
            };
            resolver.resolve_signature(signature.0).unwrap()
        })
        .collect();
    let functions = functions
        .into_iter()
        .map(|conjure_dependency| {
            let parse::ConjureDependency::Function(function) = conjure_dependency else {
                unreachable!()
            };
            resolver.resolve_function(function.0).unwrap()
        })
        .collect();
    (signatures, functions)
}

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
    // children first
    contexts: Vec<ContextId>,
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
            contexts: Default::default(),
        };
        this.build_global_context(global_context);
        this
    }

    fn build_global_context(&mut self, parsed: parse::Context<'s>) {
        self.context_locations
            .insert(self.global_context, ContextLocation::Global);
        self.build_context(self.global_context, parsed);
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

    fn resolve_define_dependencies(
        &mut self,
        resolver: &impl ResolveNames<'s>,
        define_dependencies: parse::DefineDependencies<'s>,
    ) -> (
        Vec<SignatureDefineDependency>,
        Vec<FunctionDefineDependency>,
    ) {
        let (signatures, functions): (Vec<_>, Vec<_>) = define_dependencies
            .0
            .into_iter()
            .partition(|define_dependency| {
                matches!(define_dependency, parse::DefineDependency::Signature(_))
            });
        let signatures = signatures
            .into_iter()
            .map(|define_dependency| {
                let parse::DefineDependency::Signature(signature) = define_dependency else {
                    unreachable!()
                };
                SignatureDefineDependency {
                    signature: resolver.resolve_signature(signature.signature.0).unwrap(),
                    literal: self.signature_literal(signature.literal.0),
                }
            })
            .collect();
        let functions = functions
            .into_iter()
            .map(|define_dependency| {
                let parse::DefineDependency::Function(function) = define_dependency else {
                    unreachable!()
                };
                FunctionDefineDependency {
                    function: resolver.resolve_function(function.function.0).unwrap(),
                    literal: self.function_literal(function.literal.0),
                }
            })
            .collect();
        (signatures, functions)
    }

    fn build_context(&mut self, context: ContextId, parsed: parse::Context<'s>) {
        let mut contents = ContextContents {
            statements: Vec::new(),
            name_to_signature: Default::default(),
            name_to_function: Default::default(),
        };

        for statement in parsed.0 {
            match statement {
                parse::Statement::SignatureAssignment(signature_assignment) => {
                    let rhs = match signature_assignment.rhs {
                        parse::SignatureAssignmentRhs::Use(use_signature) => {
                            let literal = self.signature_literal(use_signature.literal.0);
                            SignatureAssignmentRhs::Use { literal }
                        }
                        parse::SignatureAssignmentRhs::Take(take_signature) => {
                            let literal = self.signature_literal(take_signature.literal.0);
                            SignatureAssignmentRhs::Take { literal }
                        }
                        parse::SignatureAssignmentRhs::Conjure(conjure_signature) => {
                            let (signature_dependencies, function_dependencies) =
                                resolve_conjure_dependencies(
                                    &contents,
                                    conjure_signature.dependencies,
                                );
                            SignatureAssignmentRhs::Conjure {
                                signature_dependencies,
                                function_dependencies,
                            }
                        }
                        parse::SignatureAssignmentRhs::Define(define_signature) => {
                            let (signature_dependencies, function_dependencies) = self
                                .resolve_define_dependencies(
                                    &contents,
                                    define_signature.dependencies,
                                );
                            let define_context = ContextId::generate();
                            self.build_context(define_context, define_signature.context);
                            SignatureAssignmentRhs::Define {
                                signature_dependencies,
                                function_dependencies,
                                context: define_context,
                            }
                        }
                        parse::SignatureAssignmentRhs::TakeFrom(take_signature_from) => {
                            let source = contents
                                .resolve_function(take_signature_from.source.0)
                                .unwrap();
                            let literal = self.signature_literal(take_signature_from.literal.0);
                            SignatureAssignmentRhs::TakeFrom { literal, source }
                        }
                    };

                    let lhs = SignatureId::generate();
                    let lhs_name = signature_assignment.lhs.0;
                    contents.name_to_signature.insert(lhs_name, lhs);
                    contents
                        .statements
                        .push(Statement::SignatureAssignment { lhs, rhs });
                    self.signature_names.insert(lhs, lhs_name);
                    self.signature_locations.insert(lhs, context);
                }
                parse::Statement::FunctionAssignment(function_assignment) => {
                    let rhs = match function_assignment.rhs {
                        parse::FunctionAssignmentRhs::Use(use_function) => {
                            let signature = contents
                                .resolve_signature(use_function.signature.0)
                                .unwrap();
                            let literal = self.function_literal(use_function.literal.0);
                            FunctionAssignmentRhs::Use { signature, literal }
                        }
                        parse::FunctionAssignmentRhs::Take(take_function) => {
                            let signature = contents
                                .resolve_signature(take_function.signature.0)
                                .unwrap();
                            let literal = self.function_literal(take_function.literal.0);
                            FunctionAssignmentRhs::Take { signature, literal }
                        }
                        parse::FunctionAssignmentRhs::Conjure(conjure_function) => {
                            let signature = contents
                                .resolve_signature(conjure_function.signature.0)
                                .unwrap();
                            let (signature_dependencies, function_dependencies) =
                                resolve_conjure_dependencies(
                                    &contents,
                                    conjure_function.dependencies,
                                );
                            FunctionAssignmentRhs::Conjure {
                                signature,
                                signature_dependencies,
                                function_dependencies,
                            }
                        }
                        parse::FunctionAssignmentRhs::Define(define_function) => {
                            let signature = contents
                                .resolve_signature(define_function.signature.0)
                                .unwrap();
                            let (signature_dependencies, function_dependencies) = self
                                .resolve_define_dependencies(
                                    &contents,
                                    define_function.dependencies,
                                );
                            let define_context = ContextId::generate();
                            self.build_context(define_context, define_function.context);
                            FunctionAssignmentRhs::Define {
                                signature,
                                signature_dependencies,
                                function_dependencies,
                                context: define_context,
                            }
                        }
                        parse::FunctionAssignmentRhs::TakeFrom(take_function_from) => {
                            let source = contents
                                .resolve_function(take_function_from.source.0)
                                .unwrap();
                            let literal = self.function_literal(take_function_from.literal.0);
                            FunctionAssignmentRhs::TakeFrom { literal, source }
                        }
                        parse::FunctionAssignmentRhs::GiveSignatureTo(give_signature_to) => {
                            let signature = contents
                                .resolve_signature(give_signature_to.signature.0)
                                .unwrap();
                            let literal = self.signature_literal(give_signature_to.literal.0);
                            let source = contents
                                .resolve_function(give_signature_to.source.0)
                                .unwrap();
                            FunctionAssignmentRhs::GiveSignatureTo {
                                signature,
                                literal,
                                source,
                            }
                        }
                        parse::FunctionAssignmentRhs::GiveFunctionTo(give_function_to) => {
                            let function = contents
                                .resolve_function(give_function_to.function.0)
                                .unwrap();
                            let literal = self.function_literal(give_function_to.literal.0);
                            let source = contents
                                .resolve_function(give_function_to.source.0)
                                .unwrap();
                            FunctionAssignmentRhs::GiveFunctionTo {
                                function,
                                literal,
                                source,
                            }
                        }
                    };

                    let lhs = FunctionId::generate();
                    let lhs_name = function_assignment.lhs.0;
                    contents.name_to_function.insert(lhs_name, lhs);
                    contents
                        .statements
                        .push(Statement::FunctionAssignment { lhs, rhs });
                    self.function_names.insert(lhs, lhs_name);
                    self.function_locations.insert(lhs, context);
                }
                parse::Statement::GiveSignature(give_signature) => {
                    let literal = self.signature_literal(give_signature.literal.0);
                    let signature = contents
                        .resolve_signature(give_signature.signature.0)
                        .unwrap();
                    contents
                        .statements
                        .push(Statement::GiveSignature { signature, literal });
                }
                parse::Statement::GiveFunction(give_function) => {
                    let literal = self.function_literal(give_function.literal.0);
                    let function = contents.resolve_function(give_function.function.0).unwrap();
                    contents
                        .statements
                        .push(Statement::GiveFunction { function, literal });
                }
            }
        }

        self.context_contents.insert(context, contents);
        self.contexts.push(context);
    }
}
