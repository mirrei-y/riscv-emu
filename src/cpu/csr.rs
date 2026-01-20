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
}
