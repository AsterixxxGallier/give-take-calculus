use crate::parse;
use crate::parse::{FunctionAssignmentRhs, SignatureAssignmentRhs, SignatureOrFunction, Statement};
use std::collections::{HashMap, HashSet};
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

pub(crate) enum SignatureOrFunctionId {
    Signature(SignatureId),
    Function(FunctionId),
}

pub(crate) enum SignatureOrigin {
    Take,
    Conjure {
        dependencies: Vec<SignatureOrFunctionId>,
    },
    Define {
        context: ContextId,
    },
    TakeFrom {
        remote: SignatureId,
        source: FunctionId,
    },
}

pub(crate) enum FunctionOrigin {
    Take,
    Conjure {
        dependencies: Vec<SignatureOrFunctionId>,
    },
    Define {
        context: ContextId,
    },
    TakeFrom {
        remote: FunctionId,
        source: FunctionId,
    },
    GiveSignatureTo {
        local: SignatureId,
        remote: SignatureId,
        source: FunctionId,
    },
    GiveFunctionTo {
        local: FunctionId,
        remote: FunctionId,
        source: FunctionId,
    },
}

pub(crate) struct ContextContents<'s> {
    signatures_and_functions: Vec<SignatureOrFunctionId>,
    public_signatures: HashSet<SignatureId>,
    public_functions: HashSet<FunctionId>,
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

struct DeepResolver<'s, 'i: 's, 'p> {
    interpreter: &'i Model<'s>,
    path: &'p Vec<ContextId>,
}

impl<'s, 'i: 's, 'p> ResolveNames<'s> for DeepResolver<'s, 'i, 'p> {
    fn resolve_signature(&self, name: &'s str) -> Option<SignatureId> {
        self.path
            .iter()
            .filter_map(|&context| {
                let contents = &self.interpreter.context_contents[&context];
                contents.resolve_signature(name)
            })
            .next()
    }

    fn resolve_function(&self, name: &'s str) -> Option<FunctionId> {
        self.path
            .iter()
            .filter_map(|&context| {
                let contents = &self.interpreter.context_contents[&context];
                contents.resolve_function(name)
            })
            .next()
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
            SignatureOrFunction::Signature(signature) => {
                SignatureOrFunctionId::Signature(resolver.resolve_signature(signature.0).unwrap())
            }
            SignatureOrFunction::Function(function) => {
                SignatureOrFunctionId::Function(resolver.resolve_function(function.0).unwrap())
            }
        })
        .collect()
}

pub(crate) struct Model<'s> {
    signature_names: HashMap<SignatureId, &'s str>,
    function_names: HashMap<FunctionId, &'s str>,
    signature_origins: HashMap<SignatureId, SignatureOrigin>,
    function_origins: HashMap<FunctionId, FunctionOrigin>,
    function_signatures: HashMap<FunctionId, SignatureId>,
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
            signature_names: Default::default(),
            function_names: Default::default(),
            signature_origins: Default::default(),
            function_origins: Default::default(),
            function_signatures: Default::default(),
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

    fn deep_resolver<'i, 'p>(&'i self, path: &'p Vec<ContextId>) -> DeepResolver<'s, 'i, 'p> {
        DeepResolver {
            interpreter: self,
            path,
        }
    }

    fn build_context(
        &mut self,
        context: ContextId,
        parsed: parse::Context<'s>,
        path: &mut Vec<ContextId>,
    ) {
        path.push(context);

        let contents = ContextContents {
            signatures_and_functions: Vec::new(),
            public_signatures: Default::default(),
            public_functions: Default::default(),
            name_to_signature: Default::default(),
            name_to_function: Default::default(),
        };
        self.context_contents.insert(context, contents);
        for statement in parsed.0 {
            match statement {
                Statement::SignatureAssignment(signature_assignment) => {
                    let lhs = SignatureId::generate();
                    let lhs_name = signature_assignment.lhs.0;
                    self.context_contents
                        .get_mut(&context)
                        .unwrap()
                        .name_to_signature
                        .insert(lhs_name, lhs);
                    self.signature_names.insert(lhs, lhs_name);
                    self.signature_locations.insert(lhs, context);
                    match signature_assignment.rhs {
                        SignatureAssignmentRhs::Take(take_signature) => {
                            self.signature_origins.insert(lhs, SignatureOrigin::Take);
                        }
                        SignatureAssignmentRhs::Conjure(conjure_signature) => {
                            self.signature_origins.insert(
                                lhs,
                                SignatureOrigin::Conjure {
                                    dependencies: resolve_conjure_dependencies(
                                        &self.deep_resolver(path),
                                        conjure_signature.dependencies,
                                    ),
                                },
                            );
                        }
                        SignatureAssignmentRhs::Define(define_signature) => {
                            let define_context = ContextId::generate();
                            self.build_context(define_context, define_signature.context, path);
                            self.signature_origins.insert(
                                lhs,
                                SignatureOrigin::Define {
                                    context: define_context,
                                },
                            );
                        }
                        SignatureAssignmentRhs::TakeFrom(take_signature_from) => {
                            let source = self
                                .deep_resolver(path)
                                .resolve_function(take_signature_from.source.0)
                                .unwrap();
                            let source_signature = self.function_signatures[&source];
                            let SignatureOrigin::Define {
                                context: source_signature_context,
                            } = self.signature_origins[&source_signature]
                            else {
                                panic!();
                            };
                            let source_signature_context_contents =
                                &self.context_contents[&source_signature_context];
                            let remote = source_signature_context_contents
                                .resolve_signature(take_signature_from.remote.0)
                                .unwrap();
                            assert!(
                                source_signature_context_contents
                                    .public_signatures
                                    .contains(&remote),
                            );
                            self.signature_origins
                                .insert(lhs, SignatureOrigin::TakeFrom { remote, source });
                        }
                    }
                }
                Statement::FunctionAssignment(function_assignment) => {
                    let lhs = FunctionId::generate();
                    let lhs_name = function_assignment.lhs.0;
                    self.context_contents
                        .get_mut(&context)
                        .unwrap()
                        .name_to_function
                        .insert(lhs_name, lhs);
                    self.function_names.insert(lhs, lhs_name);
                    self.function_locations.insert(lhs, context);
                    match function_assignment.rhs {
                        FunctionAssignmentRhs::Take(take_function) => {
                            let signature = self
                                .deep_resolver(path)
                                .resolve_signature(take_function.signature.0)
                                .unwrap();
                            self.function_signatures.insert(lhs, signature);
                            self.function_origins.insert(lhs, FunctionOrigin::Take);
                        }
                        FunctionAssignmentRhs::Conjure(conjure_function) => {
                            let signature = self
                                .deep_resolver(path)
                                .resolve_signature(conjure_function.signature.0)
                                .unwrap();
                            self.function_signatures.insert(lhs, signature);
                            self.function_origins.insert(
                                lhs,
                                FunctionOrigin::Conjure {
                                    dependencies: resolve_conjure_dependencies(
                                        &self.deep_resolver(path),
                                        conjure_function.dependencies,
                                    ),
                                },
                            );
                        }
                        FunctionAssignmentRhs::Define(define_function) => {
                            let signature = self
                                .deep_resolver(path)
                                .resolve_signature(define_function.signature.0)
                                .unwrap();
                            self.function_signatures.insert(lhs, signature);
                            let define_context = ContextId::generate();
                            self.build_context(define_context, define_function.context, path);
                            self.function_origins.insert(
                                lhs,
                                FunctionOrigin::Define {
                                    context: define_context,
                                },
                            );
                        }
                        FunctionAssignmentRhs::TakeFrom(take_function_from) => {
                            // lhs signature unknown at this stage
                            let source = self
                                .deep_resolver(path)
                                .resolve_function(take_function_from.source.0)
                                .unwrap();
                            let source_signature = self.function_signatures[&source];
                            let SignatureOrigin::Define {
                                context: source_signature_context,
                            } = self.signature_origins[&source_signature]
                            else {
                                panic!();
                            };
                            let source_signature_context_contents =
                                &self.context_contents[&source_signature_context];
                            let remote = source_signature_context_contents
                                .resolve_function(take_function_from.remote.0)
                                .unwrap();
                            assert!(
                                source_signature_context_contents
                                    .public_functions
                                    .contains(&remote),
                            );
                            self.function_origins
                                .insert(lhs, FunctionOrigin::TakeFrom { remote, source });
                        }
                        FunctionAssignmentRhs::GiveSignatureTo(give_signature_to) => {
                            // lhs signature unknown at this stage
                            let local = self
                                .deep_resolver(path)
                                .resolve_signature(give_signature_to.signature.0)
                                .unwrap();
                            let source = self
                                .deep_resolver(path)
                                .resolve_function(give_signature_to.source.0)
                                .unwrap();
                            let source_signature = self.function_signatures[&source];
                            let SignatureOrigin::Define {
                                context: source_signature_context,
                            } = self.signature_origins[&source_signature]
                            else {
                                panic!();
                            };
                            let source_signature_context_contents =
                                &self.context_contents[&source_signature_context];
                            let remote = source_signature_context_contents
                                .resolve_signature(give_signature_to.remote.0)
                                .unwrap();
                            assert!(
                                source_signature_context_contents
                                    .public_signatures
                                    .contains(&remote),
                            );
                            self.function_origins.insert(
                                lhs,
                                FunctionOrigin::GiveSignatureTo {
                                    local,
                                    remote,
                                    source,
                                },
                            );
                        }
                        FunctionAssignmentRhs::GiveFunctionTo(give_function_to) => {
                            // lhs signature unknown at this stage
                            let local = self
                                .deep_resolver(path)
                                .resolve_function(give_function_to.function.0)
                                .unwrap();
                            let source = self
                                .deep_resolver(path)
                                .resolve_function(give_function_to.source.0)
                                .unwrap();
                            let source_signature = self.function_signatures[&source];
                            let SignatureOrigin::Define {
                                context: source_signature_context,
                            } = self.signature_origins[&source_signature]
                            else {
                                panic!();
                            };
                            let source_signature_context_contents =
                                &self.context_contents[&source_signature_context];
                            let remote = source_signature_context_contents
                                .resolve_function(give_function_to.remote.0)
                                .unwrap();
                            assert!(
                                source_signature_context_contents
                                    .public_functions
                                    .contains(&remote),
                            );
                            self.function_origins.insert(
                                lhs,
                                FunctionOrigin::GiveFunctionTo {
                                    local,
                                    remote,
                                    source,
                                },
                            );
                        }
                    }
                }
                Statement::GiveSignature(signature) => {
                    let contents = self.context_contents.get_mut(&context).unwrap();
                    let signature = contents.resolve_signature(signature.signature.0).unwrap();
                    contents.public_signatures.insert(signature);
                }
                Statement::GiveFunction(function) => {
                    let contents = self.context_contents.get_mut(&context).unwrap();
                    let function = contents.resolve_function(function.function.0).unwrap();
                    contents.public_functions.insert(function);
                }
            }
        }

        path.pop();
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

            }
        }

        let context_location = self.context_locations[&context];
        match context_location {
            ContextLocation::Global => {

            }
            ContextLocation::DefineSignature(_) => {}
            ContextLocation::DefineFunction(_) => {}
        }
    }
}
