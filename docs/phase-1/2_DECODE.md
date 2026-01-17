# デコードをひたすら列挙する

## 命令を分類する

ここで、何を実装すればよく分からなくなってきた。
[RISC-V 命令一覧](https://msyksphinz-self.github.io/riscv-isadoc/html/index.html) を読ませた Gemini （Gem 機能を作るのが楽だった）に命令セットを分類してもらう。

> ### RV64IM 命令一覧
> 
> #### R-type (Register-Register)
> 
> `opcode`, `funct3`, `funct7` で命令を特定します。
> 
> **基本整数演算:**
> 
> * `add` (加算)
> * `sub` (減算)
> * `sll` (論理左シフト)
> * `slt` (Set Less Than: 符号付き比較)
> * `sltu` (Set Less Than Unsigned: 符号なし比較)
> * `xor` (排他的論理和)
> * `srl` (論理右シフト)
> * `sra` (算術右シフト)
> * `or` (論理和)
> * `and` (論理積)
> 
> **32bit 演算 (結果は符号拡張されて64bitレジスタに入る):**
> 
> * `addw`
> * `subw`
> * `sllw`
> * `srlw`
> * `sraw`
> 
> **乗算・除算:**
> 
> * `mul` (下位64bit)
> * `mulh` (上位64bit: 符号付き x 符号付き)
> * `mulhsu` (上位64bit: 符号付き x 符号なし)
> * `mulhu` (上位64bit: 符号なし x 符号なし)
> * `div` (除算: 符号付き)
> * `divu` (除算: 符号なし)
> * `rem` (剰余: 符号付き)
> * `remu` (剰余: 符号なし)
> 
> **32bit 乗算・除算:**
> 
> * `mulw`
> * `divw`
> * `divuw`
> * `remw`
> * `remuw`
> 
> #### I-type (Immediate)
> 
> 演算系とロード系、ジャンプ系 (`jalr`) が混在しています。
> 
> **算術・論理演算 (即値):**
> 
> * `addi`
> * `slti`
> * `sltiu`
> * `xori`
> * `ori`
> * `andi`
> * `slli`
> * `srli`
> * `srai` (※ `srli` と `srai` は即値フィールドの一部で区別)
> 
> **32bit 算術・論理演算:**
> 
> * `addiw`
> * `slliw`
> * `srliw`
> * `sraiw`
> 
> **メモリロード:**
> 
> * `lb` (Load Byte: 符号拡張)
> * `lh` (Load Halfword: 符号拡張)
> * `lw` (Load Word: 符号拡張)
> * `lbu` (Load Byte Unsigned: ゼロ拡張)
> * `lhu` (Load Halfword Unsigned: ゼロ拡張)
> * `lwu` (Load Word Unsigned: ゼロ拡張)
> * `ld` (Load Doubleword: 64bit)
> 
> **制御・その他:**
> 
> * `jalr` (Jump And Link Register: 間接ジャンプ)
> * `fence` (メモリフェンス: フェーズ4で重要ですが、今はNOP扱いで可)
> * `ecall` (Environment Call: フェーズ3で使用)
> * `ebreak` (Environment Break: デバッガ用)
> 
> #### S-type (Store)
> 
> レジスタの値をメモリに書き込みます。
> 
> * `sb` (Store Byte)
> * `sh` (Store Halfword)
> * `sw` (Store Word)
> * `sd` (Store Doubleword)
> 
> #### B-type (Branch)
> 
> 条件分岐です。比較結果によって PC を書き換えます。
> 
> * `beq` (Equal)
> * `bne` (Not Equal)
> * `blt` (Less Than)
> * `bge` (Greater or Equal)
> * `bltu` (Less Than Unsigned)
> * `bgeu` (Greater or Equal Unsigned)
> 
> #### U-type (Upper Immediate)
> 
> 大きな即値を扱うために使われます。
> 
> * `lui` (Load Upper Immediate: 上位20bitに即値をセット)
> * `auipc` (Add Upper Immediate to PC: PC相対アドレス計算用)
> 
> #### J-type (Jump)
> 
> 無条件ジャンプです。
> 
> * `jal` (Jump And Link: 関数呼び出しなど)
> 
> ### 注意点
> 
> 1. **`*W` 命令の挙動 (RV64)**
> `addw` や `addiw` など、`W` がつく命令は「下位 32bit で演算を行い、その結果を **64bit に符号拡張 (Sign-extend)** してレジスタに書き込む」という仕様です。
> - 計算: $result = (op1 + op2) \ \& \ \text{0xFFFFFFFF}$
> - 格納: $reg[rd] = \text{sign\_extend}(result)$
> * ここを単なる 32bit マスクで終わらせると、後続の命令がおかしくなることがあります。
> 
> 2. **シフト命令の即値 (I-type)**
> `slli`, `srli`, `srai` (およびそのW版) は I-type ですが、即値フィールドすべてをシフト量として使うわけではありません。RV64 では下位 6bit (W版は 5bit) がシフト量(`shamt`)となり、上位ビットは命令の種類の区別（論理 vs 算術）に使われます。
> 
> 3. **NOP 命令**
> RISC-V には `nop` という専用命令はありません。`addi x0, x0, 0` が `nop` として扱われます（`x0` は常に 0 なので書き込みが無効）。
> 
> まずはこれら RV64IM の命令を列挙した `enum Instruction` を定義し、バイナリ列からデコードする部分を作るのが次のステップですね。

[この RISC-V 命令一覧ページ](https://msyksphinz-self.github.io/riscv-isadoc/html/rvi.html#add)を参考に進める。

最初は "RV32I, RV64I Instructions" のページを見ていたが、64bit の命令（W がついているもの）が列挙されていないことに気づいた。
このページ通りデコードでぽつぽつ実装していては、何が実装されているかわからなくなりそうな気がしたので、"RV64I Instructions", "RV32M, RV64M Instructions", "RV64M Instructions" を含め、RV64IM のデコードを一気に書くことにした。

とはいえ、サイトの "RV32I, RV64I Instructions" セクションの `fence`～`sfence.vma` までは実装してない。必要に応じて実装する。

ところでこのデコードは、ひたすら仕様書を見て match 文を埋めていくという結構苦痛な作業だった。
GitHub Copilot のおかげで、ひたすら確認＆Tabを押すだけになり、かなり楽になった。
GitHub Copilot の補完は GPT-4o だったか、そこら辺の低価格モデルを使っていたはずだ。
しかし RISC-V の命令セットは知っているのか、今まで1回も間違った補完をしなかった。いつもはデタラメな補完ばっかりしてくるのに……。
ところでやはり疲れた。なぜこんなことを始めてしまったのか。

暇なので雑談をするが、今は「結城友奈は勇者である 勇者の章」2週目を見ながら作業している。おすすめですよ。

## 今後の実装について注意された

**今後の実装（Execute フェーズ）での注意点**:
デコード後の `execute` 関数を実装する際、M 拡張には特有のエッジケースがあります（仕様書で定義されています）。

- ゼロ除算: RISC-V では例外（Trap）を発生させず、特定の値を返す仕様です。
    - `DIV` / `DIVW` でゼロ除算: `-1` (全ビット 1) を返す。
    - `REM` / `REMW` でゼロ除算: 被除数 (`rs1`) をそのまま返す。
- オーバーフロー: 符号付き除算でのオーバーフロー時も Trap しません。
    - `DIV` (`INT_MIN / -1`): `INT_MIN` を返す。
    - `REM` (`INT_MIN % -1`): `0` を返す。

これらの挙動は、Rust の標準的な / 演算子がパニックするケース（ゼロ除算など）と異なるため、`execute` 実装時には条件分岐でガードする必要があります。

## デバッグ

よく考えたらサンプル命令が終われないので、ついでに EBREAK 命令も実装した。
