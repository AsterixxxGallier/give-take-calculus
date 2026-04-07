use crate::check::{FunctionDefinition, FunctionId, SignatureDefinition, SignatureId};
use crate::parse::{Function, Signature, SourceLocation};
use ordermap::OrderMap;

#[derive(Default)]
pub(super) struct Resolver<'s> {
    signatures: OrderMap<SignatureId, SignatureDefinition<'s>>,
    functions: OrderMap<FunctionId, FunctionDefinition<'s>>,
}

#[allow(unused)]
impl<'s> Resolver<'s> {
    pub(super) fn insert_signature(
        &mut self,
        id: SignatureId,
        signature: Signature<'s>,
        statement: SourceLocation<'s>,
    ) {
        let definition = SignatureDefinition {
            signature,
            statement,
        };
        self.signatures.insert(id, definition);
    }

    pub(super) fn insert_function(
        &mut self,
        id: FunctionId,
        function: Function<'s>,
        statement: SourceLocation<'s>,
    ) {
        let definition = FunctionDefinition {
            function,
            statement,
        };
        self.functions.insert(id, definition);
    }

    pub(super) fn signature(&self, id: SignatureId) -> Signature<'s> {
        self.signatures[&id].signature
    }

    pub(super) fn function(&self, id: FunctionId) -> Function<'s> {
        self.functions[&id].function
    }

    pub(super) fn signature_definition(&self, id: SignatureId) -> SourceLocation<'s> {
        self.signatures[&id].statement
    }

    pub(super) fn function_definition(&self, id: FunctionId) -> SourceLocation<'s> {
        self.functions[&id].statement
    }
}
