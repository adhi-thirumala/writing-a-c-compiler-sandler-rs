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
                #[cfg(target_os = "linux")]
                writeln!(writer, "{}:", name)?;
                #[cfg(target_os = "macos")]
                writeln!(writer, "_{}:", name)?;

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
            asm_gen::Instruction::Binary {
                binary_operator:
                    binary_operator @ (asm_gen::BinaryOperator::LeftShift
                    | asm_gen::BinaryOperator::RightShift),
                left_operand: left_operand @ asm_gen::Operand::Register(asm_gen::Register::CX),
                right_operand,
            } => {
                binary_operator.emit(writer)?;
                write!(writer, " ")?;
                left_operand.emit_one_byte(writer)?;
                write!(writer, ", ")?;
                right_operand.emit(writer)?;
                write!(writer, "\n")?;
            }
            asm_gen::Instruction::Binary {
                binary_operator,
                left_operand,
                right_operand,
            } => {
                binary_operator.emit(writer)?;
                write!(writer, " ")?;
                left_operand.emit(writer)?;
                write!(writer, ", ")?;
                right_operand.emit(writer)?;
                write!(writer, "\n")?;
            }
            asm_gen::Instruction::Idiv(operand) => {
                write!(writer, "  idivl ")?;
                operand.emit(writer)?;
                write!(writer, "\n")?;
            }
            asm_gen::Instruction::Cdq => writeln!(writer, "  cdq")?,
            asm_gen::Instruction::Cmp {
                left_operand,
                right_operand,
            } => {
                write!(writer, "  cmpl ")?;
                left_operand.emit(writer)?;
                write!(writer, ", ")?;
                right_operand.emit(writer)?;
                write!(writer, "\n")?;
            }
            asm_gen::Instruction::Jmp(label) => writeln!(writer, "  jmp .L{}", label)?,
            asm_gen::Instruction::JmpCC {
                cond_code,
                identifier,
            } => {
                write!(writer, "  j")?;
                cond_code.emit(writer)?;
                writeln!(writer, "  .L{}", identifier)?;
            }
            asm_gen::Instruction::SetCC { cond_code, operand } => {
                write!(writer, "  set")?;
                cond_code.emit(writer)?;
                write!(writer, " ")?;
                operand.emit_one_byte(writer)?;
                write!(writer, "\n")?;
            }
            asm_gen::Instruction::Label(label) => write!(writer, ".L{}:", label)?,
        }
        Ok(())
    }
}

impl CodeEmitter for asm_gen::Operand {
    fn emit(&self, writer: &mut impl Write) -> Result<()> {
        match self {
            asm_gen::Operand::Register(register) => match register {
                asm_gen::Register::AX => write!(writer, "%eax")?,
                asm_gen::Register::R10 => write!(writer, "%r10d")?,
                asm_gen::Register::DX => write!(writer, "%edx")?,
                asm_gen::Register::R11 => write!(writer, "%r11d")?,
                asm_gen::Register::CX => write!(writer, "%ecx")?,
            },

            asm_gen::Operand::Imm(val) => write!(writer, "${}", val)?,
            asm_gen::Operand::Stack(offset) => write!(writer, "{}(%rbp)", offset)?,
            asm_gen::Operand::Psuedo(_) => {
                return Err(Error::CodeEmissionError(
                    "found a pseudo-operator, not supposed to",
                ));
            }
        }
        Ok(())
    }
}

impl asm_gen::Operand {
    fn emit_one_byte(&self, writer: &mut impl Write) -> Result<()> {
        match self {
            asm_gen::Operand::Register(register) => match register {
                asm_gen::Register::AX => write!(writer, "%al"),
                asm_gen::Register::R10 => write!(writer, "%r10b"),
                asm_gen::Register::DX => write!(writer, "%dl"),
                asm_gen::Register::R11 => write!(writer, "%rllb"),
                asm_gen::Register::CX => write!(writer, "%cl"),
            }?,
            asm_gen::Operand::Imm(val) => write!(writer, "${}", val)?,
            asm_gen::Operand::Stack(offset) => write!(writer, "{}(%rbp)", offset)?,
            asm_gen::Operand::Psuedo(_) => {
                return Err(Error::CodeEmissionError(
                    "found a pseudo-operator, not supposed to",
                ));
            }
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

impl CodeEmitter for asm_gen::BinaryOperator {
    fn emit(&self, writer: &mut impl Write) -> Result<()> {
        match self {
            asm_gen::BinaryOperator::Add => write!(writer, "  addl")?,
            asm_gen::BinaryOperator::Sub => write!(writer, "  subl")?,
            asm_gen::BinaryOperator::Mult => write!(writer, "  imull")?,
            asm_gen::BinaryOperator::BitwiseAnd => write!(writer, "  andl")?,
            asm_gen::BinaryOperator::BitwiseOr => write!(writer, "  orl")?,
            asm_gen::BinaryOperator::BitwiseXor => write!(writer, "  xorl")?,
            asm_gen::BinaryOperator::LeftShift => write!(writer, "  shll")?,
            asm_gen::BinaryOperator::RightShift => write!(writer, "  sarl")?,
        }
        Ok(())
    }
}

impl CodeEmitter for asm_gen::CondCode {
    fn emit(&self, writer: &mut impl Write) -> Result<()> {
        match self {
            asm_gen::CondCode::E => write!(writer, "e")?,
            asm_gen::CondCode::NE => write!(writer, "ne")?,
            asm_gen::CondCode::G => write!(writer, "g")?,
            asm_gen::CondCode::GE => write!(writer, "ge")?,
            asm_gen::CondCode::L => write!(writer, "l")?,
            asm_gen::CondCode::LE => write!(writer, "le")?,
        }
        Ok(())
    }
}
