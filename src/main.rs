#![allow(unused)]

use pest::Parser;
use pest::iterators::{Pair, Pairs};
use pest_derive::Parser;
use std::collections::HashMap;
use std::{fs, io};

type SymbolIndex = usize;
type NodeIndex = usize;
type ScopeIndex = usize;

#[derive(Debug)]
struct Symbol {
    text: String,
    scope: ScopeIndex,
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

#[derive(Debug)]
struct Scope {
    origin: ScopeOrigin,
    parent: Option<ScopeIndex>,
    symbols_defined_here: Vec<SymbolIndex>,
    total_symbols: Vec<SymbolIndex>,
}

#[allow(non_camel_case_types)]
#[derive(Debug)]
enum ScopeOrigin {
    File,
    TakeSignature_Inner(NodeIndex),
    ConjureSignature_Inner(NodeIndex),
    GiveSignature_Inner(NodeIndex),
    DefineSignature_Inner(NodeIndex),
    DefineSignature_Interior(NodeIndex),
    Take_Inner(NodeIndex),
    Conjure_Inner(NodeIndex),
    Give_Inner(NodeIndex),
    Prove_Inner(NodeIndex),
    Prove_Interior(NodeIndex),
    UnwrapGiving_Inner(NodeIndex),
    UnwrapGivingSignature_Inner(NodeIndex),
    UnwrapTaking_Inner(NodeIndex),
    UnwrapTakingSignature_Inner(NodeIndex),
}

#[derive(Debug)]
enum Node {
    TakeSignature {
        symbol: SymbolIndex,
        outer_scope: ScopeIndex,
        inner_scope: ScopeIndex,
        inner_node: NodeIndex,
    },
    ConjureSignature {
        symbol: SymbolIndex,
        outer_scope: ScopeIndex,
        inner_scope: ScopeIndex,
        inner_node: NodeIndex,
    },
    GiveSignature {
        symbol: SymbolIndex,
        outer_scope: ScopeIndex,
        inner_scope: ScopeIndex,
        inner_node: NodeIndex,
    },
    DefineSignature {
        symbol: SymbolIndex,
        outer_scope: ScopeIndex,
        inner_scope: ScopeIndex,
        interior_scope: ScopeIndex,
        inner_node: NodeIndex,
        interior_node: NodeIndex,
    },
    Take {
        symbol: SymbolIndex,
        signature: SymbolIndex,
        outer_scope: ScopeIndex,
        inner_scope: ScopeIndex,
        inner_node: NodeIndex,
    },
    Conjure {
        symbol: SymbolIndex,
        signature: SymbolIndex,
        outer_scope: ScopeIndex,
        inner_scope: ScopeIndex,
        inner_node: NodeIndex,
    },
    Give {
        symbol: SymbolIndex,
        outer_scope: ScopeIndex,
        inner_scope: ScopeIndex,
        inner_node: NodeIndex,
    },
    Prove {
        symbol: SymbolIndex,
        signature: SymbolIndex,
        outer_scope: ScopeIndex,
        inner_scope: ScopeIndex,
        interior_scope: ScopeIndex,
        inner_node: NodeIndex,
        interior_node: NodeIndex,
    },
    UnwrapGiving {
        inner_symbol: SymbolIndex,
        outer_symbol: SymbolIndex,
        given_symbol: SymbolIndex,
        outer_scope: ScopeIndex,
        inner_scope: ScopeIndex,
        inner_node: NodeIndex,
    },
    UnwrapGivingSignature {
        inner_symbol: SymbolIndex,
        outer_symbol: SymbolIndex,
        given_symbol: SymbolIndex,
        outer_scope: ScopeIndex,
        inner_scope: ScopeIndex,
        inner_node: NodeIndex,
    },
    UnwrapTaking {
        inner_symbol: SymbolIndex,
        outer_symbol: SymbolIndex,
        taken_symbol: SymbolIndex,
        outer_scope: ScopeIndex,
        inner_scope: ScopeIndex,
        inner_node: NodeIndex,
    },
    UnwrapTakingSignature {
        inner_symbol: SymbolIndex,
        outer_symbol: SymbolIndex,
        taken_symbol: SymbolIndex,
        outer_scope: ScopeIndex,
        inner_scope: ScopeIndex,
        inner_node: NodeIndex,
    },
}

#[derive(Debug)]
struct File {
    nodes: Vec<Node>,
    symbols: Vec<Symbol>,
    scopes: Vec<Scope>,
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
            scopes: vec![],
        };
        this.scopes.push(Scope {
            origin: ScopeOrigin::File,
            parent: None,
            symbols_defined_here: vec![],
            total_symbols: vec![],
        });
        #[derive(Copy, Clone)]
        enum Stage {
            First,
            Second,
            End,
        }
        #[derive(Clone)]
        struct StackFrame<'i> {
            scope: ScopeIndex,
            node_pair: Pair<'i, Rule>,
            symbols: HashMap<String, SymbolIndex>,
        }
        fn define_symbol(
            this: &mut File,
            stack_frame: &StackFrame,
            origin_node: NodeIndex,
            origin: SymbolOrigin,
            inner_scope: ScopeIndex,
            inner_stack_frame: Option<&mut StackFrame>,
            pair: Pair<Rule>,
        ) -> SymbolIndex {
            let text = pair.as_str().to_owned();
            let symbol = this.symbols.len();
            this.symbols.push(Symbol {
                text: text.clone(),
                scope: inner_scope,
                origin_node,
                origin,
            });
            this.scopes[inner_scope].symbols_defined_here.push(symbol);
            this.scopes[inner_scope].total_symbols.push(symbol);
            if let Some(inner_stack_frame) = inner_stack_frame {
                inner_stack_frame.symbols.insert(text, symbol);
            }
            symbol
        }
        fn resolve_symbol(
            this: &mut File,
            stack_frame: &StackFrame,
            pair: Pair<Rule>,
        ) -> SymbolIndex {
            let text = pair.as_str().to_owned();
            if let Some(&symbol) = stack_frame.symbols.get(&text) {
                symbol
            } else {
                panic!("symbol {text} not found in scope");
            }
        }
        fn create_inner_or_interior_scope(
            this: &mut File,
            pairs: Pairs<Rule>,
            origin: ScopeOrigin,
            parent: ScopeIndex,
        ) -> ScopeIndex {
            let scope = this.scopes.len();

            this.scopes.push(Scope {
                origin,
                parent: Some(parent),
                symbols_defined_here: vec![],
                total_symbols: this.scopes[parent].total_symbols.clone(),
            });

            scope
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
        fn create_inner_stack_frame<'i>(
            mut pairs: Pairs<'i, Rule>,
            inner_scope: ScopeIndex,
            parent: &StackFrame<'i>,
        ) -> Option<StackFrame<'i>> {
            find_inner_node(pairs).map(|inner_node| StackFrame {
                scope: inner_scope,
                node_pair: inner_node,
                symbols: parent.symbols.clone(),
            })
        }
        fn create_interior_stack_frame<'i>(
            mut pairs: Pairs<'i, Rule>,
            interior_scope: ScopeIndex,
            parent: &StackFrame<'i>,
        ) -> Option<StackFrame<'i>> {
            find_interior_node(pairs).map(|interior_node| StackFrame {
                scope: interior_scope,
                node_pair: interior_node,
                symbols: parent.symbols.clone(),
            })
        }
        fn recurse(this: &mut File, stack_frame: StackFrame) {
            match stack_frame.node_pair.as_rule() {
                Rule::take_signature => {
                    let mut pairs = stack_frame.node_pair.clone().into_inner();
                    let symbol_pair = pairs.next().unwrap();

                    let node = this.nodes.len();
                    let inner_scope = create_inner_or_interior_scope(
                        this,
                        pairs.clone(),
                        ScopeOrigin::TakeSignature_Inner(node),
                        stack_frame.scope,
                    );
                    let mut inner_stack_frame =
                        create_inner_stack_frame(pairs, inner_scope, &stack_frame);

                    let symbol = define_symbol(
                        this,
                        &stack_frame,
                        node,
                        SymbolOrigin::TakeSignature,
                        inner_scope,
                        inner_stack_frame.as_mut(),
                        symbol_pair,
                    );

                    this.nodes.push(Node::TakeSignature {
                        symbol,
                        outer_scope: stack_frame.scope,
                        inner_scope,
                        inner_node,
                    });

                    if let Some(inner_stack_frame) = inner_stack_frame {
                        recurse(this, inner_stack_frame);
                    }
                }
                Rule::conjure_signature => {
                    let mut pairs = stack_frame.node_pair.clone().into_inner();
                    let symbol_pair = pairs.next().unwrap();

                    let node = this.nodes.len();
                    let inner_scope = create_inner_or_interior_scope(
                        this,
                        pairs.clone(),
                        ScopeOrigin::ConjureSignature_Inner(node),
                        stack_frame.scope,
                    );
                    let mut inner_stack_frame =
                        create_inner_stack_frame(pairs, inner_scope, &stack_frame);

                    let symbol = define_symbol(
                        this,
                        &stack_frame,
                        node,
                        SymbolOrigin::ConjureSignature,
                        inner_scope,
                        inner_stack_frame.as_mut(),
                        symbol_pair,
                    );

                    this.nodes.push(Node::ConjureSignature {
                        symbol,
                        outer_scope: stack_frame.scope,
                        inner_scope,
                    });

                    if let Some(inner_stack_frame) = inner_stack_frame {
                        recurse(this, inner_stack_frame);
                    }
                }
                Rule::give_signature => {
                    let mut pairs = stack_frame.node_pair.clone().into_inner();
                    let symbol_pair = pairs.next().unwrap();

                    let node = this.nodes.len();
                    let inner_scope = create_inner_or_interior_scope(
                        this,
                        pairs.clone(),
                        ScopeOrigin::GiveSignature_Inner(node),
                        stack_frame.scope,
                    );
                    let mut inner_stack_frame =
                        create_inner_stack_frame(pairs, inner_scope, &stack_frame);

                    let symbol = resolve_symbol(this, &stack_frame, symbol_pair);
                    this.nodes.push(Node::GiveSignature {
                        symbol,
                        outer_scope: stack_frame.scope,
                        inner_scope,
                    });

                    if let Some(inner_stack_frame) = inner_stack_frame {
                        recurse(this, inner_stack_frame);
                    }
                }
                Rule::define_signature => {
                    let mut pairs = stack_frame.node_pair.clone().into_inner();
                    let symbol_pair = pairs.next().unwrap();

                    let node = this.nodes.len();
                    let interior_scope = create_inner_or_interior_scope(
                        this,
                        pairs.clone(),
                        ScopeOrigin::DefineSignature_Interior(node),
                        stack_frame.scope,
                    );
                    let inner_scope = create_inner_or_interior_scope(
                        this,
                        pairs.clone(),
                        ScopeOrigin::DefineSignature_Inner(node),
                        stack_frame.scope,
                    );
                    let mut interior_stack_frame =
                        create_interior_stack_frame(pairs.clone(), interior_scope, &stack_frame);
                    let mut inner_stack_frame =
                        create_inner_stack_frame(pairs, inner_scope, &stack_frame);

                    let symbol = define_symbol(
                        this,
                        &stack_frame,
                        node,
                        SymbolOrigin::DefineSignature,
                        inner_scope,
                        inner_stack_frame.as_mut(),
                        symbol_pair,
                    );

                    this.nodes.push(Node::DefineSignature {
                        symbol,
                        outer_scope: stack_frame.scope,
                        interior_scope,
                        inner_scope,
                    });

                    if let Some(interior_stack_frame) = interior_stack_frame {
                        recurse(this, interior_stack_frame);
                    }
                    if let Some(inner_stack_frame) = inner_stack_frame {
                        recurse(this, inner_stack_frame);
                    }
                }
                Rule::take => {
                    let mut pairs = stack_frame.node_pair.clone().into_inner();
                    let symbol_pair = pairs.next().unwrap();
                    let signature_pair = pairs.next().unwrap();

                    let node = this.nodes.len();
                    let inner_scope = create_inner_or_interior_scope(
                        this,
                        pairs.clone(),
                        ScopeOrigin::Take_Inner(node),
                        stack_frame.scope,
                    );
                    let mut inner_stack_frame =
                        create_inner_stack_frame(pairs, inner_scope, &stack_frame);

                    let symbol = define_symbol(
                        this,
                        &stack_frame,
                        node,
                        SymbolOrigin::Take,
                        inner_scope,
                        inner_stack_frame.as_mut(),
                        symbol_pair,
                    );
                    let signature = resolve_symbol(this, &stack_frame, signature_pair);

                    this.nodes.push(Node::Take {
                        symbol,
                        signature,
                        outer_scope: stack_frame.scope,
                        inner_scope,
                    });

                    if let Some(inner_stack_frame) = inner_stack_frame {
                        recurse(this, inner_stack_frame);
                    }
                }
                Rule::conjure => {
                    let mut pairs = stack_frame.node_pair.clone().into_inner();
                    let symbol_pair = pairs.next().unwrap();
                    let signature_pair = pairs.next().unwrap();

                    let node = this.nodes.len();
                    let inner_scope = create_inner_or_interior_scope(
                        this,
                        pairs.clone(),
                        ScopeOrigin::Conjure_Inner(node),
                        stack_frame.scope,
                    );
                    let mut inner_stack_frame =
                        create_inner_stack_frame(pairs, inner_scope, &stack_frame);

                    let symbol = define_symbol(
                        this,
                        &stack_frame,
                        node,
                        SymbolOrigin::Conjure,
                        inner_scope,
                        inner_stack_frame.as_mut(),
                        symbol_pair,
                    );
                    let signature = resolve_symbol(this, &stack_frame, signature_pair);

                    this.nodes.push(Node::Conjure {
                        symbol,
                        signature,
                        outer_scope: stack_frame.scope,
                        inner_scope,
                    });

                    if let Some(inner_stack_frame) = inner_stack_frame {
                        recurse(this, inner_stack_frame);
                    }
                }
                Rule::give => {
                    let mut pairs = stack_frame.node_pair.clone().into_inner();
                    let symbol_pair = pairs.next().unwrap();

                    let node = this.nodes.len();
                    let inner_scope = create_inner_or_interior_scope(
                        this,
                        pairs.clone(),
                        ScopeOrigin::Give_Inner(node),
                        stack_frame.scope,
                    );
                    let mut inner_stack_frame =
                        create_inner_stack_frame(pairs, inner_scope, &stack_frame);

                    let symbol = resolve_symbol(this, &stack_frame, symbol_pair);

                    this.nodes.push(Node::Give {
                        symbol,
                        outer_scope: stack_frame.scope,
                        inner_scope,
                    });

                    if let Some(inner_stack_frame) = inner_stack_frame {
                        recurse(this, inner_stack_frame);
                    }
                }
                Rule::prove => {
                    let mut pairs = stack_frame.node_pair.clone().into_inner();
                    let symbol_pair = pairs.next().unwrap();
                    let signature_pair = pairs.next().unwrap();

                    let node = this.nodes.len();
                    let interior_scope = create_inner_or_interior_scope(
                        this,
                        pairs.clone(),
                        ScopeOrigin::Prove_Interior(node),
                        stack_frame.scope,
                    );
                    let inner_scope = create_inner_or_interior_scope(
                        this,
                        pairs.clone(),
                        ScopeOrigin::Prove_Inner(node),
                        stack_frame.scope,
                    );
                    let mut interior_stack_frame =
                        create_interior_stack_frame(pairs.clone(), interior_scope, &stack_frame);
                    let mut inner_stack_frame =
                        create_inner_stack_frame(pairs, inner_scope, &stack_frame);

                    let symbol = define_symbol(
                        this,
                        &stack_frame,
                        node,
                        SymbolOrigin::Prove,
                        inner_scope,
                        inner_stack_frame.as_mut(),
                        symbol_pair,
                    );
                    let signature = resolve_symbol(
                        this,
                        &stack_frame,
                        signature_pair,
                    );

                    this.nodes.push(Node::Prove {
                        symbol,
                        signature,
                        outer_scope: stack_frame.scope,
                        interior_scope,
                        inner_scope,
                    });

                    if let Some(interior_stack_frame) = interior_stack_frame {
                        recurse(this, interior_stack_frame);
                    }
                    if let Some(inner_stack_frame) = inner_stack_frame {
                        recurse(this, inner_stack_frame);
                    }
                }
                Rule::unwrap_giving => {
                    let mut pairs = stack_frame.node_pair.clone().into_inner();
                    let inner_symbol_pair = pairs.next().unwrap();
                    let outer_symbol_pair = pairs.next().unwrap();
                    let given_symbol_pair = pairs.next().unwrap();

                    let node = this.nodes.len();
                    let inner_scope = create_inner_or_interior_scope(
                        this,
                        pairs.clone(),
                        ScopeOrigin::UnwrapGiving_Inner(node),
                        stack_frame.scope,
                    );
                    let mut inner_stack_frame =
                        create_inner_stack_frame(pairs, inner_scope, &stack_frame);

                    let inner_symbol = define_symbol(
                        this,
                        &stack_frame,
                        node,
                        SymbolOrigin::UnwrapGiving_Unwrapped,
                        inner_scope,
                        inner_stack_frame.as_mut(),
                        inner_symbol_pair,
                    );
                    let outer_symbol = resolve_symbol(this, &stack_frame, outer_symbol_pair);
                    let given_symbol = resolve_symbol(this, &stack_frame, given_symbol_pair);

                    this.nodes.push(Node::UnwrapGiving {
                        inner_symbol,
                        outer_symbol,
                        given_symbol,
                        outer_scope: stack_frame.scope,
                        inner_scope,
                    });

                    if let Some(inner_stack_frame) = inner_stack_frame {
                        recurse(this, inner_stack_frame);
                    }
                }
                Rule::unwrap_giving_signature => {
                    let mut pairs = stack_frame.node_pair.clone().into_inner();
                    let inner_symbol_pair = pairs.next().unwrap();
                    let outer_symbol_pair = pairs.next().unwrap();
                    let given_symbol_pair = pairs.next().unwrap();

                    let node = this.nodes.len();
                    let inner_scope = create_inner_or_interior_scope(
                        this,
                        pairs.clone(),
                        ScopeOrigin::UnwrapGivingSignature_Inner(node),
                        stack_frame.scope,
                    );
                    let mut inner_stack_frame =
                        create_inner_stack_frame(pairs, inner_scope, &stack_frame);

                    let inner_symbol = define_symbol(
                        this,
                        &stack_frame,
                        node,
                        SymbolOrigin::UnwrapGivingSignature_Unwrapped,
                        inner_scope,
                        inner_stack_frame.as_mut(),
                        inner_symbol_pair,
                    );
                    let outer_symbol = resolve_symbol(this, &stack_frame, outer_symbol_pair);
                    let given_symbol = resolve_symbol(this, &stack_frame, given_symbol_pair);

                    this.nodes.push(Node::UnwrapGivingSignature {
                        inner_symbol,
                        outer_symbol,
                        given_symbol,
                        outer_scope: stack_frame.scope,
                        inner_scope,
                    });

                    if let Some(inner_stack_frame) = inner_stack_frame {
                        recurse(this, inner_stack_frame);
                    }
                }
                Rule::unwrap_taking => {
                    let mut pairs = stack_frame.node_pair.clone().into_inner();
                    let inner_symbol_pair = pairs.next().unwrap();
                    let outer_symbol_pair = pairs.next().unwrap();
                    let taken_symbol_pair = pairs.next().unwrap();

                    let node = this.nodes.len();
                    let inner_scope = create_inner_or_interior_scope(
                        this,
                        pairs.clone(),
                        ScopeOrigin::UnwrapTaking_Inner(node),
                        stack_frame.scope,
                    );
                    let mut inner_stack_frame =
                        create_inner_stack_frame(pairs, inner_scope, &stack_frame);

                    let inner_symbol = define_symbol(
                        this,
                        &stack_frame,
                        node,
                        SymbolOrigin::UnwrapTaking_Unwrapped,
                        inner_scope,
                        inner_stack_frame.as_mut(),
                        inner_symbol_pair,
                    );
                    let outer_symbol = resolve_symbol(this, &stack_frame, outer_symbol_pair);
                    let taken_symbol = define_symbol(
                        this,
                        &stack_frame,
                        node,
                        SymbolOrigin::UnwrapTaking_Taken,
                        inner_scope,
                        inner_stack_frame.as_mut(),
                        taken_symbol_pair,
                    );

                    this.nodes.push(Node::UnwrapTaking {
                        inner_symbol,
                        outer_symbol,
                        taken_symbol,
                        outer_scope: stack_frame.scope,
                        inner_scope,
                    });

                    if let Some(inner_stack_frame) = inner_stack_frame {
                        recurse(this, inner_stack_frame);
                    }
                }
                Rule::unwrap_taking_signature => {
                    let mut pairs = stack_frame.node_pair.clone().into_inner();
                    let inner_symbol_pair = pairs.next().unwrap();
                    let outer_symbol_pair = pairs.next().unwrap();
                    let taken_symbol_pair = pairs.next().unwrap();

                    let node = this.nodes.len();
                    let inner_scope = create_inner_or_interior_scope(
                        this,
                        pairs.clone(),
                        ScopeOrigin::UnwrapTakingSignature_Inner(node),
                        stack_frame.scope,
                    );
                    let mut inner_stack_frame =
                        create_inner_stack_frame(pairs, inner_scope, &stack_frame);

                    let inner_symbol = define_symbol(
                        this,
                        &stack_frame,
                        node,
                        SymbolOrigin::UnwrapTakingSignature_Unwrapped,
                        inner_scope,
                        inner_stack_frame.as_mut(),
                        inner_symbol_pair,
                    );
                    let outer_symbol = resolve_symbol(this, &stack_frame, outer_symbol_pair);
                    let taken_symbol = define_symbol(
                        this,
                        &stack_frame,
                        node,
                        SymbolOrigin::UnwrapTakingSignature_Taken,
                        inner_scope,
                        inner_stack_frame.as_mut(),
                        taken_symbol_pair,
                    );

                    this.nodes.push(Node::UnwrapTakingSignature {
                        inner_symbol,
                        outer_symbol,
                        taken_symbol,
                        outer_scope: stack_frame.scope,
                        inner_scope,
                    });

                    if let Some(inner_stack_frame) = inner_stack_frame {
                        recurse(this, inner_stack_frame);
                    }
                }
                _ => unreachable!(),
            }
        }
        let root = MyParser::parse(Rule::file, &text).unwrap().next().unwrap();
        recurse(
            &mut this,
            StackFrame {
                scope: 0,
                node_pair: root,
                symbols: HashMap::new(),
            },
        );

        this
    }

    fn check(&self) {
        enum Value {
            TakeSignature(SymbolIndex),
        }

        fn recurse(this: &File, symbol_values: &mut HashMap<SymbolIndex, Value>, node: &Node) {
            match *node {
                Node::TakeSignature { symbol, outer_scope, inner_scope } => {
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

        recurse(self, &mut symbol_values, root_node);
    }
}

fn main() {
    let file = File::parse("resources/false.txt");
    println!("{file:#?}");
}
