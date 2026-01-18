# RISC-V エミュレータを作るぞ❗❗❗❗

基本は Gemini で得た知識:
- [メインのスレッド](https://gemini.google.com/app/b520f29e013f3ba6)

あとはこれらも役に立つ:
- [RISC-V 命令一覧](https://msyksphinz-self.github.io/riscv-isadoc/html/index.html)
- [RV32I 命令の詳細「RISC-V RV32I CPU ASFRV32Iの設計と実装」](https://qiita.com/asfdrwe/items/595c871611e6603741fa#asfrv32i)

わからなくなったらこれ:
- [拡張命令セットの図「RISC-Vについて学ぶ-前編」](https://www.lineo.co.jp/blog/linux/risc-v.html#2.2)
- [RISC-V に関連したいろいろなリンク「RISC-V技術資料集」](https://qiita.com/asfdrwe/items/c0bcab0f67d8562bcb05)

RISC-V は「モジュラー設計」で、必要な命令セットを段階的に実装していくらしい。

Linux を動かすなら RV64GC (RV64 + Integer + Multiply + Atomic + Float + Double + Compressed + Zicsr (CSRs) + Zifencei) 命令セットが必要。

## ロードマップ

## レベル 1: ベアメタルプログラムを動かす
- ✅ ~~フェーズ 1: RV64**IM** (ただのステートレス計算機)~~
    - 汎用レジスタとPCだけ定義
    - メモリはただの Vec
    - 足し算、引き算、掛け算、割り算、ロード、ストアだけ
        - 掛け算割り算は Rust で実装できるので実は楽　うれしい
- ✅ ~~フェーズ 2: RV64IM**C** (圧縮命令)~~
    - RISC-V は、基本 32bit 命令だけど、16bit 命令も定義されていて、それを圧縮命令と呼ぶ
    - 命令をフェッチした直後、`下位2ビット != 11` なら圧縮命令なので、それを 32bit に置き換えてからデコードに回す
        - ~~`decode(fetch())` -> `decode(expand(fetch()))` のイメージ~~
        - `fetch()` の結果を見て、`decode()` か `decode_compressed()` に振り分けるイメージに変更
    - 圧縮って Zlib 的なことをハードウェアでアクセラレーションするのかと思ってた、意外と簡単らしい

メモリ上で動く計算機。フィボナッチ数列計算とかできる。

## レベル 2: システム管理
- フェーズ 3: Zicsr (特権とCSR)
    - CSR (ユーザー権限とか割込み許可、例外のジャンプ先、ページテーブル位置などを置いておくレジスタ) を実装
    - ecall (システムコール) で例外ハンドラに飛ぶ Trap 処理を実装
    - M-mode (Machine Mode) だけで動くベアメタルプログラムを動かす
- フェーズ 4: RV64IM**A**C (アトミック命令)
    - アトミック命令 (LR/SC 命令や AMO 命令) を実装
        - Register Reservation というところに読み書きするアドレスを記憶する
        - LR -> SC でアドレスが一致していないならストア処理を無効にする
    - [Zifencei](https://tomo-wait-for-it-yuki.hatenablog.com/entry/2018/12/31/095511) (メモリ操作命令の順序制御) も実装
        - 例えば自分で書き換えた結果の命令をフェッチする場合とか
        - そもそも今はシングルコアかつ命令キャッシュなしなので、NOP でいいらしい
            - LR で普通にロード、SC で普通にストアし 0 をレジスタに入れておく
- フェーズ 5: MMU / Sv39 (仮想メモリ)
    - ページテーブルを実装
    - Sv39 のアドレス変換を実装
        - ここで例外が起きる場合があるので、Trap 処理を使う
    - メモリアクセスのパイプラインに割り込ませる
        - `memory[addr]` -> `memory[translate(addr)]` のイメージ
    - 面倒
        - 木構造🤡🤡🤡🤡🤡
            - と言いつつも、木構造なのはそれはそう (だって複数のページテーブルを再帰的に参照すれば、全体としてみれば木だよね) なので、まあなんとかなりそう
        - ページテーブルが多重化しているので、3回くらいポインタをたどるらしい😥

[riscv-tests](https://github.com/riscv-software-src/riscv-tests) というリポジトリに単体テストのバイナリがいろいろあるので、これを実行してみよう

## レベル 3: Linux を動かそう
- フェーズ 6: CLINT / PLIC
    - CLINT (指定した時間になったら、mip レジスタのタイマ割込みビットを立てる) を実装
    - PLIC (外部デバイスからの割込みを管理し、まとめて CPU に伝えるハブ) を実装
- フェーズ 7: RV64IMAC**FD**
    - 浮動小数点用のレジスタ (丸めモードや例外フラグである fcsr を用意) と命令を追加
    - Rust の f32, f64 にキャストすればいいので、多少楽
        - NaN 周りの挙動が面倒くさい
        - **注意点 (NaN Boxing)**: Rustの `f64` にキャストするだけで計算はできますが、RISC-Vには 「32bitのfloat (F) を64bitのレジスタ (D) に入れるとき、上位32bitを全部 1 で埋める (NaN Boxing)」 という特有の仕様があります。これを忘れるとバグります。
    - ここまで持ってきているのは、どうやら Linux よりユーザースペースのプログラム (/bin/init など) のほうが浮動小数点命令を使っているかららしい
- フェーズ 8: UART / VirtIO / Device Tree
    - UART (特定のアドレスに書き込むとターミナルに文字が出る) を実装
    - VirtIO (rootfs を読み書きするやつ。`virtio-drivers` というクレートもある) を実装
    - Device Tree (今こんなデバイスで動いてるよという情報を伝えるやつ) を実装
        - 事前にコンパイルした .dtb ファイルをバイナリとして読み込む
        - もしくは `device_tree` クレートを使う
            - ~~クレートを使ったら負け説~~
    - 終わったら実行
        - OpenSBI (RISC-V のベアメタルプログラムが Linux カーネルを起動するためのブートローダー) を動かしてみる🤩
            - 成功したらロゴのアスキーアートが出てくるらしい🤩
        - Linux が動くようになる🤩
