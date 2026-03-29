#![allow(unused)]

use pest::Parser;
use pest::iterators::{Pair, Pairs};
use pest_derive::Parser;
use std::collections::HashMap;
use std::{fs, io};

type FunctionIndex = usize;
type SignatureIndex = usize;
type NodeIndex = usize;

#[derive(Debug)]
struct Function {
    text: String,
    origin_node: NodeIndex,
    origin: FunctionOrigin,
}

#[allow(non_camel_case_types)]
#[derive(Debug)]
enum FunctionOrigin {
    Take,
    Conjure,
    Prove,
    UnwrapGiving_InnerFunction,
    UnwrapGivingSignature_InnerFunction,
    UnwrapTaking_InnerFunction,
    UnwrapTaking_TakenFunction,
    UnwrapTakingSignature_InnerFunction,
}

#[derive(Debug)]
struct Signature {
    text: String,
    origin_node: NodeIndex,
    origin: SignatureOrigin,
}

#[allow(non_camel_case_types)]
#[derive(Debug)]
enum SignatureOrigin {
    TakeSignature,
    ConjureSignature,
    DefineSignature,
    UnwrapTakingSignature_TakenSignature,
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone)]
enum RelationToParentNode {
    TakeSignature_Inner,
    ConjureSignature_Inner,
    GiveSignature_Inner,
    DefineSignature_Inner,
    DefineSignature_Interior,
    Take_Inner,
    Conjure_Inner,
    Give_Inner,
    Prove_Inner,
    Prove_Interior,
    UnwrapGiving_Inner,
    UnwrapGivingSignature_Inner,
    UnwrapTaking_Inner,
    UnwrapTakingSignature_Inner,
}

#[derive(Debug, Clone)]
struct ParentInfo {
    parent_node: NodeIndex,
    relation_to_parent_node: RelationToParentNode,
}

#[derive(Debug, Clone)]
struct Context {
    parent_info: Option<ParentInfo>,
    available_functions: Vec<FunctionIndex>,
    available_signatures: Vec<SignatureIndex>,
}

#[derive(Debug)]
struct Node {
    context: Context,
    info: NodeInfo,
}

#[derive(Debug)]
enum NodeInfo {
    TakeSignature {
        signature: SignatureIndex,
        inner_node: NodeIndex,
    },
    ConjureSignature {
        signature: SignatureIndex,
        inner_node: NodeIndex,
    },
    GiveSignature {
        signature: SignatureIndex,
        inner_node: NodeIndex,
    },
    DefineSignature {
        signature: SignatureIndex,
        interior_node: NodeIndex,
        inner_node: NodeIndex,
    },
    Take {
        function: FunctionIndex,
        signature: SignatureIndex,
        inner_node: NodeIndex,
    },
    Conjure {
        function: FunctionIndex,
        signature: SignatureIndex,
        inner_node: NodeIndex,
    },
    Give {
        function: FunctionIndex,
        inner_node: NodeIndex,
    },
    Prove {
        function: FunctionIndex,
        signature: SignatureIndex,
        interior_node: NodeIndex,
        inner_node: NodeIndex,
    },
    UnwrapGiving {
        inner_function: FunctionIndex,
        outer_function: FunctionIndex,
        given_function: FunctionIndex,
        inner_node: NodeIndex,
    },
    UnwrapGivingSignature {
        inner_function: FunctionIndex,
        outer_function: FunctionIndex,
        given_signature: SignatureIndex,
        inner_node: NodeIndex,
    },
    UnwrapTaking {
        inner_function: FunctionIndex,
        outer_function: FunctionIndex,
        taken_function: FunctionIndex,
        inner_node: NodeIndex,
    },
    UnwrapTakingSignature {
        inner_function: FunctionIndex,
        outer_function: FunctionIndex,
        taken_signature: SignatureIndex,
        inner_node: NodeIndex,
    },
}

#[derive(Debug)]
struct File {
    nodes: Vec<Option<Node>>,
    functions: Vec<Function>,
    signatures: Vec<Signature>,
}

mod parse;

impl File {
    fn check(&self) {
        #[allow(non_camel_case_types)]
        #[derive(Debug, Clone)]
        enum FunctionValue {
            Take(FunctionIndex),
            Conjure(FunctionIndex),
            Prove(FunctionIndex),
            UnwrapGiving_InnerFunction {
                outer_function: Box<FunctionValue>,
                given_function: Box<FunctionValue>,
            },
            UnwrapGivingSignature_InnerFunction {
                outer_function: Box<FunctionValue>,
                given_signature: Box<SignatureValue>,
            },
            UnwrapTaking_InnerFunction {
                outer_function: Box<FunctionValue>,
            },
            UnwrapTaking_TakenFunction {
                outer_function: Box<FunctionValue>,
            },
            UnwrapTakingSignature_InnerFunction {
                outer_function: Box<FunctionValue>,
            },
        }

        #[allow(non_camel_case_types)]
        #[derive(Debug, Clone)]
        enum SignatureValue {
            TakeSignature(SignatureIndex),
            ConjureSignature(SignatureIndex),
            DefineSignature(SignatureIndex),
            UnwrapTakingSignature_TakenSignature { outer_function: Box<FunctionValue> },
        }

        struct State<'a> {
            file: &'a File,
            function_values: HashMap<FunctionIndex, FunctionValue>,
            signature_values: HashMap<SignatureIndex, SignatureValue>,
        }

        #[derive(Debug)]
        struct ProofState {
            node: NodeIndex,
            function_values: HashMap<FunctionIndex, FunctionValue>,
            signature_values: HashMap<SignatureIndex, SignatureValue>,
        };

        #[derive(Debug)]
        enum Environment {
            Global,
            Prove(ProofState),
            DefineSignature,
        }

        fn recurse(state: &mut State, node: NodeIndex, mut environment: Environment) {
            match state.file.nodes[node].as_ref().unwrap().info {
                NodeInfo::TakeSignature {
                    signature,
                    inner_node,
                } => {
                    let signature_value = SignatureValue::TakeSignature(signature);
                    if let Environment::Prove(proof_state) = &mut environment {
                        let node = proof_state.node;
                        if let NodeInfo::TakeSignature {
                            signature,
                            inner_node,
                        } = state.file.nodes[node].as_ref().unwrap().info
                        {
                            proof_state.node = inner_node;
                            proof_state
                                .signature_values
                                .insert(signature, signature_value.clone());
                        } else {
                            panic!(
                                "proof does not match signature definition: expected take signature in signature definition"
                            );
                        }
                        loop {
                            let node = proof_state.node;
                            match state.file.nodes[node].as_ref().unwrap().info {
                                NodeInfo::TakeSignature { .. } => break,
                                NodeInfo::ConjureSignature { signature, inner_node } => {
                                    proof_state.node = inner_node;
                                    continue;
                                }
                                NodeInfo::GiveSignature { .. } => break,
                                NodeInfo::DefineSignature { signature, interior_node, inner_node } => {
                                    proof_state.node = inner_node;
                                    continue;
                                }
                                NodeInfo::Take { .. } => {}
                                NodeInfo::Conjure { .. } => {}
                                NodeInfo::Give { .. } => {}
                                NodeInfo::Prove { .. } => {}
                                NodeInfo::UnwrapGiving { .. } => {}
                                NodeInfo::UnwrapGivingSignature { .. } => {}
                                NodeInfo::UnwrapTaking { .. } => {}
                                NodeInfo::UnwrapTakingSignature { .. } => {}
                            }
                        }
                    }
                    state.signature_values.insert(signature, signature_value);
                    recurse(state, inner_node, environment);
                    state.signature_values.remove(&signature);
                }
                NodeInfo::ConjureSignature {
                    signature,
                    inner_node,
                } => {
                    assert!(
                        matches!(environment, Environment::DefineSignature),
                        "cannot conjure outside of define signature environment"
                    );
                    state
                        .signature_values
                        .insert(signature, SignatureValue::ConjureSignature(signature));
                    recurse(state, inner_node, environment);
                    state.signature_values.remove(&signature);
                }
                NodeInfo::GiveSignature {
                    signature,
                    inner_node,
                } => {
                    recurse(state, inner_node, environment);
                }
                NodeInfo::DefineSignature {
                    signature,
                    interior_node,
                    inner_node,
                } => {
                    recurse(state, interior_node, Environment::DefineSignature);
                    state
                        .signature_values
                        .insert(signature, SignatureValue::DefineSignature(signature));
                    recurse(state, inner_node, environment);
                    state.signature_values.remove(&signature);
                }
                NodeInfo::Take {
                    function,
                    signature,
                    inner_node,
                } => {
                    state
                        .function_values
                        .insert(function, FunctionValue::Take(function));
                    recurse(state, inner_node, environment);
                    state.function_values.remove(&function);
                }
                NodeInfo::Conjure {
                    function,
                    signature,
                    inner_node,
                } => {
                    assert_eq!(
                        environment,
                        Environment::DefineSignature,
                        "cannot conjure outside of define signature environment"
                    );
                    state
                        .function_values
                        .insert(function, FunctionValue::Take(function));
                    recurse(state, inner_node, environment);
                    state.function_values.remove(&function);
                }
                NodeInfo::Give {
                    function,
                    inner_node,
                } => {
                    recurse(state, inner_node, environment);
                }
                NodeInfo::Prove {
                    function,
                    signature,
                    interior_node,
                    inner_node,
                } => {
                    let signature_origin_inner_node = {
                        let node = state.file.signatures[signature].origin_node;
                        let NodeInfo::DefineSignature {
                            signature,
                            interior_node,
                            inner_node,
                        } = state.file.nodes[node].as_ref().unwrap().info
                        else {
                            unreachable!()
                        };
                        interior_node
                    };
                    recurse(
                        state,
                        interior_node,
                        Environment::Prove(ProofState(signature_origin_inner_node)),
                    );
                    state
                        .function_values
                        .insert(function, FunctionValue::Prove(function));
                    recurse(state, inner_node, environment);
                    state.function_values.remove(&function);
                }
                NodeInfo::UnwrapGiving {
                    inner_function,
                    outer_function,
                    given_function,
                    inner_node,
                } => {
                    let outer_function_value =
                        state.function_values.get(&outer_function).unwrap().clone();
                    let given_function_value =
                        state.function_values.get(&given_function).unwrap().clone();
                    state.function_values.insert(
                        inner_function,
                        FunctionValue::UnwrapGiving_InnerFunction {
                            outer_function: Box::new(outer_function_value),
                            given_function: Box::new(given_function_value),
                        },
                    );
                    recurse(state, inner_node, environment);
                    state.function_values.remove(&inner_function);
                }
                NodeInfo::UnwrapGivingSignature {
                    inner_function,
                    outer_function,
                    given_signature,
                    inner_node,
                } => {
                    let outer_function_value =
                        state.function_values.get(&outer_function).unwrap().clone();
                    let given_signature_value = state
                        .signature_values
                        .get(&given_signature)
                        .unwrap()
                        .clone();
                    state.function_values.insert(
                        inner_function,
                        FunctionValue::UnwrapGivingSignature_InnerFunction {
                            outer_function: Box::new(outer_function_value),
                            given_signature: Box::new(given_signature_value),
                        },
                    );
                    recurse(state, inner_node, environment);
                    state.function_values.remove(&inner_function);
                }
                NodeInfo::UnwrapTaking {
                    inner_function,
                    outer_function,
                    taken_function,
                    inner_node,
                } => {
                    let outer_function_value =
                        state.function_values.get(&outer_function).unwrap().clone();
                    state.function_values.insert(
                        inner_function,
                        FunctionValue::UnwrapTaking_InnerFunction {
                            outer_function: Box::new(outer_function_value.clone()),
                        },
                    );
                    state.function_values.insert(
                        taken_function,
                        FunctionValue::UnwrapTaking_TakenFunction {
                            outer_function: Box::new(outer_function_value),
                        },
                    );
                    recurse(state, inner_node, environment);
                    state.function_values.remove(&inner_function);
                    state.function_values.remove(&taken_function);
                }
                NodeInfo::UnwrapTakingSignature {
                    inner_function,
                    outer_function,
                    taken_signature,
                    inner_node,
                } => {
                    let outer_function_value =
                        state.function_values.get(&outer_function).unwrap().clone();
                    state.function_values.insert(
                        inner_function,
                        FunctionValue::UnwrapTakingSignature_InnerFunction {
                            outer_function: Box::new(outer_function_value.clone()),
                        },
                    );
                    state.signature_values.insert(
                        taken_signature,
                        SignatureValue::UnwrapTakingSignature_TakenSignature {
                            outer_function: Box::new(outer_function_value),
                        },
                    );
                    recurse(state, inner_node, environment);
                    state.function_values.remove(&inner_function);
                    state.signature_values.remove(&taken_signature);
                }
            }
        }

        let mut state = State {
            file: self,
            function_values: HashMap::new(),
            signature_values: HashMap::new(),
        };

        recurse(&mut state, 0, Environment::Global);
    }
}

fn main() {
    let file = File::parse("resources/false.txt");
    println!("{file:#?}");
}
