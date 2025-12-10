#![allow(unused)]

use pest::Parser;
use pest::iterators::{Pair, Pairs};
use pest_derive::Parser;
use std::collections::HashMap;
use std::{fs, io};

type SymbolIndex = usize;
type NodeIndex = usize;

#[derive(Debug)]
struct Symbol {
    text: String,
    origin_node: NodeIndex,
    origin: SymbolOrigin,
}

#[allow(non_camel_case_types)]
#[derive(Debug)]
enum SymbolOrigin {
    TakeSignature,
    ConjureSignature,
    DefineSignature,
    Take,
    Conjure,
    Prove,
    UnwrapGiving_Unwrapped,
    UnwrapGivingSignature_Unwrapped,
    UnwrapTaking_Unwrapped,
    UnwrapTaking_Taken,
    UnwrapTakingSignature_Unwrapped,
    UnwrapTakingSignature_Taken,
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
    available_symbols: Vec<SymbolIndex>,
}

#[derive(Debug)]
struct Node {
    context: Context,
    info: NodeInfo,
}

#[derive(Debug)]
enum NodeInfo {
    TakeSignature {
        symbol: SymbolIndex,
        inner_node: NodeIndex,
    },
    ConjureSignature {
        symbol: SymbolIndex,
        inner_node: NodeIndex,
    },
    GiveSignature {
        symbol: SymbolIndex,
        inner_node: NodeIndex,
    },
    DefineSignature {
        symbol: SymbolIndex,
        inner_node: NodeIndex,
        interior_node: NodeIndex,
    },
    Take {
        symbol: SymbolIndex,
        signature: SymbolIndex,
        inner_node: NodeIndex,
    },
    Conjure {
        symbol: SymbolIndex,
        signature: SymbolIndex,
        inner_node: NodeIndex,
    },
    Give {
        symbol: SymbolIndex,
        inner_node: NodeIndex,
    },
    Prove {
        symbol: SymbolIndex,
        signature: SymbolIndex,
        inner_node: NodeIndex,
        interior_node: NodeIndex,
    },
    UnwrapGiving {
        inner_symbol: SymbolIndex,
        outer_symbol: SymbolIndex,
        given_symbol: SymbolIndex,
        inner_node: NodeIndex,
    },
    UnwrapGivingSignature {
        inner_symbol: SymbolIndex,
        outer_symbol: SymbolIndex,
        given_symbol: SymbolIndex,
        inner_node: NodeIndex,
    },
    UnwrapTaking {
        inner_symbol: SymbolIndex,
        outer_symbol: SymbolIndex,
        taken_symbol: SymbolIndex,
        inner_node: NodeIndex,
    },
    UnwrapTakingSignature {
        inner_symbol: SymbolIndex,
        outer_symbol: SymbolIndex,
        taken_symbol: SymbolIndex,
        inner_node: NodeIndex,
    },
}

#[derive(Debug)]
struct File {
    nodes: Vec<Option<Node>>,
    symbols: Vec<Symbol>,
}

#[derive(Parser)]
#[grammar = "syntax.pest"]
struct MyParser;

impl File {
    fn parse(path: &str) -> Self {
        let text = fs::read_to_string(path).unwrap();
        let mut this = Self {
            nodes: vec![],
            symbols: vec![],
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
            symbol_resolve_map: HashMap<String, SymbolIndex>,
        }
        fn define_symbol(
            this: &mut File,
            origin_node: NodeIndex,
            origin: SymbolOrigin,
            pair: Pair<Rule>,
        ) -> SymbolIndex {
            let text = pair.as_str().to_owned();
            let symbol = this.symbols.len();
            this.symbols.push(Symbol {
                text: text.clone(),
                origin_node,
                origin,
            });
            symbol
        }
        fn resolve_symbol(
            this: &mut File,
            stack_frame: &StackFrame,
            pair: Pair<Rule>,
        ) -> SymbolIndex {
            let text = pair.as_str().to_owned();
            if let Some(&symbol) = stack_frame.symbol_resolve_map.get(&text) {
                symbol
            } else {
                panic!("symbol {text} not found in scope");
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
            additional_available_symbols: Vec<SymbolIndex>,
        ) -> StackFrame<'i> {
            let available_symbols = parent
                .context
                .available_symbols
                .iter()
                .chain(additional_available_symbols.iter())
                .copied()
                .collect();
            let symbol_resolve_map = parent
                .symbol_resolve_map
                .clone()
                .into_iter()
                .chain(
                    additional_available_symbols
                        .into_iter()
                        .map(|symbol| (this.symbols[symbol].text.clone(), symbol)),
                )
                .collect();
            StackFrame {
                context: Context {
                    parent_info: Some(ParentInfo {
                        parent_node,
                        relation_to_parent_node,
                    }),
                    available_symbols,
                },
                node_pair,
                reserved_node,
                symbol_resolve_map,
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
                    let symbol_pair = pairs.next().unwrap();

                    let node = stack_frame.reserved_node;
                    let inner_node = reserve_node(this);

                    let symbol =
                        define_symbol(this, node, SymbolOrigin::TakeSignature, symbol_pair);

                    this.nodes[node] = Some(Node {
                        context: stack_frame.context.clone(),
                        info: NodeInfo::TakeSignature { symbol, inner_node },
                    });

                    if let Some(inner_node_pair) = find_inner_node(pairs.clone()) {
                        let inner_stack_frame = create_stack_frame(
                            this,
                            inner_node_pair,
                            inner_node,
                            &stack_frame,
                            node,
                            RelationToParentNode::TakeSignature_Inner,
                            vec![symbol],
                        );
                        recurse(this, inner_stack_frame);
                    }
                }
                Rule::conjure_signature => {
                    let mut pairs = stack_frame.node_pair.clone().into_inner();
                    let symbol_pair = pairs.next().unwrap();

                    let node = stack_frame.reserved_node;
                    let inner_node = reserve_node(this);

                    let symbol =
                        define_symbol(this, node, SymbolOrigin::ConjureSignature, symbol_pair);

                    this.nodes[node] = Some(Node {
                        context: stack_frame.context.clone(),
                        info: NodeInfo::ConjureSignature { symbol, inner_node },
                    });

                    if let Some(inner_node_pair) = find_inner_node(pairs.clone()) {
                        let inner_stack_frame = create_stack_frame(
                            this,
                            inner_node_pair,
                            inner_node,
                            &stack_frame,
                            node,
                            RelationToParentNode::ConjureSignature_Inner,
                            vec![symbol],
                        );
                        recurse(this, inner_stack_frame);
                    }
                }
                Rule::give_signature => {
                    let mut pairs = stack_frame.node_pair.clone().into_inner();
                    let symbol_pair = pairs.next().unwrap();

                    let node = stack_frame.reserved_node;
                    let inner_node = reserve_node(this);

                    let symbol = resolve_symbol(this, &stack_frame, symbol_pair);

                    this.nodes[node] = Some(Node {
                        context: stack_frame.context.clone(),
                        info: NodeInfo::GiveSignature { symbol, inner_node },
                    });

                    if let Some(inner_node_pair) = find_inner_node(pairs.clone()) {
                        let inner_stack_frame = create_stack_frame(
                            this,
                            inner_node_pair,
                            inner_node,
                            &stack_frame,
                            node,
                            RelationToParentNode::GiveSignature_Inner,
                            vec![symbol],
                        );
                        recurse(this, inner_stack_frame);
                    }
                }
                Rule::define_signature => {
                    let mut pairs = stack_frame.node_pair.clone().into_inner();
                    let symbol_pair = pairs.next().unwrap();

                    let node = stack_frame.reserved_node;
                    let interior_node = reserve_node(this);
                    let inner_node = reserve_node(this);

                    let symbol =
                        define_symbol(this, node, SymbolOrigin::DefineSignature, symbol_pair);

                    this.nodes[node] = Some(Node {
                        context: stack_frame.context.clone(),
                        info: NodeInfo::DefineSignature {
                            symbol,
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
                            vec![symbol],
                        );
                        recurse(this, inner_stack_frame);
                    }
                }
                Rule::take => {
                    let mut pairs = stack_frame.node_pair.clone().into_inner();
                    let symbol_pair = pairs.next().unwrap();
                    let signature_pair = pairs.next().unwrap();

                    let node = stack_frame.reserved_node;
                    let inner_node = reserve_node(this);

                    let symbol = define_symbol(this, node, SymbolOrigin::Take, symbol_pair);
                    let signature = resolve_symbol(this, &stack_frame, signature_pair);

                    this.nodes[node] = Some(Node {
                        context: stack_frame.context.clone(),
                        info: NodeInfo::Take {
                            symbol,
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
                            vec![symbol],
                        );
                        recurse(this, inner_stack_frame);
                    }
                }
                Rule::conjure => {
                    let mut pairs = stack_frame.node_pair.clone().into_inner();
                    let symbol_pair = pairs.next().unwrap();
                    let signature_pair = pairs.next().unwrap();

                    let node = stack_frame.reserved_node;
                    let inner_node = reserve_node(this);

                    let symbol = define_symbol(this, node, SymbolOrigin::Conjure, symbol_pair);
                    let signature = resolve_symbol(this, &stack_frame, signature_pair);

                    this.nodes[node] = Some(Node {
                        context: stack_frame.context.clone(),
                        info: NodeInfo::Conjure {
                            symbol,
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
                            vec![symbol],
                        );
                        recurse(this, inner_stack_frame);
                    }
                }
                Rule::give => {
                    let mut pairs = stack_frame.node_pair.clone().into_inner();
                    let symbol_pair = pairs.next().unwrap();

                    let node = stack_frame.reserved_node;
                    let inner_node = reserve_node(this);

                    let symbol = resolve_symbol(this, &stack_frame, symbol_pair);

                    this.nodes[node] = Some(Node {
                        context: stack_frame.context.clone(),
                        info: NodeInfo::Give { symbol, inner_node },
                    });

                    if let Some(inner_node_pair) = find_inner_node(pairs.clone()) {
                        let inner_stack_frame = create_stack_frame(
                            this,
                            inner_node_pair,
                            inner_node,
                            &stack_frame,
                            node,
                            RelationToParentNode::Give_Inner,
                            vec![symbol],
                        );
                        recurse(this, inner_stack_frame);
                    }
                }
                Rule::prove => {
                    let mut pairs = stack_frame.node_pair.clone().into_inner();
                    let symbol_pair = pairs.next().unwrap();
                    let signature_pair = pairs.next().unwrap();

                    let node = stack_frame.reserved_node;
                    let interior_node = reserve_node(this);
                    let inner_node = reserve_node(this);

                    let symbol = define_symbol(this, node, SymbolOrigin::Prove, symbol_pair);
                    let signature = resolve_symbol(this, &stack_frame, signature_pair);

                    this.nodes[node] = Some(Node {
                        context: stack_frame.context.clone(),
                        info: NodeInfo::Prove {
                            symbol,
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
                            vec![symbol],
                        );
                        recurse(this, inner_stack_frame);
                    }
                }
                Rule::unwrap_giving => {
                    let mut pairs = stack_frame.node_pair.clone().into_inner();
                    let inner_symbol_pair = pairs.next().unwrap();
                    let outer_symbol_pair = pairs.next().unwrap();
                    let given_symbol_pair = pairs.next().unwrap();

                    let node = stack_frame.reserved_node;
                    let inner_node = reserve_node(this);

                    let inner_symbol = define_symbol(
                        this,
                        node,
                        SymbolOrigin::UnwrapGiving_Unwrapped,
                        inner_symbol_pair,
                    );
                    let outer_symbol = resolve_symbol(this, &stack_frame, outer_symbol_pair);
                    let given_symbol = resolve_symbol(this, &stack_frame, given_symbol_pair);

                    this.nodes[node] = Some(Node {
                        context: stack_frame.context.clone(),
                        info: NodeInfo::UnwrapGiving {
                            inner_symbol,
                            outer_symbol,
                            given_symbol,
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
                            vec![inner_symbol],
                        );
                        recurse(this, inner_stack_frame);
                    }
                }
                Rule::unwrap_giving_signature => {
                    let mut pairs = stack_frame.node_pair.clone().into_inner();
                    let inner_symbol_pair = pairs.next().unwrap();
                    let outer_symbol_pair = pairs.next().unwrap();
                    let given_symbol_pair = pairs.next().unwrap();

                    let node = stack_frame.reserved_node;
                    let inner_node = reserve_node(this);

                    let inner_symbol = define_symbol(
                        this,
                        node,
                        SymbolOrigin::UnwrapGivingSignature_Unwrapped,
                        inner_symbol_pair,
                    );
                    let outer_symbol = resolve_symbol(this, &stack_frame, outer_symbol_pair);
                    let given_symbol = resolve_symbol(this, &stack_frame, given_symbol_pair);

                    this.nodes[node] = Some(Node {
                        context: stack_frame.context.clone(),
                        info: NodeInfo::UnwrapGivingSignature {
                            inner_symbol,
                            outer_symbol,
                            given_symbol,
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
                            vec![inner_symbol],
                        );
                        recurse(this, inner_stack_frame);
                    }
                }
                Rule::unwrap_taking => {
                    let mut pairs = stack_frame.node_pair.clone().into_inner();
                    let inner_symbol_pair = pairs.next().unwrap();
                    let outer_symbol_pair = pairs.next().unwrap();
                    let taken_symbol_pair = pairs.next().unwrap();

                    let node = stack_frame.reserved_node;
                    let inner_node = reserve_node(this);

                    let inner_symbol = define_symbol(
                        this,
                        node,
                        SymbolOrigin::UnwrapTaking_Unwrapped,
                        inner_symbol_pair,
                    );
                    let outer_symbol = resolve_symbol(this, &stack_frame, outer_symbol_pair);
                    let taken_symbol = define_symbol(
                        this,
                        node,
                        SymbolOrigin::UnwrapTaking_Taken,
                        taken_symbol_pair,
                    );

                    this.nodes[node] = Some(Node {
                        context: stack_frame.context.clone(),
                        info: NodeInfo::UnwrapTaking {
                            inner_symbol,
                            outer_symbol,
                            taken_symbol,
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
                            vec![inner_symbol, taken_symbol],
                        );
                        recurse(this, inner_stack_frame);
                    }
                }
                Rule::unwrap_taking_signature => {
                    let mut pairs = stack_frame.node_pair.clone().into_inner();
                    let inner_symbol_pair = pairs.next().unwrap();
                    let outer_symbol_pair = pairs.next().unwrap();
                    let taken_symbol_pair = pairs.next().unwrap();

                    let node = stack_frame.reserved_node;
                    let inner_node = reserve_node(this);

                    let inner_symbol = define_symbol(
                        this,
                        node,
                        SymbolOrigin::UnwrapTakingSignature_Unwrapped,
                        inner_symbol_pair,
                    );
                    let outer_symbol = resolve_symbol(this, &stack_frame, outer_symbol_pair);
                    let taken_symbol = define_symbol(
                        this,
                        node,
                        SymbolOrigin::UnwrapTakingSignature_Taken,
                        taken_symbol_pair,
                    );

                    this.nodes[node] = Some(Node {
                        context: stack_frame.context.clone(),
                        info: NodeInfo::UnwrapTakingSignature {
                            inner_symbol,
                            outer_symbol,
                            taken_symbol,
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
                            vec![inner_symbol, taken_symbol],
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
                    available_symbols: vec![],
                },
                node_pair: root,
                reserved_node: root_node,
                symbol_resolve_map: HashMap::new(),
            },
        );

        this
    }

    fn check(&self) {
        /*enum Value {
            TakeSignature(SymbolIndex),
        }

        fn recurse(this: &File, symbol_values: &mut HashMap<SymbolIndex, Value>, node: &Node) {
            match *node {
                Node::TakeSignature {
                    symbol,
                    outer_scope,
                    inner_scope,
                } => {
                    symbol_values.insert(symbol, Value::TakeSignature(symbol));
                }
                Node::ConjureSignature { .. } => {}
                Node::GiveSignature { .. } => {}
                Node::DefineSignature { .. } => {}
                Node::Take { .. } => {}
                Node::Conjure { .. } => {}
                Node::Give { .. } => {}
                Node::Prove { .. } => {}
                Node::UnwrapGiving { .. } => {}
                Node::UnwrapGivingSignature { .. } => {}
                Node::UnwrapTaking { .. } => {}
                Node::UnwrapTakingSignature { .. } => {}
            }
        }

        let root_node = &self.nodes[0];
        let mut symbol_values = HashMap::new();

        recurse(self, &mut symbol_values, root_node);*/
    }
}

fn main() {
    let file = File::parse("resources/false.txt");
    println!("{file:#?}");
}
