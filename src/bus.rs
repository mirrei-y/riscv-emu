use crate::{cpu::Exception, memory::Memory};

/// バス
pub struct Bus {
    /// メモリ
    memory: Memory,
}
impl Bus {
    /// 新しい Bus を作成します。
    pub fn new(memory: Memory) -> Self {
        Self {
            memory
        }
    }

    /// メモリからデータを読み込みます。
    pub fn read(&self, addr: u64, size: u64) -> Result<u64, Exception> {
        if addr >= 0x8000_0000 {
            Ok(self.memory.read(addr - 0x8000_0000, size))
        } else {
            // TODO: UART とか将来あるかも
            Err(Exception::InvalidMemoryAccess(addr))
        }
    }

    /// メモリにデータを書き込みます。
    pub fn write(&mut self, addr: u64, value: u64, size: u64) -> Result<(), Exception> {
        if addr >= 0x8000_0000 {
            self.memory.write(addr - 0x8000_0000, value, size);
            Ok(())
        } else {
            // TODO: UART とか将来あるかも
            Err(Exception::InvalidMemoryAccess(addr))
        }
    }
}
