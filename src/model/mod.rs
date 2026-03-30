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

pub(crate) enum SignatureOrFunctionId {
    Signature(SignatureId),
    Function(FunctionId),
}

pub(crate) enum SignatureOrigin {
    Use {
        signature: SignatureId,
    },
    Take {
        literal: SignatureLiteralId,
    },
    Conjure {
        dependencies: Vec<SignatureOrFunctionId>,
    },
    Define {
        context: ContextId,
    },
    TakeFrom {
        literal: SignatureLiteralId,
        source: FunctionId,
    },
}

pub(crate) enum FunctionOrigin {
    Use {
        function: FunctionId,
    },
    Take {
        signature: SignatureId,
        literal: FunctionLiteralId,
    },
    Conjure {
        signature: SignatureId,
        dependencies: Vec<SignatureOrFunctionId>,
    },
    Define {
        signature: SignatureId,
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
        signature: SignatureId,
    },
    FunctionAssignment {
        function: FunctionId,
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
) -> Vec<SignatureOrFunctionId> {
    conjure_dependencies
        .0
        .into_iter()
        .map(|dependency| match dependency {
            parse::SignatureOrFunction::Signature(signature) => {
                SignatureOrFunctionId::Signature(resolver.resolve_signature(signature.0).unwrap())
            }
            parse::SignatureOrFunction::Function(function) => {
                SignatureOrFunctionId::Function(resolver.resolve_function(function.0).unwrap())
            }
        })
        .collect()
}

pub(crate) struct Model<'s> {
    signature_literal_names: HashMap<SignatureLiteralId, &'s str>,
    function_literal_names: HashMap<FunctionLiteralId, &'s str>,
    name_to_signature_literal: HashMap<&'s str, SignatureLiteralId>,
    name_to_function_literal: HashMap<&'s str, FunctionLiteralId>,
    signature_names: HashMap<SignatureId, &'s str>,
    function_names: HashMap<FunctionId, &'s str>,
    signature_origins: HashMap<SignatureId, SignatureOrigin>,
    function_origins: HashMap<FunctionId, FunctionOrigin>,
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
            signature_origins: Default::default(),
            function_origins: Default::default(),
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
        self.build_context(self.global_context, parsed, None);
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

    fn build_context(
        &mut self,
        context: ContextId,
        parsed: parse::Context<'s>,
        parent: Option<ContextId>,
    ) {
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
                    let lhs_name = signature_assignment.lhs.0;
                    let contents = self.context_contents.get_mut(&context).unwrap();
                    contents.name_to_signature.insert(lhs_name, lhs);
                    contents
                        .statements
                        .push(Statement::SignatureAssignment { signature: lhs });
                    self.signature_names.insert(lhs, lhs_name);
                    self.signature_locations.insert(lhs, context);

                    match signature_assignment.rhs {
                        parse::SignatureAssignmentRhs::Use(use_signature) => {
                            let parent = parent.expect(
                                "SignatureAssignmentRhs::use encountered in global context",
                            );
                            let signature = self.context_contents[&parent]
                                .resolve_signature(use_signature.signature.0)
                                .unwrap();
                            self.signature_origins
                                .insert(lhs, SignatureOrigin::Use { signature });
                        }
                        parse::SignatureAssignmentRhs::Take(take_signature) => {
                            let literal = self.signature_literal(take_signature.literal.0);
                            self.signature_origins
                                .insert(lhs, SignatureOrigin::Take { literal });
                        }
                        parse::SignatureAssignmentRhs::Conjure(conjure_signature) => {
                            self.signature_origins.insert(
                                lhs,
                                SignatureOrigin::Conjure {
                                    dependencies: resolve_conjure_dependencies(
                                        &self.context_contents[&context],
                                        conjure_signature.dependencies,
                                    ),
                                },
                            );
                        }
                        parse::SignatureAssignmentRhs::Define(define_signature) => {
                            let define_context = ContextId::generate();
                            self.build_context(
                                define_context,
                                define_signature.context,
                                Some(context),
                            );
                            self.signature_origins.insert(
                                lhs,
                                SignatureOrigin::Define {
                                    context: define_context,
                                },
                            );
                        }
                        parse::SignatureAssignmentRhs::TakeFrom(take_signature_from) => {
                            let source = self.context_contents[&context]
                                .resolve_function(take_signature_from.source.0)
                                .unwrap();
                            let literal = self.signature_literal(take_signature_from.literal.0);
                            self.signature_origins
                                .insert(lhs, SignatureOrigin::TakeFrom { literal, source });
                        }
                    }
                }
                parse::Statement::FunctionAssignment(function_assignment) => {
                    let lhs = FunctionId::generate();
                    let lhs_name = function_assignment.lhs.0;
                    let contents = self.context_contents.get_mut(&context).unwrap();
                    contents.name_to_function.insert(lhs_name, lhs);
                    contents
                        .statements
                        .push(Statement::FunctionAssignment { function: lhs });
                    self.function_names.insert(lhs, lhs_name);
                    self.function_locations.insert(lhs, context);
                    match function_assignment.rhs {
                        parse::FunctionAssignmentRhs::Use(use_function) => {
                            let parent = parent
                                .expect("FunctionAssignmentRhs::use encountered in global context");
                            let function = self.context_contents[&parent]
                                .resolve_function(use_function.function.0)
                                .unwrap();
                            self.function_origins
                                .insert(lhs, FunctionOrigin::Use { function });
                        }
                        parse::FunctionAssignmentRhs::Take(take_function) => {
                            let signature = self.context_contents[&context]
                                .resolve_signature(take_function.signature.0)
                                .unwrap();
                            let literal = self.function_literal(take_function.literal.0);
                            self.function_origins
                                .insert(lhs, FunctionOrigin::Take { signature, literal });
                        }
                        parse::FunctionAssignmentRhs::Conjure(conjure_function) => {
                            let signature = self.context_contents[&context]
                                .resolve_signature(conjure_function.signature.0)
                                .unwrap();
                            self.function_origins.insert(
                                lhs,
                                FunctionOrigin::Conjure {
                                    signature,
                                    dependencies: resolve_conjure_dependencies(
                                        &self.context_contents[&context],
                                        conjure_function.dependencies,
                                    ),
                                },
                            );
                        }
                        parse::FunctionAssignmentRhs::Define(define_function) => {
                            let signature = self.context_contents[&context]
                                .resolve_signature(define_function.signature.0)
                                .unwrap();
                            let define_context = ContextId::generate();
                            self.build_context(
                                define_context,
                                define_function.context,
                                Some(context),
                            );
                            self.function_origins.insert(
                                lhs,
                                FunctionOrigin::Define {
                                    signature,
                                    context: define_context,
                                },
                            );
                        }
                        parse::FunctionAssignmentRhs::TakeFrom(take_function_from) => {
                            let source = self.context_contents[&context]
                                .resolve_function(take_function_from.source.0)
                                .unwrap();
                            let literal = self.function_literal(take_function_from.literal.0);
                            self.function_origins
                                .insert(lhs, FunctionOrigin::TakeFrom { literal, source });
                        }
                        parse::FunctionAssignmentRhs::GiveSignatureTo(give_signature_to) => {
                            let signature = self.context_contents[&context]
                                .resolve_signature(give_signature_to.signature.0)
                                .unwrap();
                            let literal = self.signature_literal(give_signature_to.literal.0);
                            let source = self.context_contents[&context]
                                .resolve_function(give_signature_to.source.0)
                                .unwrap();
                            self.function_origins.insert(
                                lhs,
                                FunctionOrigin::GiveSignatureTo {
                                    signature,
                                    literal,
                                    source,
                                },
                            );
                        }
                        parse::FunctionAssignmentRhs::GiveFunctionTo(give_function_to) => {
                            let function = self.context_contents[&context]
                                .resolve_function(give_function_to.function.0)
                                .unwrap();
                            let literal = self.function_literal(give_function_to.literal.0);
                            let source = self.context_contents[&context]
                                .resolve_function(give_function_to.source.0)
                                .unwrap();
                            self.function_origins.insert(
                                lhs,
                                FunctionOrigin::GiveFunctionTo {
                                    function,
                                    literal,
                                    source,
                                },
                            );
                        }
                    }
                }
                parse::Statement::GiveSignature(give_signature) => {
                    let literal = self.signature_literal(give_signature.literal.0);
                    let contents = self.context_contents.get_mut(&context).unwrap();
                    let signature = contents
                        .resolve_signature(give_signature.signature.0)
                        .unwrap();
                    contents
                        .statements
                        .push(Statement::GiveSignature { signature, literal });
                }
                parse::Statement::GiveFunction(give_function) => {
                    let literal = self.function_literal(give_function.literal.0);
                    let contents = self.context_contents.get_mut(&context).unwrap();
                    let function = contents.resolve_function(give_function.function.0).unwrap();
                    contents
                        .statements
                        .push(Statement::GiveFunction { function, literal });
                }
            }
        }
    }

    pub(crate) fn check(&self) {
        self.check_context(self.global_context);
    }

    fn check_context(&self, context: ContextId) {
        id!(SignatureValueId);
        id!(FunctionValueId);

        pub(crate) enum SignatureOrFunctionValueId {
            Signature(SignatureValueId),
            Function(FunctionValueId),
        }

        enum SignatureValue {
            Take {
                id: SignatureId,
            },
            Conjure {
                id: SignatureId,
                dependencies: Vec<SignatureOrFunctionValueId>,
            },
            Define {
                dependencies: Vec<SignatureOrFunctionId>,
                taken_signatures: Vec<String>,
            },
        }

        let context_location = self.context_locations[&context];
        match context_location {
            ContextLocation::Global => {}
            ContextLocation::DefineSignature(_) => {}
            ContextLocation::DefineFunction(_) => {}
        }
    }
}
