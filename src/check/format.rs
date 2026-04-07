use crate::check::*;
use ariadne::{Color, Fmt};
use std::io;
use std::io::Write;

pub(super) trait Format<'s> {
    fn format(&self, resolve: &Resolver<'s>, out: &mut IndentingFormatter) -> io::Result<()>;
}

pub(super) struct IndentingFormatter<'a> {
    indentation: usize,
    buf: &'a mut dyn Write,
}

impl<'a> IndentingFormatter<'a> {
    pub(super) fn new(buf: &'a mut dyn Write) -> Self {
        Self {
            indentation: 0,
            buf,
        }
    }

    pub(super) fn new_line(&mut self) -> io::Result<()> {
        writeln!(self.buf)?;
        for _ in 0..self.indentation {
            write!(self.buf, "  ")?;
        }
        Ok(())
    }

    pub(super) fn indented(
        &mut self,
        f: impl FnOnce(&mut Self) -> io::Result<()>,
    ) -> io::Result<()> {
        self.indentation += 1;
        let result = f(self);
        self.indentation -= 1;
        result
    }
}

impl<'a> Write for IndentingFormatter<'a> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.buf.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.buf.flush()
    }
}

macro_rules! write_colored {
    ($buf:expr, $color:expr, $($rest:tt)*) => {
        write!($buf, "{}", format!($($rest)*).fg($color))
    };
}

impl<'s> Format<'s> for KnownSignatureValue<'s> {
    fn format(&self, resolve: &Resolver<'s>, out: &mut IndentingFormatter) -> io::Result<()> {
        write!(out, "known signature")?;
        out.indented(|out| {
            for &signature in &self.taken_signatures {
                out.new_line()?;

                write_colored!(out, Color::Blue, "taking {}", resolve.signature(signature))?;
            }
            for (&function, signature) in &self.taken_functions {
                out.new_line()?;
                write_colored!(out, Color::Blue, "taking {}", resolve.function(function))?;
                write!(out, " of signature ")?;
                signature.format(resolve, out)?;
            }
            for (&signature, conjuration) in &self.conjured_signatures {
                out.new_line()?;
                write_colored!(out, Color::Blue, "conjuring {}: ", resolve.signature(signature))?;
                conjuration.format(resolve, out)?;
            }
            for (&function, conjuration) in &self.conjured_functions {
                out.new_line()?;
                write_colored!(out, Color::Blue, "conjuring {}: ", resolve.function(function))?;
                conjuration.format(resolve, out)?;
            }
            Ok(())
        })?;
        Ok(())
    }
}

impl<'s> Format<'s> for KnownFunctionValue<'s> {
    fn format(&self, resolve: &Resolver<'s>, out: &mut IndentingFormatter) -> io::Result<()> {
        write!(out, "known function")?;
        out.indented(|out| {
            for &signature in &self.taken_signatures {
                out.new_line()?;
                write_colored!(out, Color::Blue, "taking {}", resolve.signature(signature))?;
            }
            for (&function, signature) in &self.taken_functions {
                out.new_line()?;
                write_colored!(out, Color::Blue, "taking {}", resolve.function(function))?;
                write!(out, " of signature ")?;
                signature.format(resolve, out)?;
            }
            for (&signature, lambda) in &self.given_signatures {
                out.new_line()?;
                write_colored!(out, Color::Blue, "giving {}: ", resolve.signature(signature))?;
                lambda.format(resolve, out)?;
            }
            for (&function, lambda) in &self.given_functions {
                out.new_line()?;
                write_colored!(out, Color::Blue, "giving {}: ", resolve.function(function))?;
                lambda.format(resolve, out)?;
            }
            Ok(())
        })?;
        Ok(())
    }
}

impl<'s> Format<'s> for SignatureConjuration<'s> {
    fn format(&self, resolve: &Resolver<'s>, out: &mut IndentingFormatter) -> io::Result<()> {
        write!(out, "{}", "Lambda ".fg(Color::Green))?;
        self.dependencies.format(resolve, out)?;
        write!(out, "{} signature", ".".fg(Color::Green))?;
        Ok(())
    }
}

impl<'s> Format<'s> for FunctionConjuration<'s> {
    fn format(&self, resolve: &Resolver<'s>, out: &mut IndentingFormatter) -> io::Result<()> {
        write!(out, "{}", "Lambda ".fg(Color::Green))?;
        self.dependencies.format(resolve, out)?;
        write!(out, "{} function of signature ", ".".fg(Color::Green))?;
        self.signature.format(resolve, out)?;
        Ok(())
    }
}

impl<'s> Format<'s> for SignatureLambda<'s> {
    fn format(&self, resolve: &Resolver<'s>, out: &mut IndentingFormatter) -> io::Result<()> {
        write!(out, "{}", "Lambda ".fg(Color::Green))?;
        self.dependencies.format(resolve, out)?;
        write!(out, "{} ", ".".fg(Color::Green))?;
        self.signature.format(resolve, out)?;
        Ok(())
    }
}

impl<'s> Format<'s> for FunctionLambda<'s> {
    fn format(&self, resolve: &Resolver<'s>, out: &mut IndentingFormatter) -> io::Result<()> {
        write!(out, "{}", "Lambda ".fg(Color::Green))?;
        self.dependencies.format(resolve, out)?;
        write!(out, "{} ", ".".fg(Color::Green))?;
        self.function.format(resolve, out)?;
        Ok(())
    }
}

impl<'s> Format<'s> for LambdaDependencies<'s> {
    fn format(&self, resolve: &Resolver<'s>, out: &mut IndentingFormatter) -> io::Result<()> {
        if let Some(&first) = self.signatures.first() {
            write!(out, "{}", resolve.signature(first).fg(Color::Green))?;
            for &signature in self.signatures.iter().skip(1) {
                write!(out, " {}", resolve.signature(signature).fg(Color::Green))?;
            }
            for &function in self.functions.keys() {
                write!(out, " {}", resolve.function(function).fg(Color::Green))?;
            }
        } else if let Some(&first) = self.functions.keys().next() {
            write!(out, "{}", resolve.function(first).fg(Color::Green))?;
            for &function in self.functions.keys().skip(1) {
                write!(out, " {}", resolve.function(function).fg(Color::Green))?;
            }
        }
        Ok(())
    }
}

impl<'s> Format<'s> for LambdaDependencyValues<'s> {
    fn format(&self, resolve: &Resolver<'s>, out: &mut IndentingFormatter) -> io::Result<()> {
        for (&signature, value) in &self.signatures {
            out.new_line()?;
            write!(out, "{} = ", resolve.signature(signature))?;
            value.format(resolve, out)?;
        }
        for (&function, value) in &self.functions {
            out.new_line()?;
            write!(out, "{} = ", resolve.function(function))?;
            value.format(resolve, out)?;
        }
        Ok(())
    }
}

impl<'s> Format<'s> for ConjuredSignatureValue<'s> {
    fn format(&self, resolve: &Resolver<'s>, out: &mut IndentingFormatter) -> io::Result<()> {
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
    fn format(&self, resolve: &Resolver<'s>, out: &mut IndentingFormatter) -> io::Result<()> {
        write!(
            out,
            "conjured function {}",
            resolve.function(self.conjured_function)
        )?;
        out.indented(|out| {
            out.new_line()?;
            write!(out, "signature of conjured function: ")?;
            self.conjured_function_signature.format(resolve, out)?;
            out.new_line()?;
            write!(out, "unknown function: ")?;
            self.unknown_function.format(resolve, out)?;
            out.new_line()?;
            write!(out, "dependency values:")?;
            out.indented(|out| self.conjure_dependency_values.format(resolve, out))?;
            Ok(())
        })?;
        Ok(())
    }
}

impl<'s> Format<'s> for UnknownSignatureValue<'s> {
    fn format(&self, resolve: &Resolver<'s>, out: &mut IndentingFormatter) -> io::Result<()> {
        match self {
            &UnknownSignatureValue::Taken(signature) => {
                write!(out, "{}", resolve.signature(signature))
            }
            UnknownSignatureValue::Conjured(conjured) => conjured.format(resolve, out),
        }
    }
}

impl<'s> Format<'s> for UnknownFunctionValue<'s> {
    fn format(&self, resolve: &Resolver<'s>, out: &mut IndentingFormatter) -> io::Result<()> {
        match self {
            &UnknownFunctionValue::Taken(function, _) => {
                write!(out, "{}", resolve.function(function))
            }
            UnknownFunctionValue::Conjured(conjured) => conjured.format(resolve, out),
        }
    }
}

impl<'s> Format<'s> for SignatureValue<'s> {
    fn format(&self, resolve: &Resolver<'s>, out: &mut IndentingFormatter) -> io::Result<()> {
        match self {
            SignatureValue::Known(known) => known.format(resolve, out),
            SignatureValue::Unknown(unknown) => unknown.format(resolve, out),
        }
    }
}

impl<'s> Format<'s> for FunctionValue<'s> {
    fn format(&self, resolve: &Resolver<'s>, out: &mut IndentingFormatter) -> io::Result<()> {
        match self {
            FunctionValue::Known(known) => known.format(resolve, out),
            FunctionValue::Unknown(unknown) => unknown.format(resolve, out),
        }
    }
}
