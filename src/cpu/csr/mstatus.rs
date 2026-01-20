// --- Interrupt Enables (割り込み許可フラグ) ---

/// Supervisor Interrupt Enable: Sモードでの割り込みを許可する
pub const SIE:  u64 = 1 << 1;

/// Machine Interrupt Enable: Mモードでの割り込みを許可する
pub const MIE:  u64 = 1 << 3;

/// Supervisor Previous Interrupt Enable: Sモードへトラップした際の、直前のSIEの値
pub const SPIE: u64 = 1 << 5;

/// Machine Previous Interrupt Enable: Mモードへトラップした際の、直前のMIEの値
pub const MPIE: u64 = 1 << 7;

// --- Previous Privileges (トラップ前の特権モード) ---

/// Supervisor Previous Privilege: Sモードへトラップした際の、直前の特権モード (0=U, 1=S)
pub const SPP:  u64 = 1 << 8;

/// Machine Previous Privilege: Mモードへトラップした際の、直前の特権モード (00=U, 01=S, 11=M)
pub const MPP:  u64 = 0b11 << 11;

// --- Context Status (コンテキスト状態: FPU/Vectorなど) ---

/// Vector Status: ベクトル拡張レジスタの状態 (00=Off, 01=Initial, 10=Clean, 11=Dirty)
pub const VS:   u64 = 0b11 << 9;

/// Floating-point Status: 浮動小数点レジスタの状態 (00=Off, 01=Initial, 10=Clean, 11=Dirty)
pub const FS:   u64 = 0b11 << 13;

/// Extension Status: その他のユーザー拡張機能の状態
pub const XS:   u64 = 0b11 << 15;

// --- Memory / Execution Control (メモリ・実行制御) ---

/// Modify PRiVilege: ロード・ストア実行時の権限チェックをMPPのモードで行う (通常0)
pub const MPRV: u64 = 1 << 17;

/// Permit Supervisor User Memory access: SモードがUモードのメモリページにアクセスすることを許可する
pub const SUM:  u64 = 1 << 18;

/// Make eXecutable Readable: 実行可能(X=1)ページをロード命令で読み出し可能(R=1扱い)にする
pub const MXR:  u64 = 1 << 19;

/// Trap Virtual Memory: Sモードでのsatpレジスタ変更やSFENCE.VMA実行を例外(トラップ)にする
pub const TVM:  u64 = 1 << 20;

/// Timeout Wait: WFI命令を一定時間でタイムアウト(Sモードへトラップ)させる (通常0)
pub const TW:   u64 = 1 << 21;

/// Trap SRET: Sモードでのsret命令実行を例外(トラップ)にする
pub const TSR:  u64 = 1 << 22;

// --- XLEN (ビット幅制御) ---

/// User XLEN: Uモードのレジスタビット幅 (01=32bit, 10=64bit)
pub const UXL:  u64 = 0b11 << 32;

/// Supervisor XLEN: Sモードのレジスタビット幅 (01=32bit, 10=64bit)
pub const SXL:  u64 = 0b11 << 34;

// --- Summary Bit (要約ビット) ---
/// State Dirty: FS, VS, XS のいずれかが Dirty(11) であることを示す (読み取り専用)
pub const SD:   u64 = 1 << 63;

#[derive(Clone, Copy)]
pub struct Extensions {
    /// F/D 拡張を持っているか
    pub has_fpu: bool,
    /// V 拡張を持っているか
    pub has_vector: bool,
    /// 64bit モードか
    pub is_rv64: bool,
    // TODO: has_hypervisor etc...
}

/// mstatus ラッパー
pub struct Mstatus {
    raw: u64,
}
impl Mstatus {
    pub const fn new(val: u64) -> Self {
        Self { raw: val }
    }

    /// mstatus に値を書き込みます。
    pub const fn write(&mut self, val: u64, ext: Extensions) -> &mut Self {
        let mask = self.make_write_mask(ext);

        // NOTE: 書き込み可能な部分だけ更新し、残りは元の値を維持
        let mut next_val = (self.raw & !mask) | (val & mask);

        if ext.is_rv64 {
            // NOTE: UXL = 2 (0b10), SXL = 2 (0b10) -> bits: 10
            next_val = (next_val & !UXL) | (0b10 << 32);
            next_val = (next_val & !SXL) | (0b10 << 34);
        }

        self.raw = next_val;
        self
    }

    /// mstatus の値を読み取ります。
    pub const fn read(&self) -> u64 {
        let mut val = self.raw;

        // NOTE: SD ビットの計算: (FS==11) OR (VS==11) OR (XS==11) なら 1
        let fs_dirty = (val & FS) == FS;
        let vs_dirty = (val & VS) == VS;
        let xs_dirty = (val & XS) == XS;

        if fs_dirty || vs_dirty || xs_dirty {
            val |= SD;
        } else {
            val &= !SD;
        }
        val
    }

    /// 機能フラグに基づいて「書き込み可能なビット」のマスクを作成します。
    const fn make_write_mask(&self, ext: Extensions) -> u64 {
        // NOTE: 常に書き込み可能な基本ビット
        let mut mask = SIE | MIE | SPIE | MPIE | SPP | MPP
                        | MPRV | SUM | MXR | TVM | TW | TSR;

        if ext.has_fpu {
            mask |= FS;
        }
        if ext.has_vector {
            mask |= VS;
        }

        mask
    }
}
