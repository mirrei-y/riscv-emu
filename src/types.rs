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
