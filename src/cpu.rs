mod csr;
mod decode;

use crate::{Exception, Imm, Instruction, InstructionContext, RawInstruction, RawShortInstruction, RegIdx, XLEN, bus::Bus, cpu::csr::Csr};

/// CPU
pub struct Cpu {
    /// レジスタ
    registers: [u64; 32],
    /// プログラムカウンタ
    pc: u64,
    /// バス
    bus: Bus,
    /// CSR レジスタ
    csr: Csr,
}
impl Cpu {
    pub fn new(bus: Bus) -> Self {
        Self {
            registers: [0; 32],
            pc: 0x8000_0000,
            bus,
            csr: Csr::new(),
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
    pub fn fetch(&mut self) -> Result<RawInstruction, Exception> {
        let instruction = self.bus.read(self.pc, 4)? as RawInstruction;
        Ok(instruction)
    }

    pub fn decode(&self, instruction: RawInstruction) -> Result<InstructionContext, Exception> {
        if instruction & 0b11 != 0b11 {
            Ok(InstructionContext {
                instruction: decode::decode_compressed(instruction as RawShortInstruction)?,
                next_pc: self.pc + 2,
            })
        } else {
            Ok(InstructionContext {
                instruction: decode::decode(instruction)?,
                next_pc: self.pc + 4,
            })
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
            self.pc = self.pc.wrapping_add(offset as u64);
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
    fn op_jump(&mut self, ctx: InstructionContext, rd: RegIdx, target: u64) {
        self.write_register(rd, ctx.next_pc);
        self.pc = target;
    }

    /// 命令を実行します。
    pub fn execute(&mut self, ctx: InstructionContext) -> Result<(), Exception> {
        let current_pc = self.pc;
        let next_pc = ctx.next_pc;

        match ctx.instruction {
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
            Instruction::AUIPC { rd, imm } => self.write_register(rd, self.pc.wrapping_add(imm as u64)),

            // NOTE: RV32I J-Type
            Instruction::JAL { rd, offset } => {
                let target = self.pc.wrapping_add(offset as u64);
                self.op_jump(ctx, rd, target);
            },
            Instruction::JALR { rd, rs1, offset } => {
                // NOTE: JALR は rs1 + offset の最下位ビットを0にする仕様がある
                let target = (self.read_register(rs1).wrapping_add(offset as u64)) & !1;
                self.op_jump(ctx, rd, target);
            },

            // NOTE: RV32I System
            Instruction::ECALL => {},
            Instruction::EBREAK => {},
            Instruction::CSRRW { rd, rs1, csr } => {
                let old_value = if rd != 0 { self.csr.read(csr)? } else { 0 };
                self.csr.write(csr, self.read_register(rs1));
                self.write_register(rd, old_value);
            }
            Instruction::CSRRS { rd, rs1, csr } => {
                let old_value = self.csr.read(csr)?;
                if rs1 != 0 {
                    self.csr.write(csr, old_value | self.read_register(rs1));
                }
                self.write_register(rd, old_value);
            }
            Instruction::CSRRC { rd, rs1, csr } => {
                let old_value = self.csr.read(csr)?;
                if rs1 != 0 {
                    self.csr.write(csr, old_value & !self.read_register(rs1));
                }
                self.write_register(rd, old_value);
            }
            Instruction::CSRRWI { rd, imm, csr } => {
                let old_value = if rd != 0 { self.csr.read(csr)? } else { 0 };
                self.csr.write(csr, imm as u64);
                self.write_register(rd, old_value);
            }
            Instruction::CSRRSI { rd, imm, csr } => {
                let old_value = self.csr.read(csr)?;
                if imm != 0 {
                    self.csr.write(csr, old_value | (imm as u64));
                }
                self.write_register(rd, old_value);
            }
            Instruction::CSRRCI { rd, imm, csr } => {
                let old_value = self.csr.read(csr)?;
                if imm != 0 {
                    self.csr.write(csr, old_value & !(imm as u64));
                }
                self.write_register(rd, old_value);
            }

        }

        if current_pc == self.pc {
            // NOTE: 命令によって PC が更新されていなければ、次の命令へ進む
            self.pc = next_pc;
        }

        Ok(())
    }

    pub fn cycle(&mut self) {
        // TODO: unwrap
        let instruction = self.fetch().unwrap();
        let ctx = self.decode(instruction).unwrap();

        println!("Execute: {:?}", ctx);

        if let Instruction::EBREAK = ctx.instruction {
            panic!("EBREAK encountered. Halting execution."); // TODO: 暫定的処置
        }

        match self.execute(ctx) {
            Ok(_) => {},
            Err(e) => {
                panic!("{:?}", e);
            },
        };
    }
}
