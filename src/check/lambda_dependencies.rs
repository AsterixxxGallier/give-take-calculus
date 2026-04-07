use crate::check::{
    FunctionValue, LambdaDependencies, LambdaDependencyValues, SignatureValue,
    UnknownFunctionValue, UnknownSignatureValue,
};

impl<'s> LambdaDependencies<'s> {
    pub(crate) fn is_empty(&self) -> bool {
        self.signatures.is_empty() && self.functions.is_empty()
    }

    pub(crate) fn as_values(&self) -> LambdaDependencyValues<'s> {
        LambdaDependencyValues {
            signatures: self
                .signatures
                .iter()
                .map(|&dependency| {
                    (
                        dependency,
                        SignatureValue::Unknown(UnknownSignatureValue::Taken(dependency)),
                    )
                })
                .collect(),
            functions: self
                .functions
                .iter()
                .map(|(&dependency, signature)| {
                    (
                        dependency,
                        FunctionValue::Unknown(UnknownFunctionValue::Taken(
                            dependency,
                            signature.clone(),
                        )),
                    )
                })
                .collect(),
        }
    }

    pub(super) fn add(&mut self, other: &Self) {
        for &signature in &other.signatures {
            self.signatures.insert(signature);
        }

        for (&function, signature) in &other.functions {
            if self.functions.contains_key(&function) {
                // can this ever happen?
                // if so: should we allow one function to be constrained by multiple signatures?
                assert!(&self.functions[&function] == signature);
            } else {
                self.functions.insert(function, signature.clone());
            }
        }
    }
}
