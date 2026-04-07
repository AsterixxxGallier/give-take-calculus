use crate::check::error::CheckResult;
use crate::check::{CheckError, FunctionDefinition, FunctionId, SignatureDefinition, SignatureId};
use crate::parse::{Function, Signature, SourceLocation};
use ordermap::OrderMap;

#[derive(Clone, Default, Eq, PartialEq)]
pub(super) struct ContextResolver<'s> {
    taken_signature_definitions: OrderMap<SignatureId, SignatureDefinition<'s>>,
    taken_function_definitions: OrderMap<FunctionId, FunctionDefinition<'s>>,
    taken_signature_ids: OrderMap<&'s str, SignatureId>,
    taken_function_ids: OrderMap<&'s str, FunctionId>,
    produced_signature_definitions: OrderMap<SignatureId, SignatureDefinition<'s>>,
    produced_function_definitions: OrderMap<FunctionId, FunctionDefinition<'s>>,
    produced_signature_ids: OrderMap<&'s str, SignatureId>,
    produced_function_ids: OrderMap<&'s str, FunctionId>,
}

#[allow(unused)]
impl<'s> ContextResolver<'s> {
    pub(super) fn insert_taken_signature(
        &mut self,
        id: SignatureId,
        signature: Signature<'s>,
        statement: SourceLocation<'s>,
    ) -> CheckResult<'s, ()> {
        if let Some(other_id) = self.taken_signature_id(signature.as_str()) {
            let other_definition = self.taken_signature_definitions[&other_id];
            return Err(CheckError::CannotTakeTwoSignaturesWithIdenticalName {
                signature,
                statement,
                other_signature: other_definition.signature,
                other_statement: other_definition.statement,
            });
        }

        self.taken_signature_ids.insert(signature.as_str(), id);
        let definition = SignatureDefinition {
            signature,
            statement,
        };
        self.taken_signature_definitions.insert(id, definition);
        Ok(())
    }

    pub(super) fn insert_taken_function(
        &mut self,
        id: FunctionId,
        function: Function<'s>,
        statement: SourceLocation<'s>,
    ) -> CheckResult<'s, ()> {
        if let Some(other_id) = self.taken_function_id(function.as_str()) {
            let other_definition = self.taken_function_definitions[&other_id];
            return Err(CheckError::CannotTakeTwoFunctionsWithIdenticalName {
                function,
                statement,
                other_function: other_definition.function,
                other_statement: other_definition.statement,
            });
        }

        self.taken_function_ids.insert(function.as_str(), id);
        let definition = FunctionDefinition {
            function,
            statement,
        };
        self.taken_function_definitions.insert(id, definition);
        Ok(())
    }

    pub(super) fn insert_produced_signature(
        &mut self,
        id: SignatureId,
        signature: Signature<'s>,
        statement: SourceLocation<'s>,
    ) {
        self.produced_signature_ids.insert(signature.as_str(), id);
        let definition = SignatureDefinition {
            signature,
            statement,
        };
        self.produced_signature_definitions.insert(id, definition);
    }

    pub(super) fn insert_produced_function(
        &mut self,
        id: FunctionId,
        function: Function<'s>,
        statement: SourceLocation<'s>,
    ) {
        self.produced_function_ids.insert(function.as_str(), id);
        let definition = FunctionDefinition {
            function,
            statement,
        };
        self.produced_function_definitions.insert(id, definition);
    }

    pub(super) fn remove_taken_signature(&mut self, id: SignatureId) {
        let definition = self.taken_signature_definitions.remove(&id).unwrap();
        self.taken_signature_ids
            .remove(definition.signature.as_str());
    }

    pub(super) fn remove_taken_function(&mut self, id: FunctionId) {
        let definition = self.taken_function_definitions.remove(&id).unwrap();
        self.taken_function_ids
            .remove(definition.function.as_str());
    }

    pub(super) fn taken_signature_id(&self, name: &'s str) -> Option<SignatureId> {
        self.taken_signature_ids.get(name).copied()
    }

    pub(super) fn taken_function_id(&self, name: &'s str) -> Option<FunctionId> {
        self.taken_function_ids.get(name).copied()
    }

    pub(super) fn produced_signature_id(&self, name: &'s str) -> Option<SignatureId> {
        self.produced_signature_ids.get(name).copied()
    }

    pub(super) fn produced_function_id(&self, name: &'s str) -> Option<FunctionId> {
        self.produced_function_ids.get(name).copied()
    }

    pub(super) fn taken_signature(&self, id: SignatureId) -> Signature<'s> {
        self.taken_signature_definitions[&id].signature
    }

    pub(super) fn taken_signature_name(&self, id: SignatureId) -> &'s str {
        self.taken_signature_definitions[&id].signature.as_str()
    }

    pub(super) fn taken_signature_definition(&self, id: SignatureId) -> SourceLocation<'s> {
        self.taken_signature_definitions[&id].statement
    }

    pub(super) fn taken_function(&self, id: FunctionId) -> Function<'s> {
        self.taken_function_definitions[&id].function
    }

    pub(super) fn taken_function_name(&self, id: FunctionId) -> &'s str {
        self.taken_function_definitions[&id].function.as_str()
    }

    pub(super) fn taken_function_definition(&self, id: FunctionId) -> SourceLocation<'s> {
        self.taken_function_definitions[&id].statement
    }

    pub(super) fn produced_signature(&self, id: SignatureId) -> Signature<'s> {
        self.produced_signature_definitions[&id].signature
    }

    pub(super) fn produced_signature_name(&self, id: SignatureId) -> &'s str {
        self.produced_signature_definitions[&id].signature.as_str()
    }

    pub(super) fn produced_signature_definition(&self, id: SignatureId) -> SourceLocation<'s> {
        self.produced_signature_definitions[&id].statement
    }

    pub(super) fn produced_function(&self, id: FunctionId) -> Function<'s> {
        self.produced_function_definitions[&id].function
    }

    pub(super) fn produced_function_name(&self, id: FunctionId) -> &'s str {
        self.produced_function_definitions[&id].function.as_str()
    }

    pub(super) fn produced_function_definition(&self, id: FunctionId) -> SourceLocation<'s> {
        self.produced_function_definitions[&id].statement
    }
}
