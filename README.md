# Starframe FITS

**FITS画像を直接開ける、Windows向けの画像編集アプリ。**

PhotoCraft v0.3.0を基に、FITS読み込みと32-bit GPU表示の改善を加えた独立した派生版です。
PhotoCraft／ArtCraftの公式配布版ではありません。MIT OR Apache-2.0。

## ダウンロード・インストール

**[Windowsインストーラーをダウンロード](https://github.com/Aoi-Morio/starframe-fits/releases/latest/download/starframe-fits-0.1.0-windows-x64-setup.exe)**

1. ダウンロードしたファイルを開き、日本語または英語を選びます。
2. インストール後、スタートメニューの **Starframe FITS** を開きます。
3. **ファイル → 開く** で `.fits` / `.fit` / `.fts` を選びます。

管理者権限、Rust、Python、Visual C++再頒布パッケージの追加導入は不要です。
Windows 10/11の64-bit x64版を対象としています。Windows ARM、32-bit版、macOS、Linuxは今回の配布対象外です。

インストールしたくない場合は **[ポータブル版](https://github.com/Aoi-Morio/starframe-fits/releases/latest/download/starframe-fits-0.1.0-windows-x64-portable.zip)** を展開し、`StarframeFITS.exe` を起動してください。
同梱の `demo-stars.fits` で動作を試せます。

> **Windowsの署名について**：初回リリースは未署名です。SmartScreenの警告や、Smart App Control／組織のポリシーによる実行拒否が起きる場合があります。SHA256SUMSで改変の確認はできますが、Windowsの署名を代替するものではありません。保護機能の無効化は案内していません。拒否された場合は署名済みの配布版を待つか、管理者に確認してください。

## FITSの対応範囲

| 項目 | v0.1.0 |
|---|---|
| 拡張子 | `.fits` / `.fit` / `.fts` |
| 画像 | Primary HDUの2D白黒、3面RGB（NAXIS3=3） |
| BITPIX | 8 / 16 / 32 / -32 |
| 画素 | BSCALE/BZEROを適用し、32-bit floatへ変換 |
| 無効値 | BLANK / NaN / Infinityは0へ置換し、件数を通知 |
| 上下方向 | ROWORDERを尊重。指定なしはFITSの下から上の順序 |
| 保存 | `.pcraft`、PSD、TIFFなど。本アプリからFITS書き出しは未対応 |

元の物理値はBackgroundレイヤーに保持します。表示範囲を超える値には、解除できる **FITS display scaling** 露出調整を追加します。
整数値や非常に小さい値でもGPU転送時に16-bitへ精度を落としません。
ただし整数や倍精度のスケーリング結果がf32へ丸められる場合は通知します。

**暗く見えるスタック画像**は線形画像である場合があります。取り込みエラーとは限りません。露出やガンマを調整するか、天体画像処理ソフトでストレッチした32-bit TIFFを使ってください。
非線形の天体画像用自動ストレッチ、Bayerのカラー化、WCS／天文ヘッダーの保存、拡張HDUの選択、圧縮FITS、一般的な多次元キューブ、BITPIX 64/-64は未対応です。
3面画像はRGBと解釈するため、任意の3枚の科学データをカラーにしたい用途では注意してください。

## 使い方・困ったとき

- 白黒のBayer画像は自動でカラーになりません。
- 編集結果は別名で `.pcraft` に保存すると調整レイヤーを保持できます。
- インストール版の設定は `%APPDATA%\StarframeFITS`、ポータブル版は実行ファイル横の `StarframeData` に保存されます。
- 旧PhotoCraftの設定をコピーせず、新規設定で起動してください。CPU互換モードでは大きな画像の操作が重くなる場合があります。
- 更新は新しいインストーラーを実行します。アンインストールはWindowsの「インストールされているアプリ」から行います。画像ファイルや設定は削除しません。
- 不具合報告は [Issues](https://github.com/Aoi-Morio/starframe-fits/issues) へ。個人画像やFITSヘッダーは、公開してよい内容か確認してから添付してください。

## 品質・制限

[検証結果と公開前チェック](docs/starframe-quality.md)、[FITS仕様](docs/fits-import.md)、[変更履歴](CHANGELOG.md)。
FITS取り込み、実画像の読み込み、保存・再読み込み、GPUとCPUの表示比較を確認しています。
ホストアプリは開発途上のため、多数の文字・レイヤー編集の重さなど既存の制限が残ります。
精密な科学解析や天文メタデータの保存を目的としたアプリではありません。

## English

Starframe FITS is an independent Windows image editor fork of [PhotoCraft](https://github.com/storytold/photocraft), with FITS image import and full-float GPU upload improvements. It is not an official ArtCraft release.

Download the [Windows x64 installer](https://github.com/Aoi-Morio/starframe-fits/releases/latest/download/starframe-fits-0.1.0-windows-x64-setup.exe), or extract the [portable ZIP](https://github.com/Aoi-Morio/starframe-fits/releases/latest/download/starframe-fits-0.1.0-windows-x64-portable.zip) and launch `StarframeFITS.exe`.
No administrator rights or separately installed VC++ runtime are required. The initial release is **unsigned**: SmartScreen may warn and Smart App Control may block it. Checksums do not replace trusted code signing; do not disable protection to run it.

Supported: Primary HDU, 2D grayscale / three-plane RGB, BITPIX 8/16/32/-32, BSCALE/BZERO and ROWORDER. Physical values are kept in a 32-bit Background layer, with an optional removable display exposure adjustment. This does not implement FITS export, WCS preservation, general cubes, compressed images, Bayer demosaicing or automatic astronomical nonlinear stretching.

## Build and attribution

The Rust workspace retains upstream crate and format identifiers for compatibility. See [build instructions](packaging/starframe/README.md).
Upstream base: `storytold/photocraft`, v0.3.0, commit `bd632e7eb33c601fbeee7aa19c77b1859f02a7e7`.
Original copyright, dual licence and third-party attribution are preserved in [NOTICE](NOTICE), [LICENSE-MIT](LICENSE-MIT), [LICENSE-APACHE](LICENSE-APACHE) and [ATTRIBUTION.md](ATTRIBUTION.md).
ArtCraft logos have been removed; the app icon is new original geometric artwork.
