pub struct Memory {
    /// メモリのデータ
    data: Vec<u8>,
}
impl Memory {
    /// 新しい Memory を作成します。
    pub fn new(size: usize) -> Self {
        Self {
            data: vec![0; size],
        }
    }

    /// メモリからデータを読み込みます。
    pub fn read(&self, addr: u64, size: u64) -> u64 {
        let mut value = 0;

        for i in 0..size {
            value |= (self.data[(addr + i) as usize] as u64) << (i * 8);
        }
        value
    }

    /// メモリにデータを書き込みます。
    pub fn write(&mut self, addr: u64, value: u64, size: u64) -> () {
        for i in 0..size {
            self.data[(addr + i) as usize] = ((value >> (i * 8)) & 0xff) as u8;
        }
    }
}
