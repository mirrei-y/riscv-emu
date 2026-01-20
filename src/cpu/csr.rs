use crate::Exception;

pub const CSR_MHARTID: u16 = 0xF14;
pub const CSR_MISA: u16 = 0x301;
pub const CSR_MSTATUS: u16 = 0x300;

const MISA_64BIT: u64 = 2 << 62; // 32bit=1, 64bit=2

/// MISA レジスタの CPU 拡張表現ビットを取得します。
const fn ext(ext: u8) -> u64 {
    1 << (ext - b'A')
}

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
        } else if addr == CSR_MHARTID {
            Ok(0) // TODO: シングルコア
        } else if addr == CSR_MISA {
            Ok(MISA_64BIT | ext(b'I') | ext(b'M') | ext(b'C'))
        } else {
            Ok(self.data[addr as usize])
        }
        // TODO: marchid, mvendorid, mimpid, etc...
    }
    /// CSR レジスタに値を書き込みます。
    pub fn write(&mut self, addr: u16, val: u64) {
        if addr as usize >= self.data.len() {
            return;
        }
        if addr == CSR_MHARTID || addr == CSR_MISA {
            return;
        }
        self.data[addr as usize] = val;
    }
}
