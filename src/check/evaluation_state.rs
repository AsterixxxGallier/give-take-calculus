use crate::check::error::CheckResult;
use crate::check::{CheckError, EvaluationState, FunctionId, FunctionLambda, LambdaDependencies, SignatureId, SignatureLambda};
use crate::parse::{Function, Signature};
use std::mem;

impl<'s> EvaluationState<'s> {
    // yes i know how fucked up this looks but i've been fighting the borrow checker for half an
    // hour now so please forgive me
    pub(super) fn do_as_child<R>(
        &mut self,
        as_child: impl FnOnce(&mut Self) -> R,
    ) -> (R, LambdaDependencies<'s>) {
        let parent = mem::take(self);
        let mut child = Self {
            parent: Some(Box::new(parent)),
            ..Self::default()
        };
        let result = as_child(&mut child);
        let dependencies = child.dependencies;
        *self = *child.parent.take().unwrap();
        (result, dependencies)
    }

    pub(super) fn resolve_signature(&self, signature: Signature<'s>) -> CheckResult<'s, SignatureId> {
        let mut this = Some(self);
        while let Some(current) = this {
            if let Some(&id) = current.signature_ids.get(signature.as_str()) {
                return Ok(id);
            }
            this = current.parent.as_deref();
        }
        Err(CheckError::CannotResolveSignature { signature })
    }

    pub(super) fn resolve_function(&self, function: Function<'s>) -> CheckResult<'s, FunctionId> {
        let mut this = Some(self);
        while let Some(current) = this {
            if let Some(&id) = current.function_ids.get(function.as_str()) {
                return Ok(id);
            }
            this = current.parent.as_deref();
        }
        Err(CheckError::CannotResolveFunction { function })
    }

    pub(super) fn signature_lambda(&mut self, signature: SignatureId) -> SignatureLambda<'s> {
        if let Some(lambda) = self.signature_lambdas.get(&signature) {
            return lambda.clone();
        }

        let mut this = Some(self);
        while let Some(current) = this {
            if let Some(parent) = current.parent.as_deref_mut() {
                if let Some(lambda) = parent.signature_lambdas.get(&signature) {
                    current.dependencies.add(&lambda.dependencies);
                    return SignatureLambda {
                        signature: lambda.signature.clone(),
                        dependencies: Default::default(),
                    };
                }
            }
            this = current.parent.as_deref_mut();
        }

        panic!("could not get lambda for signature");
    }

    pub(super) fn function_lambda(&mut self, function: FunctionId) -> FunctionLambda<'s> {
        if let Some(lambda) = self.function_lambdas.get(&function) {
            return lambda.clone();
        }

        let mut this = Some(self);
        while let Some(current) = this {
            if let Some(parent) = current.parent.as_deref_mut() {
                if let Some(lambda) = parent.function_lambdas.get(&function) {
                    current.dependencies.add(&lambda.dependencies);
                    return FunctionLambda {
                        function: lambda.function.clone(),
                        dependencies: Default::default(),
                    };
                }
            }
            this = current.parent.as_deref_mut();
        }

        panic!("could not get lambda for function");
    }
}