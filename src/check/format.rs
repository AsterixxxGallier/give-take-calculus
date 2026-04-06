use crate::check::{
    ConjuredFunctionValue, ConjuredSignatureValue, EvaluationState, FunctionConjuration,
    FunctionId, FunctionLambda, FunctionValue, KnownFunctionValue, KnownSignatureValue,
    LambdaDependencies, LambdaDependencyValues, SignatureConjuration, SignatureId, SignatureLambda,
    SignatureValue, UnknownFunctionValue, UnknownSignatureValue,
};
use crate::parse::{Function, Signature};
use std::fmt;
use std::fmt::{Arguments, Display, Formatter, Write};

pub(crate) struct FormatAsDisplay<'a, 's>(&'a dyn Format<'s>, &'a dyn Resolve<'s>);

impl<'a, 's> Display for FormatAsDisplay<'a, 's> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.0.format(self.1, &mut IndentingFormatter::new(f))
    }
}

pub(crate) trait Format<'s> {
    fn format(&self, resolve: &dyn Resolve<'s>, out: &mut IndentingFormatter) -> fmt::Result;
}

pub(crate) trait Resolve<'s> {
    fn signature(&self, id: SignatureId) -> Signature<'s>;

    fn function(&self, id: FunctionId) -> Function<'s>;
}

impl<'s> Resolve<'s> for EvaluationState<'s> {
    fn signature(&self, id: SignatureId) -> Signature<'s> {
        self.signature_names[&id]
    }

    fn function(&self, id: FunctionId) -> Function<'s> {
        self.function_names[&id]
    }
}

pub(crate) struct IndentingFormatter<'a> {
    indentation: usize,
    buf: &'a mut dyn Write,
}

impl<'a> IndentingFormatter<'a> {
    pub(crate) fn new(buf: &'a mut dyn Write) -> Self {
        Self {
            indentation: 0,
            buf,
        }
    }

    pub(crate) fn new_line(&mut self) -> fmt::Result {
        self.buf.write_char('\n')?;
        for _ in 0..self.indentation {
            self.buf.write_str("  ")?;
        }
        Ok(())
    }

    pub(crate) fn indented(&mut self, f: impl FnOnce(&mut Self) -> fmt::Result) -> fmt::Result {
        self.indentation += 1;
        let result = f(self);
        self.indentation -= 1;
        result
    }
}

impl<'a> Write for IndentingFormatter<'a> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.buf.write_str(s)
    }

    fn write_char(&mut self, c: char) -> fmt::Result {
        self.buf.write_char(c)
    }

    fn write_fmt(&mut self, args: Arguments<'_>) -> fmt::Result {
        self.buf.write_fmt(args)
    }
}

impl<'s> Format<'s> for KnownSignatureValue<'s> {
    fn format(&self, resolve: &dyn Resolve<'s>, out: &mut IndentingFormatter) -> fmt::Result {
        write!(out, "known signature")?;
        out.indented(|out| {
            for &signature in self.taken_signature_ids.values() {
                out.new_line()?;
                write!(out, "taking {}", resolve.signature(signature))?;
            }
            for (&function, signature) in &self.taken_function_signatures {
                out.new_line()?;
                write!(out, "taking {} of signature ", resolve.function(function))?;
                signature.format(resolve, out)?;
            }
            for (&signature, conjuration) in &self.conjured_signatures {
                out.new_line()?;
                write!(out, "{} => ", resolve.signature(signature))?;
                conjuration.format(resolve, out)?;
            }
            for (&function, conjuration) in &self.conjured_functions {
                out.new_line()?;
                write!(out, "{} => ", resolve.function(function))?;
                conjuration.format(resolve, out)?;
            }
            Ok(())
        })?;
        Ok(())
    }
}

impl<'s> Format<'s> for KnownFunctionValue<'s> {
    fn format(&self, resolve: &dyn Resolve<'s>, out: &mut IndentingFormatter) -> fmt::Result {
        write!(out, "known function")?;
        out.indented(|out| {
            for &signature in self.taken_signature_ids.values() {
                out.new_line()?;
                write!(out, "taking {}", resolve.signature(signature))?;
            }
            for (&function, signature) in &self.taken_function_signatures {
                out.new_line()?;
                write!(out, "taking {} of signature ", resolve.function(function))?;
                signature.format(resolve, out)?;
            }
            for (&signature, lambda) in &self.given_signatures {
                out.new_line()?;
                write!(out, "{} => ", resolve.signature(signature))?;
                lambda.format(resolve, out)?;
            }
            for (&function, lambda) in &self.given_functions {
                out.new_line()?;
                write!(out, "{} => ", resolve.function(function))?;
                lambda.format(resolve, out)?;
            }
            Ok(())
        })?;
        Ok(())
    }
}

impl<'s> Format<'s> for SignatureConjuration<'s> {
    fn format(&self, resolve: &dyn Resolve<'s>, out: &mut IndentingFormatter) -> fmt::Result {
        write!(out, "Lambda ")?;
        self.dependencies.format(resolve, out)?;
        write!(out, ". conjured signature")?;
        Ok(())
    }
}

impl<'s> Format<'s> for FunctionConjuration<'s> {
    fn format(&self, resolve: &dyn Resolve<'s>, out: &mut IndentingFormatter) -> fmt::Result {
        write!(out, "Lambda ")?;
        self.dependencies.format(resolve, out)?;
        write!(out, ". conjured function of signature ")?;
        self.signature.format(resolve, out)?;
        Ok(())
    }
}

impl<'s> Format<'s> for SignatureLambda<'s> {
    fn format(&self, resolve: &dyn Resolve<'s>, out: &mut IndentingFormatter) -> fmt::Result {
        write!(out, "Lambda ")?;
        self.dependencies.format(resolve, out)?;
        write!(out, ". ")?;
        self.signature.format(resolve, out)?;
        Ok(())
    }
}

impl<'s> Format<'s> for FunctionLambda<'s> {
    fn format(&self, resolve: &dyn Resolve<'s>, out: &mut IndentingFormatter) -> fmt::Result {
        write!(out, "Lambda ")?;
        self.dependencies.format(resolve, out)?;
        write!(out, ". ")?;
        self.function.format(resolve, out)?;
        Ok(())
    }
}

impl<'s> Format<'s> for LambdaDependencies<'s> {
    fn format(&self, resolve: &dyn Resolve<'s>, out: &mut IndentingFormatter) -> fmt::Result {
        if let Some(&first) = self.signatures.first() {
            write!(out, "{}", resolve.signature(first))?;
            for &signature in self.signatures.iter().skip(1) {
                write!(out, " {}", resolve.signature(signature))?;
            }
            for &function in self.functions.keys().skip(1) {
                write!(out, " {}", resolve.function(function))?;
            }
        } else if let Some(&first) = self.functions.keys().next() {
            write!(out, "{}", resolve.function(first))?;
            for &function in self.functions.keys().skip(1) {
                write!(out, " {}", resolve.function(function))?;
            }
        }
        Ok(())
    }
}

impl<'s> Format<'s> for LambdaDependencyValues<'s> {
    fn format(&self, resolve: &dyn Resolve<'s>, out: &mut IndentingFormatter) -> fmt::Result {
        for (&signature, value) in self.signatures.iter().skip(1) {
            out.new_line()?;
            write!(out, "{} = ", resolve.signature(signature))?;
            value.format(resolve, out)?;
        }
        for (&function, value) in self.functions.iter().skip(1) {
            out.new_line()?;
            write!(out, "{} = ", resolve.function(function))?;
            value.format(resolve, out)?;
        }
        Ok(())
    }
}

impl<'s> Format<'s> for ConjuredSignatureValue<'s> {
    fn format(&self, resolve: &dyn Resolve<'s>, out: &mut IndentingFormatter) -> fmt::Result {
        write!(
            out,
            "conjured signature {}",
            resolve.signature(self.conjured_signature)
        )?;
        out.indented(|out| {
            out.new_line()?;
            write!(out, "unknown function: ")?;
            self.unknown_function.format(resolve, out)?;
            write!(out, "dependency values:")?;
            out.indented(|out| self.conjure_dependency_values.format(resolve, out))?;
            Ok(())
        })?;
        Ok(())
    }
}

impl<'s> Format<'s> for ConjuredFunctionValue<'s> {
    fn format(&self, resolve: &dyn Resolve<'s>, out: &mut IndentingFormatter) -> fmt::Result {
        write!(
            out,
            "conjured function {}",
            resolve.function(self.conjured_function)
        )?;
        out.indented(|out| {
            out.new_line()?;
            write!(out, "unknown function: ")?;
            self.unknown_function.format(resolve, out)?;
            write!(out, "dependency values:")?;
            out.indented(|out| self.conjure_dependency_values.format(resolve, out))?;
            Ok(())
        })?;
        Ok(())
    }
}

impl<'s> Format<'s> for UnknownSignatureValue<'s> {
    fn format(&self, resolve: &dyn Resolve<'s>, out: &mut IndentingFormatter) -> fmt::Result {
        match self {
            &UnknownSignatureValue::Taken(signature) => {
                write!(out, "{}", resolve.signature(signature))
            }
            UnknownSignatureValue::Conjured(conjured) => conjured.format(resolve, out),
        }
    }
}

impl<'s> Format<'s> for UnknownFunctionValue<'s> {
    fn format(&self, resolve: &dyn Resolve<'s>, out: &mut IndentingFormatter) -> fmt::Result {
        match self {
            &UnknownFunctionValue::Taken(function, _) => {
                write!(out, "{}", resolve.function(function))
            }
            UnknownFunctionValue::Conjured(conjured) => conjured.format(resolve, out),
        }
    }
}

impl<'s> Format<'s> for SignatureValue<'s> {
    fn format(&self, resolve: &dyn Resolve<'s>, out: &mut IndentingFormatter) -> fmt::Result {
        match self {
            SignatureValue::Known(known) => known.format(resolve, out),
            SignatureValue::Unknown(unknown) => unknown.format(resolve, out),
        }
    }
}

impl<'s> Format<'s> for FunctionValue<'s> {
    fn format(&self, resolve: &dyn Resolve<'s>, out: &mut IndentingFormatter) -> fmt::Result {
        match self {
            FunctionValue::Known(known) => known.format(resolve, out),
            FunctionValue::Unknown(unknown) => unknown.format(resolve, out),
        }
    }
}
