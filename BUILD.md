# Templazy 開発・ビルドガイド

開発者／配布物を作る人向け。利用者向けの説明は [README.md](README.md) を参照。

Tauri v2 製。UI（`src/index.html`）は依存ゼロのバニラJSで、データ保存だけ Rust 側コマンド（`load`/`save`）に置き換えている。**Node/バンドラ不要**。

## 構成
```
templazy-desktop/
├── src/index.html              # フロントエンド（Apiレイヤのみ Tauri invoke 化）
├── .github/workflows/build.yml # 各OSビルドのCI
└── src-tauri/
    ├── Cargo.toml
    ├── tauri.conf.json         # frontendDist=../src（devサーバ不要）
    ├── capabilities/default.json
    ├── icons/                  # アプリアイコン
    ├── seed_templates.json     # 初期テンプレ（既定は空 []。CIでは常に空でビルド）
    └── src/main.rs             # load/save コマンド + seed初期化
```

## 前提ツール
- **Rust / cargo**（rustup）
- **Tauri CLI**：`cargo install tauri-cli --version "^2"`
- 各OSの WebView / ビルド依存
  - Windows: **Visual Studio Build Tools（C++）**＋ 実行には **WebView2**（Win10/11は標準搭載）
  - macOS: WKWebView（OS標準）
  - Linux: `libwebkit2gtk-4.1-dev build-essential libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev`

## 開発
```bash
cd src-tauri
cargo tauri dev      # ホットリロードでウィンドウ起動
```

## ビルド（配布物）
```bash
cd src-tauri
cargo tauri build                    # 既定の全バンドル
cargo tauri build --bundles deb      # 個別指定も可（msi / nsis / dmg / deb）
```
生成物（`src-tauri/target/release/bundle/…`）：
- Windows: `.msi`（WiX）/ `setup.exe`（NSIS）
- macOS: `.app` / `.dmg`（**.app なので Dock/Finder にアイコンが出る**）
- Linux: `.deb`

> **クロスビルド不可**：mac版はMacで、win版はWindowsで、Linux版はLinuxでビルドする。

### Windows 手順（要点）
1. **Visual Studio Build Tools** の「C++ によるデスクトップ開発」を入れる（リンカ必須）
2. rustup（既定の MSVC ツールチェーン）
3. `cargo install tauri-cli --version "^2"`
4. `cd src-tauri && cargo tauri build`
- 初回は WiX / NSIS を GitHub から自動取得（社外遮断環境だと失敗。通信許可 or 事前取得が必要）。

### Linux 手順（確認済み）
```bash
sudo apt install -y libwebkit2gtk-4.1-dev build-essential curl wget file libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev
cargo install tauri-cli --version "^2"
cd src-tauri && cargo tauri build --bundles deb
```

## CI（GitHub Actions）
`.github/workflows/build.yml` で Windows / macOS(Intel・ARM) / Linux を一括ビルド。
- **手動実行**（Actions → Build Templazy → Run workflow）または **`v*` タグ push** で起動
- 成果物は各実行の **Artifacts** からダウンロード
- **seed は CI では常に空 `[]`**（社内テンプレを載せない）。テンプレは利用者が「JSONインポート」で取り込む
- ⚠ ワークフローは **リポジトリ直下の `.github/workflows/`** のみ有効。`templazy-desktop` を**リポジトリのルート**にして push すること

## 社内 Artifactory 経由（cargo）
`~/.cargo/config.toml`（Windowsは `%USERPROFILE%\.cargo\config.toml`）：
```toml
[source.crates-io]
replace-with = "artifactory"

[source.artifactory]
registry = "sparse+https://<host>/artifactory/api/cargo/<repo>/index/"
```
本構成は Node不要なので npm レジストリ問題は発生しない。

## 初期テンプレート（seed）
`src-tauri/seed_templates.json` に**テンプレ配列(JSON)**を入れてビルドすると、利用者の初回起動時（`personal.json` が無いときだけ）に取り込まれる。
- 既定は `[]`。CIでは常に空でビルド。
- 社内テンプレは**配布物に焼き込まず**、利用者がアプリの「JSONインポート」で取り込む運用を推奨。

## アイコン
親フォルダの `make_icons.py` で `src-tauri/icons/`（`32x32.png` / `128x128.png` / `128x128@2x.png` / `icon.icns` / `icon.ico`）を生成済み。デザイン変更時は `make_icons.py` を編集して再生成。

## データ保存先（OSユーザーごと）
- Windows: `%APPDATA%\jp.templazy.app\`
- macOS: `~/Library/Application Support/jp.templazy.app/`
- Linux: `~/.local/share/jp.templazy.app/`
ファイル：`personal.json`（テンプレ）/ `tabs.json`（作業状態）。書き込みはアトミック（temp→rename）。

## 状態 / TODO
- [x] 雛形（Rust load/save + seed、tauri.conf、capabilities、アイコン）
- [x] 既存UI流用・Apiレイヤを invoke 化
- [x] 起動・データ層・リッチコピー（Linux/WebKitGTK）確認
- [x] パッケージング検証（Linux `.deb`、~3.5MB、アイコン＋.desktop）
- [x] 各OSビルド CI（GitHub Actions）
- [ ] Windows(WebView2) / macOS(WKWebView) でのリッチコピー実機確認
- [ ] Windows `.msi`/`.exe` / macOS `.app`/`.dmg` のビルド機での生成確認
- [ ] （任意）コード署名・notarization、自動更新
