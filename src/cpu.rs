use crate::bus::Bus;

/// レジスタ番号 (Register Index)
pub type RegIdx = u8;
/// 即値 (Immediate)
pub type Imm = i64;
/// シフト量 (Shift Amount)
pub type Shamt = u32;
/// レジスタ長
pub const XLEN: u8 = 64;
// TODO: 将来、Trap に変換される
/// エラー型
#[derive(Debug)]
pub enum Exception {
    /// 未知の命令
    UnknownInstruction(u64),
    /// 不正なメモリアクセス
    InvalidMemoryAccess(u64),
}

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

/// CPU
pub struct Cpu {
    /// レジスタ
    registers: [u64; 32],
    /// プログラムカウンタ
    pc: u64,
    /// バス
    bus: Bus,
}
impl Cpu {
    pub fn new(bus: Bus) -> Self {
        Self {
            registers: [0; 32],
            pc: 0x8000_0000,
            bus,
        }
    }

    /// レジスタを読み込みます。
    pub fn read_register(&self, index: RegIdx) -> u64 {
        if index == 0 {
            return 0;
        }

        self.registers[index as usize]
    }
    /// レジスタに書き込みます。
    pub fn write_register(&mut self, index: RegIdx, value: u64) -> () {
        if index == 0 {
            return;
        }

        self.registers[index as usize] = value;
    }

    /// 命令をフェッチします。
    pub fn fetch(&mut self) -> Result<u64, Exception> {
        // TODO: 圧縮命令を考慮していない (フェーズ2にて実装)
        let instruction = self.bus.read(self.pc, 4)?;
        self.pc += 4;
        Ok(instruction)
    }

    /// 命令をデコードします。
    pub fn decode(&self, instruction: u64) -> Result<Instruction, Exception> {
        let opcode = instruction & 0b111_1111;
        let rd = ((instruction >> 7) & 0b1_1111) as RegIdx; // 宛先レジスタ
        let funct3 = (instruction >> 12) & 0b111; // 細分類その1
        let rs1 = ((instruction >> 15) & 0b1_1111) as RegIdx; // ソースレジスタ1
        let rs2 = ((instruction >> 20) & 0b1_1111) as RegIdx; // ソースレジスタ2
        let funct7 = (instruction >> 25) & 0b111_1111; // 細分類その2

        match opcode {
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
                let funct3 = (instruction >> 12) & 0x7;
                let imm12 = (instruction >> 20) & 0xfff;
                match (funct3, imm12) {
                    (0b000, 0b000000000001) => Ok(Instruction::EBREAK),

                    _ => Err(Exception::UnknownInstruction(instruction)),
                }
            },

            _ => Err(Exception::UnknownInstruction(instruction)),
        }
    }

    /// R-Type (Register-Register) 演算用ヘルパー: rs1 と rs2 を読み出し、op を適用して rd に書き込みます。
    #[inline(always)]
    fn op_rr<F>(&mut self, rd: RegIdx, rs1: RegIdx, rs2: RegIdx, op: F)
    where
        F: FnOnce(u64, u64) -> u64,
    {
        let val1 = self.read_register(rs1);
        let val2 = self.read_register(rs2);
        self.write_register(rd, op(val1, val2));
    }

    /// I-Type (Register-Immediate) 演算用ヘルパー: rs1 を読み出し、imm と op を適用して rd に書き込みます。
    #[inline(always)]
    fn op_ri<F>(&mut self, rd: RegIdx, rs1: RegIdx, imm: Imm, op: F)
    where
        F: FnOnce(u64, u64) -> u64,
    {
        let val1 = self.read_register(rs1);
        self.write_register(rd, op(val1, imm as u64));
    }

    /// R-Type (Register-Register) Word 演算用ヘルパー: 入力の下位 32bit を利用して演算し、その結果を符号拡張して rd に書き込みます。
    #[inline(always)]
    fn op_rrw<F>(&mut self, rd: RegIdx, rs1: RegIdx, rs2: RegIdx, op: F)
    where
        F: FnOnce(i32, i32) -> i32,
    {
        let val1 = self.read_register(rs1) as i32; // NOTE: 下位 32bit を取り出し、i32 として解釈
        let val2 = self.read_register(rs2) as i32;
        let res = op(val1, val2);
        self.write_register(rd, res as i64 as u64); // NOTE: 符号拡張し、u64 として解釈
    }

    /// I-Type (Register-Immediate) Word 演算用ヘルパー: rs1 を読み出し、imm と op を適用して rd に書き込みます。
    #[inline(always)]
    fn op_riw<F>(&mut self, rd: RegIdx, rs1: RegIdx, imm: Imm, op: F)
    where
        F: FnOnce(i32, i32) -> i32,
    {
        let val1 = self.read_register(rs1) as i32;
        let val2 = imm as i32;
        let res = op(val1, val2);
        self.write_register(rd, res as i64 as u64); // NOTE: 符号拡張
    }

    /// B-Type (Branch) 演算用ヘルパー: 条件 (condition) が true なら、offset に基づき PC を更新します。
    #[inline(always)]
    fn op_branch<F>(&mut self, rs1: RegIdx, rs2: RegIdx, offset: Imm, condition: F)
    where
        F: FnOnce(u64, u64) -> bool,
    {
        let val1 = self.read_register(rs1);
        let val2 = self.read_register(rs2);
        if condition(val1, val2) {
            // NOTE: pc は fetch 時に既に +4 されているため、現在の命令アドレスは pc - 4
            self.pc = (self.pc - 4).wrapping_add(offset as u64);
        }
    }

    /// Load 命令用ヘルパー: アドレスを計算し、指定されたクロージャで符号を拡張してレジスタに書き込みます。
    #[inline(always)]
    fn op_load<F>(&mut self, rd: RegIdx, rs1: RegIdx, offset: Imm, width: u64, extend: F) -> Result<(), Exception>
    where
        F: FnOnce(u64) -> u64,
    {
        let addr = self.read_register(rs1).wrapping_add(offset as u64);
        let val = self.bus.read(addr, width)?;
        self.write_register(rd, extend(val));
        Ok(())
    }

    /// Store 命令用ヘルパー: アドレスを計算し、バスに書き込みます。
    #[inline(always)]
    fn op_store(&mut self, rs1: RegIdx, rs2: RegIdx, offset: Imm, width: u64) -> Result<(), Exception> {
        let addr = self.read_register(rs1).wrapping_add(offset as u64);
        let val = self.read_register(rs2);
        self.bus.write(addr, val, width)
    }

    /// Jump 命令用ヘルパー: rd に戻り先アドレスを書き込み、PC を target にジャンプします。
    #[inline(always)]
    fn op_jump(&mut self, rd: RegIdx, target: u64) {
        let next_pc = self.pc; // NOTE: fetch 後なので既に +4 されている
        self.write_register(rd, next_pc);
        self.pc = target;
    }

    /// 命令を実行します。
    pub fn execute(&mut self, instruction: Instruction) -> Result<(), Exception> {
        Ok(match instruction {
            // NOTE: RV32I R-Type
            Instruction::ADD  { rd, rs1, rs2 } => self.op_rr(rd, rs1, rs2, |v1, v2| v1.wrapping_add(v2)),
            Instruction::SUB  { rd, rs1, rs2 } => self.op_rr(rd, rs1, rs2, |v1, v2| v1.wrapping_sub(v2)),
            Instruction::SLL  { rd, rs1, rs2 } => self.op_rr(rd, rs1, rs2, |v1, v2| v1 << (v2 & 0b111111)),
            Instruction::SLT  { rd, rs1, rs2 } => self.op_rr(rd, rs1, rs2, |v1, v2| if (v1 as i64) < (v2 as i64) { 1 } else { 0 }),
            Instruction::SLTU { rd, rs1, rs2 } => self.op_rr(rd, rs1, rs2, |v1, v2| if v1 < v2 { 1 } else { 0 }),
            Instruction::XOR  { rd, rs1, rs2 } => self.op_rr(rd, rs1, rs2, |v1, v2| v1 ^ v2),
            Instruction::SRL  { rd, rs1, rs2 } => self.op_rr(rd, rs1, rs2, |v1, v2| v1 >> (v2 & 0b111111)),
            Instruction::SRA  { rd, rs1, rs2 } => self.op_rr(rd, rs1, rs2, |v1, v2| ((v1 as i64) >> (v2 & 0b111111)) as u64),
            Instruction::OR   { rd, rs1, rs2 } => self.op_rr(rd, rs1, rs2, |v1, v2| v1 | v2),
            Instruction::AND  { rd, rs1, rs2 } => self.op_rr(rd, rs1, rs2, |v1, v2| v1 & v2),
            // NOTE: RV32M
            Instruction::MUL    { rd, rs1, rs2 } => self.op_rr(rd, rs1, rs2, |v1, v2| v1.wrapping_mul(v2)),
            Instruction::MULH   { rd, rs1, rs2 } => self.op_rr(rd, rs1, rs2, |v1, v2| ((v1 as i64 as i128).wrapping_mul(v2 as i64 as i128) >> XLEN) as u64),
            Instruction::MULHSU { rd, rs1, rs2 } => self.op_rr(rd, rs1, rs2, |v1, v2| ((v1 as i64 as i128).wrapping_mul(v2 as u128 as i128) >> XLEN) as u64),
            Instruction::MULHU  { rd, rs1, rs2 } => self.op_rr(rd, rs1, rs2, |v1, v2| ((v1 as u64 as u128).wrapping_mul(v2 as u64 as u128) >> XLEN) as u64),
            Instruction::DIV    { rd, rs1, rs2 } => self.op_rr(rd, rs1, rs2, |v1, v2| {
                let dividend = v1 as i64;
                let divisor = v2 as i64;
                if divisor == 0 {
                    u64::MAX // NOTE: ゼロ除算は -1 を返す
                } else if dividend == i64::MIN && divisor == -1 {
                    dividend as u64 // NOTE: オーバーフロー時は dividend を返す
                } else {
                    dividend.wrapping_div(divisor) as u64
                }
            }),
            Instruction::DIVU   { rd, rs1, rs2 } => self.op_rr(rd, rs1, rs2, |v1, v2| {
                if v2 == 0 {
                    u64::MAX // NOTE: ゼロ除算は -1 を返す
                } else {
                    v1.wrapping_div(v2)
                }
            }),
            Instruction::REM    { rd, rs1, rs2 } => self.op_rr(rd, rs1, rs2, |v1, v2| {
                let dividend = v1 as i64;
                let divisor = v2 as i64;
                if divisor == 0 {
                    dividend as u64 // NOTE: ゼロ除算は dividend を返す
                } else if dividend == i64::MIN && divisor == -1 {
                    0 // NOTE: オーバーフロー時は 0 を返す
                } else {
                    dividend.wrapping_rem(divisor) as u64
                }
            }),
            Instruction::REMU   { rd, rs1, rs2 } => self.op_rr(rd, rs1, rs2, |v1, v2| {
                if v2 == 0 {
                    v1 // NOTE: ゼロ除算は dividend を返す
                } else {
                    v1.wrapping_rem(v2)
                }
            }),
            // NOTE: RV64I R-Type
            Instruction::ADDW { rd, rs1, rs2 } => self.op_rrw(rd, rs1, rs2, |v1, v2| v1.wrapping_add(v2)),
            Instruction::SUBW { rd, rs1, rs2 } => self.op_rrw(rd, rs1, rs2, |v1, v2| v1.wrapping_sub(v2)),
            Instruction::SLLW { rd, rs1, rs2 } => self.op_rrw(rd, rs1, rs2, |v1, v2| v1 << (v2 & 0b11111)), // NOTE: 32bitシフトは下位5bit
            Instruction::SRLW { rd, rs1, rs2 } => self.op_rrw(rd, rs1, rs2, |v1, v2| ((v1 as u32) >> (v2 & 0b11111)) as i32),
            Instruction::SRAW { rd, rs1, rs2 } => self.op_rrw(rd, rs1, rs2, |v1, v2| v1 >> (v2 & 0b11111)), // NOTE: i32 なので算術シフト
            // NOTE: RV64M
            Instruction::MULW { rd, rs1, rs2 } => self.op_rrw(rd, rs1, rs2, |v1, v2| v1.wrapping_mul(v2)),
            Instruction::DIVW { rd, rs1, rs2 } => self.op_rrw(rd, rs1, rs2, |v1, v2| {
                if v2 == 0 {
                    u32::MAX as i32 // -1
                } else if v1 == i32::MIN && v2 == -1 {
                    v1
                } else {
                    v1.wrapping_div(v2)
                }
            }),
            Instruction::DIVUW { rd, rs1, rs2 } => self.op_rrw(rd, rs1, rs2, |v1, v2| {
                let d1 = v1 as u32;
                let d2 = v2 as u32;
                if d2 == 0 {
                    u32::MAX as i32
                } else {
                    d1.wrapping_div(d2) as i32
                }
            }),
            Instruction::REMW { rd, rs1, rs2 } => self.op_rrw(rd, rs1, rs2, |v1, v2| {
                if v2 == 0 {
                    v1
                } else if v1 == i32::MIN && v2 == -1 {
                    0
                } else {
                    v1.wrapping_rem(v2)
                }
            }),
            Instruction::REMUW { rd, rs1, rs2 } => self.op_rrw(rd, rs1, rs2, |v1, v2| {
                let d1 = v1 as u32;
                let d2 = v2 as u32;
                if d2 == 0 {
                    d1 as i32
                } else {
                    d1.wrapping_rem(d2) as i32
                }
            }),

            // NOTE: RV32I I-Type
            Instruction::ADDI  { rd, rs1, imm } => self.op_ri(rd, rs1, imm, |v1, v2| v1.wrapping_add(v2)),
            Instruction::SLTI  { rd, rs1, imm } => self.op_ri(rd, rs1, imm, |v1, v2| if (v1 as i64) < (v2 as i64) { 1 } else { 0 }),
            Instruction::SLTIU { rd, rs1, imm } => self.op_ri(rd, rs1, imm, |v1, v2| if v1 < v2 { 1 } else { 0 }),
            Instruction::XORI  { rd, rs1, imm } => self.op_ri(rd, rs1, imm, |v1, v2| v1 ^ v2),
            Instruction::ORI   { rd, rs1, imm } => self.op_ri(rd, rs1, imm, |v1, v2| v1 | v2),
            Instruction::ANDI  { rd, rs1, imm } => self.op_ri(rd, rs1, imm, |v1, v2| v1 & v2),
            Instruction::SLLI  { rd, rs1, shamt } => self.op_ri(rd, rs1, shamt as i64, |v1, v2| v1 << v2),
            Instruction::SRLI  { rd, rs1, shamt } => self.op_ri(rd, rs1, shamt as i64, |v1, v2| v1 >> v2),
            Instruction::SRAI  { rd, rs1, shamt } => self.op_ri(rd, rs1, shamt as i64, |v1, v2| ((v1 as i64) >> v2) as u64),
            // NOTE: RV64I I-Type
            Instruction::ADDIW { rd, rs1, imm } => self.op_riw(rd, rs1, imm, |v1, v2| v1.wrapping_add(v2)),
            Instruction::SLLIW { rd, rs1, shamt } => self.op_riw(rd, rs1, shamt as i64, |v1, v2| v1 << v2),
            Instruction::SRLIW { rd, rs1, shamt } => self.op_riw(rd, rs1, shamt as i64, |v1, v2| ((v1 as u32) >> v2) as i32),
            Instruction::SRAIW { rd, rs1, shamt } => self.op_riw(rd, rs1, shamt as i64, |v1, v2| v1 >> v2),
            // NOTE: RV32I I-Type (メモリ操作)
            Instruction::LB  { rd, rs1, offset } => self.op_load(rd, rs1, offset, 1, |v| v as i8 as i64 as u64)?,
            Instruction::LH  { rd, rs1, offset } => self.op_load(rd, rs1, offset, 2, |v| v as i16 as i64 as u64)?,
            Instruction::LW  { rd, rs1, offset } => self.op_load(rd, rs1, offset, 4, |v| v as i32 as i64 as u64)?,
            Instruction::LBU { rd, rs1, offset } => self.op_load(rd, rs1, offset, 1, |v| v)?, // NOTE: 符号拡張しない
            Instruction::LHU { rd, rs1, offset } => self.op_load(rd, rs1, offset, 2, |v| v)?,
            // NOTE: RV64I I-Type (メモリ操作)
            Instruction::LD  { rd, rs1, offset } => self.op_load(rd, rs1, offset, 8, |v| v)?,
            Instruction::LWU { rd, rs1, offset } => self.op_load(rd, rs1, offset, 4, |v| v)?,

            // NOTE: RV32I S-Type
            Instruction::SB { rs1, rs2, offset } => self.op_store(rs1, rs2, offset, 1)?,
            Instruction::SH { rs1, rs2, offset } => self.op_store(rs1, rs2, offset, 2)?,
            Instruction::SW { rs1, rs2, offset } => self.op_store(rs1, rs2, offset, 4)?,
            // NOTE: RV64I S-Type
            Instruction::SD { rs1, rs2, offset } => self.op_store(rs1, rs2, offset, 8)?,

            // NOTE: RV32I B-Type
            Instruction::BEQ  { rs1, rs2, offset } => self.op_branch(rs1, rs2, offset, |v1, v2| v1 == v2),
            Instruction::BNE  { rs1, rs2, offset } => self.op_branch(rs1, rs2, offset, |v1, v2| v1 != v2),
            Instruction::BLT  { rs1, rs2, offset } => self.op_branch(rs1, rs2, offset, |v1, v2| (v1 as i64) < (v2 as i64)),
            Instruction::BGE  { rs1, rs2, offset } => self.op_branch(rs1, rs2, offset, |v1, v2| (v1 as i64) >= (v2 as i64)),
            Instruction::BLTU { rs1, rs2, offset } => self.op_branch(rs1, rs2, offset, |v1, v2| v1 < v2),
            Instruction::BGEU { rs1, rs2, offset } => self.op_branch(rs1, rs2, offset, |v1, v2| v1 >= v2),

            // NOTE: RV32I U-Type
            Instruction::LUI   { rd, imm } => self.write_register(rd, imm as u64),
            Instruction::AUIPC { rd, imm } => self.write_register(rd, (self.pc - 4).wrapping_add(imm as u64)),

            // NOTE: RV32I J-Type
            Instruction::JAL { rd, offset } => {
                let target = (self.pc - 4).wrapping_add(offset as u64);
                self.op_jump(rd, target);
            },
            Instruction::JALR { rd, rs1, offset } => {
                // NOTE: JALR は rs1 + offset の最下位ビットを0にする仕様がある
                let target = (self.read_register(rs1).wrapping_add(offset as u64)) & !1;
                self.op_jump(rd, target);
            },

            // NOTE: RV32I System
            Instruction::EBREAK => {},
        })
    }
}
