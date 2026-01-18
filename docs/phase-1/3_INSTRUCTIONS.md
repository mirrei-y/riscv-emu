# 命令の細かい仕様がわからん

## ゼロ除算・オーバーフロー時の挙動

DIVU は符号なし除算であることはわかるが、意外と細かい仕様がわからない。
例えば、ゼロ除算はどうするの？とか。Trap を発生させないということは、なんとなくどこかで見たような気がする。

調べた結果、RISC-V の仕様書まで行きついた。しかしアメリカ語で読むのが辛い。
読めないわけじゃないけど、読んでるだけで日が暮れそうだからここは Gemini に頼ることにした。

> - ゼロ除算 (オペランド `rs2` が 0 の場合) は、結果 `rd` に全ビット 1 の値 (`u64::MAX`) を返す。
> - オーバーフロー (符号付き除算で `INT_MIN / -1` など) は、結果 `rd` に `INT_MIN` を返す。

ってかこれ 2_DECODE.md に書いてあったな……。

## BLTU がなんか違う気がする

[BLTU 命令の Implementation](https://msyksphinz-self.github.io/riscv-isadoc/#_bltu) には以下のように書いてある。

```
if (rs1 >u rs2) pc += sext(offset)
```

これ `>u` じゃなくて `<u` じゃないの？
Gemini に聞いても「うんうん、それは仕様書が悪いね　じゃ、直すね……」って言ってくる。

BLT は `<s` と書いてあるし、BLTU は "Branch if Less Than, Unsigned" だから確かに `<u` だし、やっぱりこれは仕様書が間違ってるような気がする。

というか [SLLIW の OpCode も違うという Issue](https://github.com/msyksphinz-self/riscv-isadoc/issues/27) が立ってる。

いろいろ調査してみたら [BLTU の問題を修正するコミット](https://github.com/msyksphinz-self/riscv-isadoc/commit/ce4377f5c45369ffa851f82fe80435f153bfb217)自体はあるようだが、読んでいた index.html のみ修正されていなかった。
手元に手軽に Ruby と make できる環境がなかったため、PR は断念した（他力本願）。暇になったらやる。
