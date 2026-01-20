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

## 特例処理の追加

### rd = x0, rs1 = x0

パスしたからいいかと思っていたら、Gemini からのフィードバックで「`rd = x0` などの際、特別処理を追加する必要がある」と言われた。

rd によって書き込む・書き込まないが決まってしまうので、csr.rs の `execute_*` 関数はひとまず全部削除。cpu.rs の execute 関数内で直接 CSR を触ってしまうことにした。

```rust
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
```

### mhartid, misa

CSR レジスタの中で特別扱いしなければならないものがあるらしく、それが `mhartid` と `misa`。

`mhartid` は Machine Hardware Thread ID の略、CPU の ID を表す CSR レジスタで、シングルコアなので 0 固定で良いらしい。
書き込み命令は無視。

`misa` は Machine ISA の略、CPU がサポートしている命令セットを表現する CSR レジスタらしい。
これも書き込み命令は無視。

`misa` に実際に設定する値については、[riscv-isa-manual](https://github.com/riscv/riscv-isa-manual/blob/main/src/machine.adoc#machine-isa-misa-register) が参考になった。

あと、MXL のビットはどこに置く？という疑問も発生した。
これは「[M-modeの実装 (1. CSRの実装)](https://cpu.kanataso.net/20-mmode-csr.html#%E6%A6%82%E8%A6%81)」の図が大変役に立った。
……というかこれに沿って進めばよかったのでは。

これを踏まえて、`Csr::read` メソッドを以下のように修正する。

```rust
/// CSR レジスタの値を読み取ります。
pub fn read(&self, addr: u16) -> Result<u64, Exception> {
    if addr as usize >= self.data.len() {
        Err(Exception::InvalidCsrAccess(addr))
    } else if addr == CSR_MHARTID {
        Ok(0) // TODO: シングルコア
    } else if addr == CSR_MISA {
        const MISA_64BIT: u64 = 2 << 62; // 32bit=1, 64bit=2
        const MISA_EXT_I: u64 = 1 << (b'I' - b'A');
        const MISA_EXT_M: u64 = 1 << (b'M' - b'A');
        const MISA_EXT_C: u64 = 1 << (b'C' - b'A');
        Ok(MISA_64BIT | MISA_EXT_I | MISA_EXT_M | MISA_EXT_C)
    } else {
        Ok(self.data[addr as usize])
    }
}
```

`marchid`, `mvendorid`, `mimpid` など設定できるみたい。個人的にすごいやってみたいが、本質的ではないので後にする。

### mstatus

`mstatus` レジスタは、特権関連の状態を管理したり、その他いろいろ管理している CSR レジスタらしいが、これに対し未定義のビットは常に 0、そして未定義のビットの書き込みもしてはいけないらしい。

そしてここにフィールドが大量にあり、把握するのがかなり面倒。
