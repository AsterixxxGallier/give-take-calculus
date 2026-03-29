use std::collections::HashMap;
use std::fs;
use pest::iterators::{Pair, Pairs};
use pest::Parser;
use pest_derive::Parser;
use crate::{Context, File, Function, FunctionIndex, FunctionOrigin, Node, NodeIndex, NodeInfo, ParentInfo, RelationToParentNode, Signature, SignatureIndex, SignatureOrigin};

#[derive(Parser)]
#[grammar = "syntax.pest"]
struct MyParser;

impl File {
    pub(crate) fn parse(path: &str) -> Self {
        let text = fs::read_to_string(path).unwrap();
        let mut this = Self {
            nodes: vec![],
            functions: vec![],
            signatures: vec![],
        };
        #[derive(Copy, Clone)]
        enum Stage {
            First,
            Second,
            End,
        }
        #[derive(Clone)]
        struct StackFrame<'i> {
            context: Context,
            node_pair: Pair<'i, Rule>,
            reserved_node: NodeIndex,
            function_resolve_map: HashMap<String, FunctionIndex>,
            signature_resolve_map: HashMap<String, SignatureIndex>,
        }
        fn define_function(
            this: &mut File,
            origin_node: NodeIndex,
            origin: FunctionOrigin,
            pair: Pair<Rule>,
        ) -> FunctionIndex {
            let text = pair.as_str().to_owned();
            let function = this.functions.len();
            this.functions.push(Function {
                text: text.clone(),
                origin_node,
                origin,
            });
            function
        }
        fn define_signature(
            this: &mut File,
            origin_node: NodeIndex,
            origin: SignatureOrigin,
            pair: Pair<Rule>,
        ) -> SignatureIndex {
            let text = pair.as_str().to_owned();
            let signature = this.signatures.len();
            this.signatures.push(Signature {
                text: text.clone(),
                origin_node,
                origin,
            });
            signature
        }
        fn resolve_function(
            this: &mut File,
            stack_frame: &StackFrame,
            pair: Pair<Rule>,
        ) -> FunctionIndex {
            let text = pair.as_str().to_owned();
            if let Some(&function) = stack_frame.function_resolve_map.get(&text) {
                function
            } else {
                panic!("function {text} not found in scope");
            }
        }
        fn resolve_signature(
            this: &mut File,
            stack_frame: &StackFrame,
            pair: Pair<Rule>,
        ) -> SignatureIndex {
            let text = pair.as_str().to_owned();
            if let Some(&signature) = stack_frame.signature_resolve_map.get(&text) {
                signature
            } else {
                panic!("signature {text} not found in scope");
            }
        }
        fn find_inner_node(mut pairs: Pairs<Rule>) -> Option<Pair<Rule>> {
            pairs
                .find(|pair| pair.as_rule() == Rule::inner_node)
                .map(|pair| pair.into_inner().next().unwrap())
        }
        fn find_interior_node(mut pairs: Pairs<Rule>) -> Option<Pair<Rule>> {
            pairs
                .find(|pair| pair.as_rule() == Rule::interior_node)
                .map(|pair| pair.into_inner().next().unwrap())
        }
        fn create_stack_frame<'i>(
            this: &File,
            node_pair: Pair<'i, Rule>,
            reserved_node: NodeIndex,
            parent: &StackFrame<'i>,
            parent_node: NodeIndex,
            relation_to_parent_node: RelationToParentNode,
            new_functions: Vec<FunctionIndex>,
            new_signatures: Vec<SignatureIndex>,
        ) -> StackFrame<'i> {
            let available_functions = parent
                .context
                .available_functions
                .iter()
                .chain(new_functions.iter())
                .copied()
                .collect();
            let function_resolve_map = parent
                .function_resolve_map
                .clone()
                .into_iter()
                .chain(
                    new_functions
                        .into_iter()
                        .map(|function| (this.functions[function].text.clone(), function)),
                )
                .collect();
            let available_signatures = parent
                .context
                .available_signatures
                .iter()
                .chain(new_signatures.iter())
                .copied()
                .collect();
            let signature_resolve_map = parent
                .signature_resolve_map
                .clone()
                .into_iter()
                .chain(
                    new_signatures
                        .into_iter()
                        .map(|signature| (this.signatures[signature].text.clone(), signature)),
                )
                .collect();
            StackFrame {
                context: Context {
                    parent_info: Some(ParentInfo {
                        parent_node,
                        relation_to_parent_node,
                    }),
                    available_functions,
                    available_signatures,
                },
                node_pair,
                reserved_node,
                function_resolve_map,
                signature_resolve_map,
            }
        }
        fn reserve_node(this: &mut File) -> NodeIndex {
            let node = this.nodes.len();
            this.nodes.push(None);
            node
        }
        fn recurse(this: &mut File, stack_frame: StackFrame) {
            match stack_frame.node_pair.as_rule() {
                Rule::take_signature => {
                    let mut pairs = stack_frame.node_pair.clone().into_inner();
                    let signature_pair = pairs.next().unwrap();

                    let node = stack_frame.reserved_node;
                    let inner_node = reserve_node(this);

                    let signature =
                        define_signature(this, node, SignatureOrigin::TakeSignature, signature_pair);

                    this.nodes[node] = Some(Node {
                        context: stack_frame.context.clone(),
                        info: NodeInfo::TakeSignature { signature, inner_node },
                    });

                    if let Some(inner_node_pair) = find_inner_node(pairs.clone()) {
                        let inner_stack_frame = create_stack_frame(
                            this,
                            inner_node_pair,
                            inner_node,
                            &stack_frame,
                            node,
                            RelationToParentNode::TakeSignature_Inner,
                            vec![],
                            vec![signature],
                        );
                        recurse(this, inner_stack_frame);
                    }
                }
                Rule::conjure_signature => {
                    let mut pairs = stack_frame.node_pair.clone().into_inner();
                    let signature_pair = pairs.next().unwrap();

                    let node = stack_frame.reserved_node;
                    let inner_node = reserve_node(this);

                    let signature =
                        define_signature(this, node, SignatureOrigin::ConjureSignature, signature_pair);

                    this.nodes[node] = Some(Node {
                        context: stack_frame.context.clone(),
                        info: NodeInfo::ConjureSignature { signature, inner_node },
                    });

                    if let Some(inner_node_pair) = find_inner_node(pairs.clone()) {
                        let inner_stack_frame = create_stack_frame(
                            this,
                            inner_node_pair,
                            inner_node,
                            &stack_frame,
                            node,
                            RelationToParentNode::ConjureSignature_Inner,
                            vec![],
                            vec![signature],
                        );
                        recurse(this, inner_stack_frame);
                    }
                }
                Rule::give_signature => {
                    let mut pairs = stack_frame.node_pair.clone().into_inner();
                    let signature_pair = pairs.next().unwrap();

                    let node = stack_frame.reserved_node;
                    let inner_node = reserve_node(this);

                    let signature = resolve_signature(this, &stack_frame, signature_pair);

                    this.nodes[node] = Some(Node {
                        context: stack_frame.context.clone(),
                        info: NodeInfo::GiveSignature { signature, inner_node },
                    });

                    if let Some(inner_node_pair) = find_inner_node(pairs.clone()) {
                        let inner_stack_frame = create_stack_frame(
                            this,
                            inner_node_pair,
                            inner_node,
                            &stack_frame,
                            node,
                            RelationToParentNode::GiveSignature_Inner,
                            vec![],
                            vec![signature],
                        );
                        recurse(this, inner_stack_frame);
                    }
                }
                Rule::define_signature => {
                    let mut pairs = stack_frame.node_pair.clone().into_inner();
                    let signature_pair = pairs.next().unwrap();

                    let node = stack_frame.reserved_node;
                    let interior_node = reserve_node(this);
                    let inner_node = reserve_node(this);

                    let signature =
                        define_signature(this, node, SignatureOrigin::DefineSignature, signature_pair);

                    this.nodes[node] = Some(Node {
                        context: stack_frame.context.clone(),
                        info: NodeInfo::DefineSignature {
                            signature,
                            interior_node,
                            inner_node,
                        },
                    });

                    if let Some(interior_node_pair) = find_interior_node(pairs.clone()) {
                        let interior_stack_frame = create_stack_frame(
                            this,
                            interior_node_pair,
                            interior_node,
                            &stack_frame,
                            node,
                            RelationToParentNode::DefineSignature_Interior,
                            vec![],
                            vec![],
                        );
                        recurse(this, interior_stack_frame);
                    }

                    if let Some(inner_node_pair) = find_inner_node(pairs.clone()) {
                        let inner_stack_frame = create_stack_frame(
                            this,
                            inner_node_pair,
                            inner_node,
                            &stack_frame,
                            node,
                            RelationToParentNode::DefineSignature_Inner,
                            vec![],
                            vec![signature],
                        );
                        recurse(this, inner_stack_frame);
                    }
                }
                Rule::take => {
                    let mut pairs = stack_frame.node_pair.clone().into_inner();
                    let function_pair = pairs.next().unwrap();
                    let signature_pair = pairs.next().unwrap();

                    let node = stack_frame.reserved_node;
                    let inner_node = reserve_node(this);

                    let function = define_function(this, node, FunctionOrigin::Take, function_pair);
                    let signature = resolve_signature(this, &stack_frame, signature_pair);

                    this.nodes[node] = Some(Node {
                        context: stack_frame.context.clone(),
                        info: NodeInfo::Take {
                            function,
                            signature,
                            inner_node,
                        },
                    });

                    if let Some(inner_node_pair) = find_inner_node(pairs.clone()) {
                        let inner_stack_frame = create_stack_frame(
                            this,
                            inner_node_pair,
                            inner_node,
                            &stack_frame,
                            node,
                            RelationToParentNode::Take_Inner,
                            vec![function],
                            vec![],
                        );
                        recurse(this, inner_stack_frame);
                    }
                }
                Rule::conjure => {
                    let mut pairs = stack_frame.node_pair.clone().into_inner();
                    let function_pair = pairs.next().unwrap();
                    let signature_pair = pairs.next().unwrap();

                    let node = stack_frame.reserved_node;
                    let inner_node = reserve_node(this);

                    let function = define_function(this, node, FunctionOrigin::Conjure, function_pair);
                    let signature = resolve_signature(this, &stack_frame, signature_pair);

                    this.nodes[node] = Some(Node {
                        context: stack_frame.context.clone(),
                        info: NodeInfo::Conjure {
                            function,
                            signature,
                            inner_node,
                        },
                    });

                    if let Some(inner_node_pair) = find_inner_node(pairs.clone()) {
                        let inner_stack_frame = create_stack_frame(
                            this,
                            inner_node_pair,
                            inner_node,
                            &stack_frame,
                            node,
                            RelationToParentNode::Conjure_Inner,
                            vec![function],
                            vec![],
                        );
                        recurse(this, inner_stack_frame);
                    }
                }
                Rule::give => {
                    let mut pairs = stack_frame.node_pair.clone().into_inner();
                    let function_pair = pairs.next().unwrap();

                    let node = stack_frame.reserved_node;
                    let inner_node = reserve_node(this);

                    let function = resolve_function(this, &stack_frame, function_pair);

                    this.nodes[node] = Some(Node {
                        context: stack_frame.context.clone(),
                        info: NodeInfo::Give { function, inner_node },
                    });

                    if let Some(inner_node_pair) = find_inner_node(pairs.clone()) {
                        let inner_stack_frame = create_stack_frame(
                            this,
                            inner_node_pair,
                            inner_node,
                            &stack_frame,
                            node,
                            RelationToParentNode::Give_Inner,
                            vec![],
                            vec![],
                        );
                        recurse(this, inner_stack_frame);
                    }
                }
                Rule::prove => {
                    let mut pairs = stack_frame.node_pair.clone().into_inner();
                    let function_pair = pairs.next().unwrap();
                    let signature_pair = pairs.next().unwrap();

                    let node = stack_frame.reserved_node;
                    let interior_node = reserve_node(this);
                    let inner_node = reserve_node(this);

                    let function = define_function(this, node, FunctionOrigin::Prove, function_pair);
                    let signature = resolve_signature(this, &stack_frame, signature_pair);

                    this.nodes[node] = Some(Node {
                        context: stack_frame.context.clone(),
                        info: NodeInfo::Prove {
                            function,
                            signature,
                            interior_node,
                            inner_node,
                        },
                    });

                    if let Some(interior_node_pair) = find_interior_node(pairs.clone()) {
                        let interior_stack_frame = create_stack_frame(
                            this,
                            interior_node_pair,
                            interior_node,
                            &stack_frame,
                            node,
                            RelationToParentNode::Prove_Interior,
                            vec![],
                            vec![],
                        );
                        recurse(this, interior_stack_frame);
                    }

                    if let Some(inner_node_pair) = find_inner_node(pairs.clone()) {
                        let inner_stack_frame = create_stack_frame(
                            this,
                            inner_node_pair,
                            inner_node,
                            &stack_frame,
                            node,
                            RelationToParentNode::Prove_Inner,
                            vec![function],
                            vec![],
                        );
                        recurse(this, inner_stack_frame);
                    }
                }
                Rule::unwrap_giving => {
                    let mut pairs = stack_frame.node_pair.clone().into_inner();
                    let inner_function_pair = pairs.next().unwrap();
                    let outer_function_pair = pairs.next().unwrap();
                    let given_function_pair = pairs.next().unwrap();

                    let node = stack_frame.reserved_node;
                    let inner_node = reserve_node(this);

                    let inner_function = define_function(
                        this,
                        node,
                        FunctionOrigin::UnwrapGiving_InnerFunction,
                        inner_function_pair,
                    );
                    let outer_function = resolve_function(this, &stack_frame, outer_function_pair);
                    let given_function = resolve_function(this, &stack_frame, given_function_pair);

                    this.nodes[node] = Some(Node {
                        context: stack_frame.context.clone(),
                        info: NodeInfo::UnwrapGiving {
                            inner_function,
                            outer_function,
                            given_function,
                            inner_node,
                        },
                    });

                    if let Some(inner_node_pair) = find_inner_node(pairs.clone()) {
                        let inner_stack_frame = create_stack_frame(
                            this,
                            inner_node_pair,
                            inner_node,
                            &stack_frame,
                            node,
                            RelationToParentNode::UnwrapGiving_Inner,
                            vec![inner_function],
                            vec![],
                        );
                        recurse(this, inner_stack_frame);
                    }
                }
                Rule::unwrap_giving_signature => {
                    let mut pairs = stack_frame.node_pair.clone().into_inner();
                    let inner_function_pair = pairs.next().unwrap();
                    let outer_function_pair = pairs.next().unwrap();
                    let given_signature_pair = pairs.next().unwrap();

                    let node = stack_frame.reserved_node;
                    let inner_node = reserve_node(this);

                    let inner_function = define_function(
                        this,
                        node,
                        FunctionOrigin::UnwrapGivingSignature_InnerFunction,
                        inner_function_pair,
                    );
                    let outer_function = resolve_function(this, &stack_frame, outer_function_pair);
                    let given_signature = resolve_signature(this, &stack_frame, given_signature_pair);

                    this.nodes[node] = Some(Node {
                        context: stack_frame.context.clone(),
                        info: NodeInfo::UnwrapGivingSignature {
                            inner_function,
                            outer_function,
                            given_signature,
                            inner_node,
                        },
                    });

                    if let Some(inner_node_pair) = find_inner_node(pairs.clone()) {
                        let inner_stack_frame = create_stack_frame(
                            this,
                            inner_node_pair,
                            inner_node,
                            &stack_frame,
                            node,
                            RelationToParentNode::UnwrapGivingSignature_Inner,
                            vec![inner_function],
                            vec![],
                        );
                        recurse(this, inner_stack_frame);
                    }
                }
                Rule::unwrap_taking => {
                    let mut pairs = stack_frame.node_pair.clone().into_inner();
                    let inner_function_pair = pairs.next().unwrap();
                    let outer_function_pair = pairs.next().unwrap();
                    let taken_function_pair = pairs.next().unwrap();

                    let node = stack_frame.reserved_node;
                    let inner_node = reserve_node(this);

                    let inner_function = define_function(
                        this,
                        node,
                        FunctionOrigin::UnwrapTaking_InnerFunction,
                        inner_function_pair,
                    );
                    let outer_function = resolve_function(this, &stack_frame, outer_function_pair);
                    let taken_function = define_function(
                        this,
                        node,
                        FunctionOrigin::UnwrapTaking_TakenFunction,
                        taken_function_pair,
                    );

                    this.nodes[node] = Some(Node {
                        context: stack_frame.context.clone(),
                        info: NodeInfo::UnwrapTaking {
                            inner_function,
                            outer_function,
                            taken_function,
                            inner_node,
                        },
                    });

                    if let Some(inner_node_pair) = find_inner_node(pairs.clone()) {
                        let inner_stack_frame = create_stack_frame(
                            this,
                            inner_node_pair,
                            inner_node,
                            &stack_frame,
                            node,
                            RelationToParentNode::UnwrapTaking_Inner,
                            vec![inner_function, taken_function],
                            vec![],
                        );
                        recurse(this, inner_stack_frame);
                    }
                }
                Rule::unwrap_taking_signature => {
                    let mut pairs = stack_frame.node_pair.clone().into_inner();
                    let inner_function_pair = pairs.next().unwrap();
                    let outer_function_pair = pairs.next().unwrap();
                    let taken_signature_pair = pairs.next().unwrap();

                    let node = stack_frame.reserved_node;
                    let inner_node = reserve_node(this);

                    let inner_function = define_function(
                        this,
                        node,
                        FunctionOrigin::UnwrapTakingSignature_InnerFunction,
                        inner_function_pair,
                    );
                    let outer_function = resolve_function(this, &stack_frame, outer_function_pair);
                    let taken_signature = define_signature(
                        this,
                        node,
                        SignatureOrigin::UnwrapTakingSignature_TakenSignature,
                        taken_signature_pair,
                    );

                    this.nodes[node] = Some(Node {
                        context: stack_frame.context.clone(),
                        info: NodeInfo::UnwrapTakingSignature {
                            inner_function,
                            outer_function,
                            taken_signature,
                            inner_node,
                        },
                    });

                    if let Some(inner_node_pair) = find_inner_node(pairs.clone()) {
                        let inner_stack_frame = create_stack_frame(
                            this,
                            inner_node_pair,
                            inner_node,
                            &stack_frame,
                            node,
                            RelationToParentNode::UnwrapTakingSignature_Inner,
                            vec![inner_function],
                            vec![taken_signature],
                        );
                        recurse(this, inner_stack_frame);
                    }
                }
                _ => unreachable!(),
            }
        }
        let root = MyParser::parse(Rule::file, &text).unwrap().next().unwrap();
        let root_node = reserve_node(&mut this);
        recurse(
            &mut this,
            StackFrame {
                context: Context {
                    parent_info: None,
                    available_functions: vec![],
                    available_signatures: vec![],
                },
                node_pair: root,
                reserved_node: root_node,
                function_resolve_map: HashMap::new(),
                signature_resolve_map: HashMap::new(),
            },
        );

        this
    }
}