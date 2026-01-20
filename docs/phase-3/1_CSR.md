# CSR とは

CPU の中にある、CPU 自体の状態や動作を決定する特別なレジスタのこと。
既存のレジスタとも MMIO とも独立で、12bit のアドレス空間 (最大 4096 個) を持つらしい。

これを読み書きするのが、このフェーズで実装する Zicsr 命令群。

今は必要なものだけ実装し、それ以外は 0 を返すという方針で動きそう。

## 命令の実装

```rust
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
```

とりあえず Exception::InvalidCsrAccess を定義しちゃったが大丈夫だろうか。

で、次に csrrw, csrrs, csrrc, ... などの命令を実装していく。
しかし Cpu 構造体の execute から直接 Csr を触ると大変なので、Csr の impl に命令は分離する。

```rust
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
// ...
```

「[Zicsr拡張の実装](https://cpu.kanataso.net/04a-zicsr.html)」によると、**CSR に対する操作命令は**これだけらしい。あとは ECALL, *RET などの細かい命令はあるが、これらは CSR を使う命令であって、CSR 自体の操作ではない。

意外と簡単に CSR 自体の実装は終わってしまった。

## デコードの実装

やってて一番つらい時間第一位、デコード。

いつもと同様、decode.rs の中に CSR 命令のデコードを追加する。

```rust
match funct3 {
    0b000 => match imm12 {
        0b000000000001 => Ok(Instruction::EBREAK),

        _ => Err(Exception::UnknownInstruction(instruction)),
    },

    0b001 => Ok(Instruction::CSRRW { rd, rs1, csr: imm12 as u16 }),
    // ...

    _ => Err(Exception::UnknownInstruction(instruction)),
}
```

EBREAK の判別処理の手前で少し match を分岐させた。

実装してて知ったが、オペコード的には `0b11100_11` であるものの、EBREAK, ECALL は RV32I であって Zicsr ではないらしい。

さらに MRET, SRET, URET は**特権命令**と呼ばれるものであって、これも Zicsr 命令ではないものの、やはり特権がどうのこうのの話なので、これを実用するならば CSR の実装が必要らしい。

- RV32I: ECALL, EBREAK, FENCE
- Zicsr: CSR*
- Zifencei: FENCE.I
- 特権命令: URET, SRET, MRET, WFI, SFENCE.VMA

という感じらしい。

CSR* については無事テストをパスした。
とはいえ、まだ CSR に代入することしかしていないので、果たして本当にパスしたと言えるかは怪しいが……。
