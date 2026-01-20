use crate::{Exception, Imm, Instruction, RawInstruction, RawShortInstruction, RegIdx, Shamt};

/// 命令をデコードします。
pub fn decode(instruction: RawInstruction) -> Result<Instruction, Exception> {
    let opcode = instruction & 0b111_1111;
    let rd = ((instruction >> 7) & 0b1_1111) as RegIdx; // 宛先レジスタ
    let funct3 = (instruction >> 12) & 0b111; // 細分類その1
    let rs1 = ((instruction >> 15) & 0b1_1111) as RegIdx; // ソースレジスタ1
    let rs2 = ((instruction >> 20) & 0b1_1111) as RegIdx; // ソースレジスタ2
    let funct7 = (instruction >> 25) & 0b111_1111; // 細分類その2

    Ok(match opcode {
        0b01100_11 => match (funct7, funct3) {
            // NOTE: RV32I R-Type
            (0b00000_00, 0b000) => Ok(Instruction::ADD { rd, rs1, rs2 }),
            (0b01000_00, 0b000) => Ok(Instruction::SUB { rd, rs1, rs2 }),
            (0b00000_00, 0b001) => Ok(Instruction::SLL { rd, rs1, rs2 }),
            (0b00000_00, 0b010) => Ok(Instruction::SLT { rd, rs1, rs2 }),
            (0b00000_00, 0b011) => Ok(Instruction::SLTU { rd, rs1, rs2 }),
            (0b00000_00, 0b100) => Ok(Instruction::XOR { rd, rs1, rs2 }),
            (0b00000_00, 0b101) => Ok(Instruction::SRL { rd, rs1, rs2 }),
            (0b01000_00, 0b101) => Ok(Instruction::SRA { rd, rs1, rs2 }),
            (0b00000_00, 0b110) => Ok(Instruction::OR { rd, rs1, rs2 }),
            (0b00000_00, 0b111) => Ok(Instruction::AND { rd, rs1, rs2 }),

            // NOTE: RV32M R-Type
            (0b00000_01, 0b000) => Ok(Instruction::MUL { rd, rs1, rs2 }),
            (0b00000_01, 0b001) => Ok(Instruction::MULH { rd, rs1, rs2 }),
            (0b00000_01, 0b010) => Ok(Instruction::MULHSU { rd, rs1, rs2 }),
            (0b00000_01, 0b011) => Ok(Instruction::MULHU { rd, rs1, rs2 }),
            (0b00000_01, 0b100) => Ok(Instruction::DIV { rd, rs1, rs2 }),
            (0b00000_01, 0b101) => Ok(Instruction::DIVU { rd, rs1, rs2 }),
            (0b00000_01, 0b110) => Ok(Instruction::REM { rd, rs1, rs2 }),
            (0b00000_01, 0b111) => Ok(Instruction::REMU { rd, rs1, rs2 }),

            _ => Err(Exception::UnknownInstruction(instruction)),
        },
        0b01110_11 => match (funct7, funct3) {
            // NOTE: RV64I R-Type
            (0b00000_00, 0b000) => Ok(Instruction::ADDW { rd, rs1, rs2 }),
            (0b01000_00, 0b000) => Ok(Instruction::SUBW { rd, rs1, rs2 }),
            (0b00000_00, 0b001) => Ok(Instruction::SLLW { rd, rs1, rs2 }),
            (0b00000_00, 0b101) => Ok(Instruction::SRLW { rd, rs1, rs2 }),
            (0b01000_00, 0b101) => Ok(Instruction::SRAW { rd, rs1, rs2 }),

            // NOTE: RV64M R-Type
            (0b00000_01, 0b000) => Ok(Instruction::MULW { rd, rs1, rs2 }),
            (0b00000_01, 0b100) => Ok(Instruction::DIVW { rd, rs1, rs2 }),
            (0b00000_01, 0b101) => Ok(Instruction::DIVUW { rd, rs1, rs2 }),
            (0b00000_01, 0b110) => Ok(Instruction::REMW { rd, rs1, rs2 }),
            (0b00000_01, 0b111) => Ok(Instruction::REMUW { rd, rs1, rs2 }),

            _ => Err(Exception::UnknownInstruction(instruction)),
        },

        // NOTE: RV32I I-Type
        0b00100_11 => {
            let imm = ((instruction as i32) >> 20) as Imm;
            let shamt = ((instruction >> 20) & 0b111111) as Shamt; // NOTE: RV64 では、shamt は 6bit
            match funct3 {
                0b000 => Ok(Instruction::ADDI { rd, rs1, imm }),
                0b010 => Ok(Instruction::SLTI { rd, rs1, imm }),
                0b011 => Ok(Instruction::SLTIU { rd, rs1, imm }),
                0b100 => Ok(Instruction::XORI { rd, rs1, imm }),
                0b110 => Ok(Instruction::ORI { rd, rs1, imm }),
                0b111 => Ok(Instruction::ANDI { rd, rs1, imm }),
                0b001 => Ok(Instruction::SLLI { rd, rs1, shamt }),
                0b101 => Ok(if imm & 0b10000000000 == 0 {
                    Instruction::SRLI { rd, rs1, shamt }
                } else {
                    Instruction::SRAI { rd, rs1, shamt }
                }),

                _ => Err(Exception::UnknownInstruction(instruction)),
            }
        },
        // NOTE: RV64I I-Type
        0b00110_11 => {
            let imm = ((instruction as i32) >> 20) as Imm;
            let shamt = ((instruction >> 20) & 0b11111) as Shamt; // NOTE: RV64 の W 命令の shamt は 5bit
            match funct3 {
                0b000 => Ok(Instruction::ADDIW { rd, rs1, imm }),
                0b001 => Ok(Instruction::SLLIW { rd, rs1, shamt }),
                0b101 => match funct7 {
                    0b00000_00 => Ok(Instruction::SRLIW { rd, rs1, shamt }),
                    0b01000_00 => Ok(Instruction::SRAIW { rd, rs1, shamt }),

                    _ => Err(Exception::UnknownInstruction(instruction)),
                },

                _ => Err(Exception::UnknownInstruction(instruction)),
            }
        },

        // NOTE: RV32/64I I-Type (メモリ操作)
        0b00000_11 => {
            let offset = ((instruction as i32) >> 20) as Imm;
            match funct3 {
                // NOTE: RV32I I-Type (メモリ操作)
                0b000 => Ok(Instruction::LB { rd, rs1, offset }),
                0b001 => Ok(Instruction::LH { rd, rs1, offset }),
                0b010 => Ok(Instruction::LW { rd, rs1, offset }),
                0b100 => Ok(Instruction::LBU { rd, rs1, offset }),
                0b101 => Ok(Instruction::LHU { rd, rs1, offset }),

                // NOTE: RV64I I-Type (メモリ操作)
                0b011 => Ok(Instruction::LD { rd, rs1, offset }),
                0b110 => Ok(Instruction::LWU { rd, rs1, offset }),

                _ => Err(Exception::UnknownInstruction(instruction)),
            }
        },

        // NOTE: RV32/64I S-Type
        0b01000_11 => {
            // NOTE: imm[11:5] + imm[4:0] を結合して符号拡張
            let imm11_5 = (instruction >> 25) & 0x7f;
            let imm4_0 = (instruction >> 7) & 0x1f;
            let imm12 = (imm11_5 << 5) | imm4_0;
            // NOTE: 12bitを符号拡張: 20bit左シフトしてi32へキャストし、右シフトで戻す
            let offset = (((imm12 as i32) << 20) >> 20) as Imm;
            match funct3 {
                // NOTE: RV32I S-Type
                0b000 => Ok(Instruction::SB { rs1, rs2, offset }),
                0b001 => Ok(Instruction::SH { rs1, rs2, offset }),
                0b010 => Ok(Instruction::SW { rs1, rs2, offset }),

                // NOTE: RV64I S-Type
                0b011 => Ok(Instruction::SD { rs1, rs2, offset }),

                _ => Err(Exception::UnknownInstruction(instruction)),
            }
        },

        // NOTE: RV32I B-Type
        0b11000_11 => {
            let imm12 = (instruction >> 31) & 1;
            let imm10_5 = (instruction >> 25) & 0x3f;
            let imm4_1 = (instruction >> 8) & 0xf;
            let imm11 = (instruction >> 7) & 1;
            let imm13 = (imm12 << 12) | (imm11 << 11) | (imm10_5 << 5) | (imm4_1 << 1);
            // NOTE: 13bitを符号拡張
            let offset = (((imm13 as i32) << 19) >> 19) as Imm;

            match funct3 {
                0b000 => Ok(Instruction::BEQ { rs1, rs2, offset }),
                0b001 => Ok(Instruction::BNE { rs1, rs2, offset }),
                0b100 => Ok(Instruction::BLT { rs1, rs2, offset }),
                0b101 => Ok(Instruction::BGE { rs1, rs2, offset }),
                0b110 => Ok(Instruction::BLTU { rs1, rs2, offset }),
                0b111 => Ok(Instruction::BGEU { rs1, rs2, offset }),

                _ => Err(Exception::UnknownInstruction(instruction)),
            }
        },

        // NOTE: RV32I U-Type
        0b01101_11 => Ok(Instruction::LUI { rd, imm: (instruction as i32 & 0xfffff000u32 as i32) as Imm }),
        0b00101_11 => Ok(Instruction::AUIPC { rd, imm: (instruction as i32 & 0xfffff000u32 as i32) as Imm }),

        // NOTE: RV32I J-Type
        0b11011_11 => {
            let imm20 = (instruction >> 31) & 1;
            let imm10_1 = (instruction >> 21) & 0x3ff;
            let imm11 = (instruction >> 20) & 1;
            let imm19_12 = (instruction >> 12) & 0xff;
            let imm21 = (imm20 << 20) | (imm19_12 << 12) | (imm11 << 11) | (imm10_1 << 1);
            // NOTE: 21bitを符号拡張
            let offset = (((imm21 as i32) << 11) >> 11) as Imm;

            Ok(Instruction::JAL { rd, offset })
        },
        0b11001_11 => {
            // NOTE: JALRはフォーマット上は I-Type と同じ
            let offset = ((instruction as i32) >> 20) as Imm;
            match funct3 {
                0b000 => Ok(Instruction::JALR { rd, rs1, offset }),

                _ => Err(Exception::UnknownInstruction(instruction)),
            }
        },

        // NOTE: RV32I System
        0b11100_11 => {
            let funct3 = (instruction >> 12) & 0b111;
            let csr = ((instruction >> 20) & 0b1111_1111_1111) as u16;

            match funct3 {
                0b000 => match csr {
                    0b00000_00_00000 => Ok(Instruction::ECALL),
                    0b00000_00_00001 => Ok(Instruction::EBREAK),

                    _ => Err(Exception::UnknownInstruction(instruction)),
                },

                0b001 => Ok(Instruction::CSRRW { rd, rs1, csr }),
                0b010 => Ok(Instruction::CSRRS { rd, rs1, csr }),
                0b011 => Ok(Instruction::CSRRC { rd, rs1, csr }),
                0b101 => Ok(Instruction::CSRRWI { rd, imm: rs1 as u8, csr }),
                0b110 => Ok(Instruction::CSRRSI { rd, imm: rs1 as u8, csr }),
                0b111 => Ok(Instruction::CSRRCI { rd, imm: rs1 as u8, csr }),

                _ => Err(Exception::UnknownInstruction(instruction)),
            }
        },

        _ => Err(Exception::UnknownInstruction(instruction)),
    }?)
}

/// 圧縮命令をデコードします。
pub fn decode_compressed(instruction: RawShortInstruction) -> Result<Instruction, Exception> {
    let opcode = instruction & 0b11;
    let funct3 = (instruction >> 13) & 0b111;

    // NOTE: 圧縮命令のレジスタ表現 rd', rs1', rs2' を通常のレジスタ番号に変換するクロージャ
    let to_register = |x: u16| ((x & 0b111) + 8) as RegIdx;
    // NOTE: 5bit レジスタをそのまま使うクロージャ
    let as_register = |x: u16| (x & 0b1_1111) as RegIdx;

    Ok(match opcode {
        0b00 => match funct3 {
            // NOTE: C.ADDI4SPN (addi rd', x2, nzuimm)
            0b000 => {
                let rd = to_register(instruction >> 2);
                if rd == 0 { return Err(Exception::UnknownInstruction(instruction as RawInstruction)); }
                // NOTE: nzuimm[5:4|9:6|2|3] (12-5 bit)
                let nzuimm = ((instruction >> 7) & 0b11_0000)
                    | ((instruction >> 1) & 0b11_1100_0000)
                    | ((instruction >> 4) & 0b100)
                    | ((instruction >> 2) & 0b1000);
                Ok(Instruction::ADDI { rd, rs1: 2, imm: nzuimm as Imm })
            },
            // NOTE: C.FLD
            // TODO: Phase 7 (RV64F/D) で実装
            0b001 => Err(Exception::UnknownInstruction(instruction as RawInstruction)),
            // NOTE: C.LW (lw rd', offset(rs1'))
            0b010 => {
                let rd = to_register(instruction >> 2);
                let rs1 = to_register(instruction >> 7);
                // NOTE: uimm[5:3|2|6] * 4
                let uimm = ((instruction >> 7) & 0b11_1000)
                    | ((instruction >> 4) & 0b100)
                    | ((instruction << 1) & 0b100_0000);
                Ok(Instruction::LW { rd, rs1, offset: uimm as Imm })
            },
            // NOTE: C.LD (ld rd', offset(rs1')) (RV64)
            0b011 => {
                // RV64 なので C.LD として実装
                let rd = to_register(instruction >> 2);
                let rs1 = to_register(instruction >> 7);
                // NOTE: uimm[5:3|7:6] * 8
                let uimm = ((instruction >> 7) & 0b11_1000)
                    | ((instruction << 1) & 0b1100_0000);
                Ok(Instruction::LD { rd, rs1, offset: uimm as Imm })
            },
            // NOTE: C.FSD
            // TODO: Phase 7 (RV64F/D) で実装
            0b101 => Err(Exception::UnknownInstruction(instruction as RawInstruction)),
            // NOTE: C.SW (sw rs2', offset(rs1'))
            0b110 => {
                let rs2 = to_register(instruction >> 2);
                let rs1 = to_register(instruction >> 7);
                // NOTE: uimm[5:3|2|6] * 4
                let uimm = ((instruction >> 7) & 0b11_1000)
                    | ((instruction >> 4) & 0b100)
                    | ((instruction << 1) & 0b100_0000);
                Ok(Instruction::SW { rs1, rs2, offset: uimm as Imm })
            },
            // NOTE: C.SD (sd rs2', offset(rs1')) (RV64)
            0b111 => {
                let rs2 = to_register(instruction >> 2);
                let rs1 = to_register(instruction >> 7);
                // NOTE: uimm[5:3|7:6] * 8
                let uimm = ((instruction >> 7) & 0b11_1000)
                    | ((instruction << 1) & 0b1100_0000);
                Ok(Instruction::SD { rs1, rs2, offset: uimm as Imm })
            },

            _ => Err(Exception::UnknownInstruction(instruction as RawInstruction)),
        },
        0b01 => match funct3 {
            // NOTE: C.NOP / C.ADDI
            0b000 => {
                let rd = as_register(instruction >> 7);
                let imm_val = (instruction as i16 >> 7) & 0b10_0000 // bit 5 (sign ext source)
                    | ((instruction >> 2) & 0b1_1111) as i16;
                let nzimm = ((imm_val << 10) >> 10) as Imm; // NOTE: 6bit sign-extend

                if rd == 0 {
                    // NOTE: NOP (ADDI x0, x0, 0)
                    // TODO: rd = 0 かつ imm != 0 は HINT 命令らしい
                    Ok(Instruction::ADDI { rd: 0, rs1: 0, imm: 0 })
                } else {
                    // NOTE: C.ADDI (addi rd, rd, nzimm)
                    Ok(Instruction::ADDI { rd, rs1: rd, imm: nzimm })
                }
            },
            // NOTE: C.ADDIW (RV64)
            0b001 => {
                let rd = as_register(instruction >> 7);
                if rd == 0 { return Err(Exception::UnknownInstruction(instruction as RawInstruction)); }
                let imm_val = (instruction as i16 >> 7) & 0b10_0000
                    | ((instruction >> 2) & 0b1_1111) as i16;
                let imm = ((imm_val << 10) >> 10) as Imm;
                Ok(Instruction::ADDIW { rd, rs1: rd, imm })
            },
            // NOTE: C.LI (addi rd, x0, imm)
            0b010 => {
                let rd = as_register(instruction >> 7);
                if rd == 0 { return Err(Exception::UnknownInstruction(instruction as RawInstruction)); }
                let imm_val = (instruction as i16 >> 7) & 0b10_0000
                    | ((instruction >> 2) & 0b1_1111) as i16;
                let imm = ((imm_val << 10) >> 10) as Imm;
                Ok(Instruction::ADDI { rd, rs1: 0, imm })
            },
            // NOTE: C.ADDI16SP (addi x2, x2, nzimm) / C.LUI (lui rd, nzimm)
            0b011 => {
                let rd = as_register(instruction >> 7);
                if rd == 2 {
                    // NOTE: C.ADDI16SP
                    // NOTE: nzimm[9|4|6|8:7|5] * 16
                    let imm_val = ((instruction >> 3) & 0b10_0000_0000)
                        | ((instruction >> 2) & 0b1_0000)
                        | ((instruction << 1) & 0b100_0000)
                        | ((instruction << 4) & 0b1_1000_0000)
                        | ((instruction << 3) & 0b10_0000);
                    // NOTE: sign extend from bit 9
                    let nzimm = (((imm_val as i16) << 6) >> 6) as Imm;
                    if nzimm == 0 { return Err(Exception::UnknownInstruction(instruction as RawInstruction)); }
                    Ok(Instruction::ADDI { rd: 2, rs1: 2, imm: nzimm })
                } else if rd != 0 {
                    // NOTE: C.LUI
                    // NOTE: nzimm[17|16:12] (inst bits 12 | 6:2)
                    let imm_val = ((instruction >> 7) & 0b10_0000)
                        | ((instruction >> 2) & 0b1_1111);
                    // NOTE: sign extend from bit 17 (bit 5 in imm_val)
                    let nzimm = (((imm_val as i32) << 26) >> 26) as Imm;
                    if nzimm == 0 { return Err(Exception::UnknownInstruction(instruction as RawInstruction)); }
                    Ok(Instruction::LUI { rd, imm: nzimm << 12 })
                } else {
                    Err(Exception::UnknownInstruction(instruction as RawInstruction))
                }
            },
            0b100 => {
                let funct2 = (instruction >> 10) & 0b11;
                let rd = to_register(instruction >> 7); // rd' / rs1'
                let bit12 = (instruction >> 12) & 1;
                let imm_shamt = ((instruction >> 7) & 0b10_0000) | ((instruction >> 2) & 0b1_1111); // imm[5|4:0]

                match funct2 {
                    0b00 => {
                        // NOTE: C.SRLI (srli rd', rd', shamt)
                        let shamt = (bit12 << 5) | imm_shamt;
                        Ok(Instruction::SRLI { rd, rs1: rd, shamt: shamt as Shamt })
                    },
                    0b01 => {
                        // NOTE: C.SRAI (srai rd', rd', shamt)
                        let shamt = (bit12 << 5) | imm_shamt;
                        Ok(Instruction::SRAI { rd, rs1: rd, shamt: shamt as Shamt })
                    },
                    0b10 => {
                        // NOTE: C.ANDI (andi rd', rd', imm)
                        let imm = (((imm_shamt as i8) << 2) >> 2) as Imm;
                        Ok(Instruction::ANDI { rd, rs1: rd, imm })
                    },
                    0b11 => {
                        let rs2 = to_register(instruction >> 2);
                        let op_sub = (instruction >> 5) & 0b11;
                        match (bit12, op_sub) {
                            (0, 0b00) => Ok(Instruction::SUB { rd, rs1: rd, rs2 }),
                            (0, 0b01) => Ok(Instruction::XOR { rd, rs1: rd, rs2 }),
                            (0, 0b10) => Ok(Instruction::OR  { rd, rs1: rd, rs2 }),
                            (0, 0b11) => Ok(Instruction::AND { rd, rs1: rd, rs2 }),
                            (1, 0b00) => Ok(Instruction::SUBW { rd, rs1: rd, rs2 }), // RV64
                            (1, 0b01) => Ok(Instruction::ADDW { rd, rs1: rd, rs2 }), // RV64
                            _ => Err(Exception::UnknownInstruction(instruction as RawInstruction)),
                        }
                    },

                    _ => Err(Exception::UnknownInstruction(instruction as RawInstruction)),
                }
            },
            // NOTE: C.J (jal x0, offset)
            0b101 => {
                // NOTE: offset[11|4|9:8|10|6|7|3:1|5]
                let offset = ((instruction >> 1) & 0b1000_0000_0000)
                    | ((instruction >> 7) & 0b1_0000)
                    | ((instruction >> 1) & 0b11_0000_0000)
                    | ((instruction << 2) & 0b100_0000_0000)
                    | ((instruction >> 1) & 0b100_0000)
                    | ((instruction << 1) & 0b1000_0000)
                    | ((instruction >> 2) & 0b1110)
                    | ((instruction << 3) & 0b10_0000);
                let offset = (((offset as i16) << 4) >> 4) as Imm;
                Ok(Instruction::JAL { rd: 0, offset })
            },
            // NOTE: C.BEQZ (beq rs1', x0, offset)
            0b110 => {
                let rs1 = to_register(instruction >> 7);
                // NOTE: offset[8|4:3|7:6|2:1|5]
                let offset = ((instruction >> 4) & 0b1_0000_0000)
                    | ((instruction >> 7) & 0b1_1000)
                    | ((instruction << 1) & 0b1100_0000)
                    | ((instruction >> 2) & 0b110)
                    | ((instruction << 3) & 0b10_0000);
                let offset = (((offset as i16) << 7) >> 7) as Imm;
                Ok(Instruction::BEQ { rs1, rs2: 0, offset })
            },
            // NOTE: C.BNEZ (bne rs1', x0, offset)
            0b111 => {
                let rs1 = to_register(instruction >> 7);
                // NOTE: offset のビット配置は C.BEQZ と同じ
                let offset = ((instruction >> 4) & 0b1_0000_0000)
                    | ((instruction >> 7) & 0b1_1000)
                    | ((instruction << 1) & 0b1100_0000)
                    | ((instruction >> 2) & 0b110)
                    | ((instruction << 3) & 0b10_0000);
                let offset = (((offset as i16) << 7) >> 7) as Imm;
                Ok(Instruction::BNE { rs1, rs2: 0, offset })
            },

            _ => Err(Exception::UnknownInstruction(instruction as RawInstruction)),
        },
        0b10 => match funct3 {
            // NOTE: C.SLLI (slli rd, rd, shamt)
            0b000 => {
                let rd = as_register(instruction >> 7);
                if rd == 0 { return Err(Exception::UnknownInstruction(instruction as RawInstruction)); }
                // NOTE: shamt[5|4:0] encoded in bit 12 | 6:2
                let shamt = ((instruction >> 7) & 0b10_0000)
                    | ((instruction >> 2) & 0b1_1111);
                Ok(Instruction::SLLI { rd, rs1: rd, shamt: shamt as Shamt })
            },
            // NOTE: C.FLDSP
            // TODO: Phase 7
            0b001 => Err(Exception::UnknownInstruction(instruction as RawInstruction)),
            // NOTE: C.LWSP (lw rd, offset(x2))
            0b010 => {
                let rd = as_register(instruction >> 7);
                if rd == 0 { return Err(Exception::UnknownInstruction(instruction as RawInstruction)); }
                // NOTE: uimm[5|4:2|7:6] * 4
                let uimm = ((instruction >> 7) & 0b10_0000)
                    | ((instruction >> 2) & 0b01_1100)
                    | ((instruction << 4) & 0b1100_0000);
                Ok(Instruction::LW { rd, rs1: 2, offset: uimm as Imm })
            },
            // NOTE: C.LDSP (ld rd, offset(x2)) (RV64)
            0b011 => {
                let rd = as_register(instruction >> 7);
                if rd == 0 { return Err(Exception::UnknownInstruction(instruction as RawInstruction)); }
                // NOTE: uimm[5|4:3|8:6] * 8
                let uimm = ((instruction >> 7) & 0b10_0000)
                    | ((instruction >> 2) & 0b01_1000)
                    | ((instruction << 4) & 0b1_1100_0000);
                Ok(Instruction::LD { rd, rs1: 2, offset: uimm as Imm })
            },
            0b100 => {
                let bit12 = (instruction >> 12) & 1;
                let rs1 = as_register(instruction >> 7); // rd / rs1
                let rs2 = as_register(instruction >> 2); // rs2

                if bit12 == 0 {
                    if rs2 == 0 {
                        // NOTE: C.JR (jalr x0, rs1, 0)
                        if rs1 == 0 { return Err(Exception::UnknownInstruction(instruction as RawInstruction)); }
                        Ok(Instruction::JALR { rd: 0, rs1, offset: 0 })
                    } else {
                        // NOTE: C.MV (add rd, x0, rs2)
                        if rs1 == 0 { return Err(Exception::UnknownInstruction(instruction as RawInstruction)); }
                        Ok(Instruction::ADD { rd: rs1, rs1: 0, rs2 })
                    }
                } else {
                    if rs2 == 0 {
                        if rs1 == 0 {
                            // NOTE: C.EBREAK
                            Ok(Instruction::EBREAK)
                        } else {
                            // NOTE: C.JALR (jalr x1, rs1, 0)
                            Ok(Instruction::JALR { rd: 1, rs1, offset: 0 })
                        }
                    } else {
                        // NOTE: C.ADD (add rd, rd, rs2)
                        if rs1 == 0 { return Err(Exception::UnknownInstruction(instruction as RawInstruction)); }
                        Ok(Instruction::ADD { rd: rs1, rs1, rs2 })
                    }
                }
            },
            // NOTE: C.FSDSP
            // TODO: Phase 7
            0b101 => Err(Exception::UnknownInstruction(instruction as RawInstruction)),
            // NOTE: C.SWSP (sw rs2, offset(x2))
            0b110 => {
                let rs2 = as_register(instruction >> 2);
                // NOTE: uimm[5:2|7:6] * 4
                let uimm = ((instruction >> 7) & 0b11_1100)
                    | ((instruction >> 1) & 0b1100_0000);
                Ok(Instruction::SW { rs1: 2, rs2, offset: uimm as Imm })
            },
            // NOTE: C.SDSP (sd rs2, offset(x2)) (RV64)
            0b111 => {
                let rs2 = as_register(instruction >> 2);
                // NOTE: uimm[5:3|8:6] * 8
                let uimm = ((instruction >> 7) & 0b11_1000)
                    | ((instruction >> 1) & 0b1_1100_0000);
                Ok(Instruction::SD { rs1: 2, rs2, offset: uimm as Imm })
            }

            _ => Err(Exception::UnknownInstruction(instruction as RawInstruction)),
        },

        _ => Err(Exception::UnknownInstruction(instruction as RawInstruction)), // NOTE: opcode = 11 は 32 bit 命令
    }?)
}
