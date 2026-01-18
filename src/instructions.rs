use std::fmt::Debug;

use crate::{Imm, RegIdx, Shamt};

#[derive(Debug)]
pub enum Instruction {
    // NOTE: RV32I R-Type
    ADD { rd: RegIdx, rs1: RegIdx, rs2: RegIdx },
    SUB { rd: RegIdx, rs1: RegIdx, rs2: RegIdx },
    SLL { rd: RegIdx, rs1: RegIdx, rs2: RegIdx },
    SLT { rd: RegIdx, rs1: RegIdx, rs2: RegIdx },
    SLTU { rd: RegIdx, rs1: RegIdx, rs2: RegIdx },
    XOR { rd: RegIdx, rs1: RegIdx, rs2: RegIdx },
    SRL { rd: RegIdx, rs1: RegIdx, rs2: RegIdx },
    SRA { rd: RegIdx, rs1: RegIdx, rs2: RegIdx },
    OR { rd: RegIdx, rs1: RegIdx, rs2: RegIdx },
    AND { rd: RegIdx, rs1: RegIdx, rs2: RegIdx },
    // NOTE: RV32M
    MUL { rd: RegIdx, rs1: RegIdx, rs2: RegIdx },
    MULH { rd: RegIdx, rs1: RegIdx, rs2: RegIdx },
    MULHSU { rd: RegIdx, rs1: RegIdx, rs2: RegIdx },
    MULHU { rd: RegIdx, rs1: RegIdx, rs2: RegIdx },
    DIV { rd: RegIdx, rs1: RegIdx, rs2: RegIdx },
    DIVU { rd: RegIdx, rs1: RegIdx, rs2: RegIdx },
    REM { rd: RegIdx, rs1: RegIdx, rs2: RegIdx },
    REMU { rd: RegIdx, rs1: RegIdx, rs2: RegIdx },
    // NOTE: RV64I R-Type
    ADDW { rd: RegIdx, rs1: RegIdx, rs2: RegIdx },
    SUBW { rd: RegIdx, rs1: RegIdx, rs2: RegIdx },
    SLLW { rd: RegIdx, rs1: RegIdx, rs2: RegIdx },
    SRLW { rd: RegIdx, rs1: RegIdx, rs2: RegIdx },
    SRAW { rd: RegIdx, rs1: RegIdx, rs2: RegIdx },
    // NOTE: RV64M
    MULW { rd: RegIdx, rs1: RegIdx, rs2: RegIdx },
    DIVW { rd: RegIdx, rs1: RegIdx, rs2: RegIdx },
    DIVUW { rd: RegIdx, rs1: RegIdx, rs2: RegIdx },
    REMW { rd: RegIdx, rs1: RegIdx, rs2: RegIdx },
    REMUW { rd: RegIdx, rs1: RegIdx, rs2: RegIdx },

    // NOTE: RV32I I-Type
    ADDI { rd: RegIdx, rs1: RegIdx, imm: Imm },
    SLTI { rd: RegIdx, rs1: RegIdx, imm: Imm },
    SLTIU { rd: RegIdx, rs1: RegIdx, imm: Imm },
    XORI { rd: RegIdx, rs1: RegIdx, imm: Imm },
    ORI { rd: RegIdx, rs1: RegIdx, imm: Imm },
    ANDI { rd: RegIdx, rs1: RegIdx, imm: Imm },
    SLLI { rd: RegIdx, rs1: RegIdx, shamt: Shamt },
    SRLI { rd: RegIdx, rs1: RegIdx, shamt: Shamt },
    SRAI { rd: RegIdx, rs1: RegIdx, shamt: Shamt },
    // NOTE: RV64I I-Type
    ADDIW { rd: RegIdx, rs1: RegIdx, imm: Imm },
    SLLIW { rd: RegIdx, rs1: RegIdx, shamt: Shamt },
    SRLIW { rd: RegIdx, rs1: RegIdx, shamt: Shamt },
    SRAIW { rd: RegIdx, rs1: RegIdx, shamt: Shamt },
    // NOTE: RV32I I-Type (メモリ操作)
    LB { rd: RegIdx, rs1: RegIdx, offset: Imm },
    LH { rd: RegIdx, rs1: RegIdx, offset: Imm },
    LW { rd: RegIdx, rs1: RegIdx, offset: Imm },
    LBU { rd: RegIdx, rs1: RegIdx, offset: Imm },
    LHU { rd: RegIdx, rs1: RegIdx, offset: Imm },
    // NOTE: RV64I I-Type (メモリ操作)
    LD { rd: RegIdx, rs1: RegIdx, offset: Imm },
    LWU { rd: RegIdx, rs1: RegIdx, offset: Imm },

    // NOTE: RV32I S-Type
    SB { rs1: RegIdx, rs2: RegIdx, offset: Imm },
    SH { rs1: RegIdx, rs2: RegIdx, offset: Imm },
    SW { rs1: RegIdx, rs2: RegIdx, offset: Imm },
    // NOTE: RV64I S-Type
    SD { rs1: RegIdx, rs2: RegIdx, offset: Imm },

    // NOTE: RV32I B-Type
    BEQ { rs1: RegIdx, rs2: RegIdx, offset: Imm },
    BNE { rs1: RegIdx, rs2: RegIdx, offset: Imm },
    BLT { rs1: RegIdx, rs2: RegIdx, offset: Imm },
    BGE { rs1: RegIdx, rs2: RegIdx, offset: Imm },
    BLTU { rs1: RegIdx, rs2: RegIdx, offset: Imm },
    BGEU { rs1: RegIdx, rs2: RegIdx, offset: Imm },

    // NOTE: RV32I U-Type
    LUI { rd: RegIdx, imm: Imm },
    AUIPC { rd: RegIdx, imm: Imm },

    // NOTE: RV32I J-Type
    JAL { rd: RegIdx, offset: Imm },
    JALR { rd: RegIdx, rs1: RegIdx, offset: Imm },

    // NOTE: RV32I System
    EBREAK,
}

pub struct InstructionContext {
    pub instruction: Instruction,
    pub next_pc: u64,
}
impl Debug for InstructionContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?} (Next PC: 0x{:08x})", self.instruction, self.next_pc)
    }
}
