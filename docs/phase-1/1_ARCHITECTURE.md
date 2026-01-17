# アーキテクチャはどうする

何もわからないなりに RISC-V エミュレータを作ることになった。

さて cargo new すると main.rs に Hello World! が書いてあるが、このままじゃ何も進まん。

```rust
struct Cpu {
    registers
}
impl Cpu {
    fn fetch();
    fn execute(bytecode: Vec<u8>);
    // ...
}
```

みたいな感じであることはわかるが、そこから進まない。

Gemini に聞いてみると、どうやら**バス**という概念がいるらしい。こいつ経由でメモリから命令を取得するらしい。

## バス

直接メモリを読めばよくね？と思ったが、後々面倒になるからやめるらしい。

> CPUは直接 Dram を触らず、Bus を通します。これは後々「UART」や「CLINT」などのデバイスが増えたとき、アドレスによってアクセス先を振り分けるためです。

確かに、仮想メモリとか実装するなら、CPU が直接メモリを触るのはまずいかも。

よって、以下のようになる。

```rust
struct Bus {
    memory: Vec<u8>,
    // 他のデバイスもここに追加される
}
struct Cpu {
    registers: [u64; 32],
    pc: u64,
    bus: Bus,
}

fn main() {
    let bus = Bus {
        memory: vec![0; 1024 * 1024],
    };
    let mut cpu = Cpu {
        registers,
        pc: 0,
        bus,
    };

    match cpu.fetch() {
        Some(instruction) => cpu.execute(instruction),
        None => println!("END"),
    };
}
```

`Bus.memory` があるのがいいのかなと思って Gemini に聞いてみたらダメらしい。`Memory`, `Bus`, `Cpu` と分けるか。

```rust
struct Memory {
    data: Vec<u8>,
}
```

で、バスからメモリを読むようにする。
RISC-V ではリトルエンティアンであることに注意し、後々のため Exception を定義しておく。

```rust
impl Memory {
    fn read(&self, addr: u64, size: u64) -> u64 {
        let mut value = 0;

        for i in 0..size {
            value |= (self.data[(addr + i) as usize] as u64) << (i * 8);
        }
        value
    }
}
impl Bus {
    pub fn read(&self, addr: u64, size: u64) -> Result<u64, Exception> {
        if addr >= 0x8000_0000 {
            Ok(self.memory.read(addr - 0x8000_0000, size))
        } else {
            Err(Exception::InvalidMemoryAccess(addr))
        }
    }
}
```

なんか 0x8000_0000 からメモリを読むらしい。
メモリマップド I/O というやつらしい、聞いたことくらいはある。

## レジスタ

レジスタって `[u64; ?]` とかでいいのかと思い聞いてみた。
RISC-V には 32 本の汎用レジスタがあるらしく、各 64bit なので、`[u64; 32]` ということになる。

## フェッチを書く

メモリを読んで、PC を進める。

```rust
pub fn fetch(&mut self) -> Result<u64, Exception> {
    // TODO: 圧縮命令を考慮していない (フェーズ2)
    let instructions = self.bus.read(self.pc, 4)?;
    self.pc += 4;
    Ok(instructions)
}
```

これだけ。

## デコードを書く

### ひたすら列挙していくまで

RISC-V の命令セットを enum にする必要がありそう。

Gemini によると、ほとんどの形式で共通する場所にあるビットは抜き出したほうが楽とのこと。
ソースレジスタとかならわかるけど、細分類って何だろう……。とりあえず実装する。

```rust
let opcode = instruction & 0x7f;
let rd = (instruction >> 7) & 0x1f; // 宛先レジスタ
let rs1 = (instruction >> 15) & 0x1f; // ソースレジスタ1
let rs2 = (instruction >> 20) & 0x1f; // ソースレジスタ2
let funct3 = (instruction >> 12) & 0x7; // 細分類その1
let funct7 = (instruction >> 25) & 0x7f; // 細分類その2
```

オペコードは、どの命令も下位 7bit で大体わかるらしい。
RISC-V の命令形式には以下の種類があるらしい。

- R-type (Register): レジスタ同士の演算 (add, sub, xor など)
- I-type (Immediate): 即値演算、ロード、ジャンプ (addi, lb, jalr など)
- S-type (Store): ストア (sb, sw など)
- B-type (Branch): 条件分岐 (beq, bne など)
    - ※ S-type の亜種。即値の場所が少し変則的。
- U-type (Upper): 上位ビット設定 (lui, auipc)
- J-type (Jump): 無条件ジャンプ (jal)
    - ※ U-type の亜種。即値の場所がかなり変則的。

enum にしようかと思ったけど、これなら match で分岐したほうがいいかも。

まず即値で簡単そうな I-type を実装することにする。
下位 7bit が 0b0010011 なので、こんな感じ。

符号拡張というのがあるらしい。
Rust でやるなら:
- `as i32` とかで一旦符号付き 32 bit 整数に変換
- 算術右シフト (`>>`) で上位ビットを符号ビットで埋める
- `as u64` で戻すと、符号拡張された 64 bit 整数になる

```rust
pub fn decode(&self, instruction: u64) -> Result<Instruction, Exception> {
    let opcode = instruction & 0x7f;
    let rd = (instruction >> 7) & 0x1f; // 宛先レジスタ
    let funct3 = (instruction >> 12) & 0x7; // 細分類その1
    let rs1 = (instruction >> 15) & 0x1f; // ソースレジスタ1
    let rs2 = (instruction >> 20) & 0x1f; // ソースレジスタ2
    let funct7 = (instruction >> 25) & 0x7f; // 細分類その2
    match opcode {
        // NOTE: I-Type 命令
        0b00100_11 => match (funct7, funct3) {
            // NOTE: ADDI 命令
            (_, 0b000) => Ok(Instruction::ADDI {
                rd: rd,
                rs1: rs1,
                imm: ((instruction as i32) >> 20) as u64,
            }),
            _ => Err(Exception::UnknownInstruction(instruction)),
        },
        _ => Err(Exception::UnknownInstruction(instruction)),
    }
}
```

残りもひたすら実装していく。
