use std::io::Write;

use crate::compile::asm_gen;
use crate::error::Result;

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
        /*
        match self {
            asm_gen::Instruction::Mov { src, dst } => {
                write!(writer, "  movl ")?;
                src.emit(writer)?;
                write!(writer, ", ")?;
                dst.emit(writer)?;
                write!(writer, "\n")?;
            }
            asm_gen::Instruction::Ret => writeln!(writer, "  ret")?,
        }
        Ok(())
        */
        todo!("add matching")
    }
}

impl CodeEmitter for asm_gen::Operand {
    fn emit(&self, writer: &mut impl Write) -> Result<()> {
        /*
        match self {
            asm_gen::Operand::Imm(val) => write!(writer, "${}", val)?,
            asm_gen::Operand::Register => write!(writer, "%eax")?,
            _ => todo!(),
        }
        Ok(())
        */
        todo!()
    }
}
