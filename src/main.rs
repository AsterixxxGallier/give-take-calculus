#![allow(unused)]

enum PathElement {
    Child,
    SignatureDefinition,
    Proof,
}

#[allow(non_camel_case_types)]
enum Leaf {
    TakeSignature,
    ConjureSignature,
    DefineSignature,
    Take,
    Conjure,
    Prove,
    TakeFromProcess_Process,
    TakeFromProcess_Gift,
}

struct Symbol {
    path: Vec<PathElement>,
    leaf: Leaf,
}

enum Statement {
    TakeSignature {
        child: Option<Box<Statement>>,
    },
    ConjureSignature {
        child: Option<Box<Statement>>,
    },
    GiveSignature {
        signature: Symbol,
        child: Option<Box<Statement>>,
    },
    DefineSignature {
        signature_definition: Box<Statement>,
        child: Option<Box<Statement>>,
    },

    Take {
        signature: Symbol,
        child: Option<Box<Statement>>,
    },
    Conjure {
        signature: Symbol,
        child: Option<Box<Statement>>,
    },
    Give {
        process: Symbol,
        child: Option<Box<Statement>>,
    },
    Prove {
        signature: Symbol,
        proof: Box<Statement>,
        child: Option<Box<Statement>>,
    },

    GiveToProcess {
        process: Symbol,
        gift: Symbol,
        child: Option<Box<Statement>>,
    },
    TakeFromProcess {
        process: Symbol,
        child: Option<Box<Statement>>,
    },
}

struct ObjectDependence {
    path: Vec<DependencePathElement>,
}

enum DependencePathElement {
    TakeSignature {
        symbol: Symbol,
        result: Object,
    },
    Take {
        symbol: Symbol,
        result: Object,
    },
    TakeFromProcess {
        symbol: Symbol,
        process: Object,
        result: Object,
    },
    NoDependence,
}

/// Process or Signature
#[allow(non_camel_case_types)]
struct Object {
    symbol: Symbol,
    dependence: ObjectDependence,
}

impl Into<Option<Box<Statement>>> for Statement {
    fn into(self) -> Option<Box<Statement>> {
        Some(self.into())
    }
}

fn main() {
    use PathElement::*;
    let statement = Statement::TakeSignature {
        child: Statement::DefineSignature {
            signature_definition: Statement::TakeSignature {
                child: Statement::DefineSignature {
                    signature_definition: Statement::Take {
                        signature: Symbol {
                            path: vec![Child, SignatureDefinition],
                            leaf: Leaf::TakeSignature,
                        },
                        child: Statement::Conjure {
                            signature: Symbol {
                                path: vec![],
                                leaf: Leaf::TakeSignature,
                            },
                            child: Statement::Give {
                                process: Symbol {
                                    path: vec![
                                        Child,
                                        SignatureDefinition,
                                        Child,
                                        SignatureDefinition,
                                        Child,
                                    ],
                                    leaf: Leaf::Conjure,
                                },
                                child: None,
                            }
                            .into(),
                        }
                        .into(),
                    }
                    .into(),
                    child: Statement::DefineSignature {
                        signature_definition: Statement::Take {
                            signature: Symbol {
                                path: vec![Child, SignatureDefinition, Child],
                                leaf: Leaf::DefineSignature,
                            },
                            child: Statement::Conjure {
                                signature: Symbol {
                                    path: vec![],
                                    leaf: Leaf::TakeSignature,
                                },
                                child: Statement::Give {
                                    process: Symbol {
                                        path: vec![
                                            Child,
                                            SignatureDefinition,
                                            Child,
                                            Child,
                                            SignatureDefinition,
                                            Child,
                                        ],
                                        leaf: Leaf::Conjure,
                                    },
                                    child: None,
                                }
                                    .into(),
                            }
                                .into(),
                        }
                        .into(),
                        child: Statement::Take {
                            signature: Symbol {
                                path: vec![Child, SignatureDefinition, Child, Child],
                                leaf: Leaf::DefineSignature,
                            },
                            child: Statement::Conjure {
                                signature: Symbol {
                                    path: vec![Child, SignatureDefinition],
                                    leaf: Leaf::TakeSignature,
                                },
                                child: Statement::Give {
                                    process: Symbol {
                                        path: vec![
                                            Child,
                                            SignatureDefinition,
                                            Child,
                                            Child,
                                            Child,
                                            Child,
                                        ],
                                        leaf: Leaf::Conjure,
                                    },
                                    child: None,
                                }
                                .into(),
                            }
                            .into(),
                        }
                        .into(),
                    }
                    .into(),
                }
                .into(),
            }
            .into(),
            child: Statement::Take {
                signature: Symbol {
                    path: vec![Child],
                    leaf: Leaf::DefineSignature,
                },
                child: None,
            }
            .into(),
        }
        .into(),
    };
}
