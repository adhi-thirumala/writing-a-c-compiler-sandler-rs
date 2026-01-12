use std::io::Write;

use super::Result;
use super::asm_gen;
use crate::error::Error;

pub(super) fn code_emission(writer: &mut impl Write, root: asm_gen::Program) -> Result<()> {
    let result = root.emit(writer);
    writer.flush()?;
    result
}

trait CodeEmitter {
    fn emit(&self, writer: &mut impl Write) -> Result<()>;
}

impl CodeEmitter for asm_gen::Program {
    fn emit(&self, writer: &mut impl Write) -> Result<()> {
        match self {
            asm_gen::Program::Program(function_definition) => {
                function_definition.emit(writer)?;
                #[cfg(target_os = "linux")]
                writeln!(writer, ".section .note.GNU-stack,\"\",@progbits")?;
                Ok(())
            }
        }
    }
}

impl CodeEmitter for asm_gen::FunctionDefinition {
    fn emit(&self, writer: &mut impl Write) -> Result<()> {
        match self {
            asm_gen::FunctionDefinition::Function { name, instructions } => {
                writeln!(writer, ".globl {}", name)?;
                writeln!(writer, "{}:", name)?;
                writeln!(writer, "  pushq %rbp\n  movq %rsp, %rbp")?;
                instructions
                    .into_iter()
                    .try_for_each(|instruction| instruction.emit(writer))?
            }
        }
        Ok(())
    }
}

impl CodeEmitter for asm_gen::Instruction {
    fn emit(&self, writer: &mut impl Write) -> Result<()> {
        match self {
            asm_gen::Instruction::Mov { src, dst } => {
                write!(writer, "  movl ")?;
                src.emit(writer)?;
                write!(writer, ", ")?;
                dst.emit(writer)?;
                write!(writer, "\n")?;
            }
            asm_gen::Instruction::Ret => writeln!(writer, "  movq %rbp, %rsp\n  popq %rbp\n  ret")?,
            asm_gen::Instruction::AllocateStack(offset) => {
                writeln!(writer, "  subq ${}, %rsp", offset)?
            }
            asm_gen::Instruction::Unary {
                unary_operator,
                operand,
            } => {
                unary_operator.emit(writer)?;
                write!(writer, " ")?;
                operand.emit(writer)?;
                write!(writer, "\n")?;
            }
        }
        Ok(())
    }
}

impl CodeEmitter for asm_gen::Operand {
    fn emit(&self, writer: &mut impl Write) -> Result<()> {
        match self {
            asm_gen::Operand::Imm(val) => write!(writer, "${}", val)?,
            asm_gen::Operand::Register(register) => match register {
                asm_gen::Register::AX => write!(writer, "%eax")?,
                asm_gen::Register::R10 => write!(writer, "%r10d")?,
            },
            asm_gen::Operand::Psuedo(_) => {
                return Err(Error::CodeEmissionError(
                    "found a pseudo-operator, not supposed to",
                ));
            }
            asm_gen::Operand::Stack(offset) => write!(writer, "{}(%rbp)", offset)?,
        }
        Ok(())
    }
}

impl CodeEmitter for asm_gen::UnaryOperator {
    fn emit(&self, writer: &mut impl Write) -> Result<()> {
        match self {
            asm_gen::UnaryOperator::Neg => write!(writer, "  negl")?,
            asm_gen::UnaryOperator::Not => write!(writer, "  notl")?,
        }
        Ok(())
    }
}
