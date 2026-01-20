use crate::Exception;

/// CSR レジスタ構造体
pub struct Csr {
    /// CSR レジスタの値
    data: [u64; 4096],
}
impl Csr {
    /// CSR レジスタ構造体を作成します。
    pub fn new() -> Self {
        Self { data: [0; 4096] }
    }

    /// CSR レジスタの値を読み取ります。
    pub fn read(&self, addr: u16) -> Result<u64, Exception> {
        if addr as usize >= self.data.len() {
            Err(Exception::InvalidCsrAccess(addr))
        } else {
            Ok(self.data[addr as usize])
        }
    }
    /// CSR レジスタに値を書き込みます。
    pub fn write(&mut self, addr: u16, val: u64) {
        // 書き込み可能ビットマスク（WARL: Write Any Read Legal）の処理が必要な場合がある
        // 例: mstatus の特定ビットは書き換え不可、など
        self.data[addr as usize] = val;
    }

    /// csrrw 命令 (Read and Write) を実行します。
    pub fn execute_rw(&mut self, addr: u16, val: u64) -> Result<u64, Exception> {
        let old_val = self.read(addr)?;
        self.write(addr, val);
        Ok(old_val)
    }
    /// csrrs 命令 (Read and Set) を実行します。
    pub fn execute_rs(&mut self, addr: u16, val: u64) -> Result<u64, Exception> {
        let old_val = self.read(addr)?;
        self.write(addr, old_val | val);
        Ok(old_val)
    }
    /// csrrc 命令 (Read and Clear) を実行します。
    pub fn execute_rc(&mut self, addr: u16, val: u64) -> Result<u64, Exception> {
        let old_val = self.read(addr)?;
        self.write(addr, old_val & !val);
        Ok(old_val)
    }
    /// csrrwi 命令 (Read and Write Immediate) を実行します。
    pub fn execute_rwi(&mut self, addr: u16, imm: u8) -> Result<u64, Exception> {
        let old_val = self.read(addr)?;
        self.write(addr, imm as u64);
        Ok(old_val)
    }
    /// csrrsi 命令 (Read and Set Immediate) を実行します。
    pub fn execute_rsi(&mut self, addr: u16, imm: u8) -> Result<u64, Exception> {
        let old_val = self.read(addr)?;
        self.write(addr, old_val | (imm as u64));
        Ok(old_val)
    }
    /// csrrci 命令 (Read and Clear Immediate) を実行します。
    pub fn execute_rci(&mut self, addr: u16, imm: u8) -> Result<u64, Exception> {
        let old_val = self.read(addr)?;
        self.write(addr, old_val & !(imm as u64));
        Ok(old_val)
    }
}
