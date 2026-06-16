# Templazy Desktop (Tauri v2)

現行の「Python + ブラウザ」版を、軽量ネイティブアプリ化したもの。
UI（`src/index.html`）は既存のバニラJSを流用し、データ保存だけ Rust 側コマンドに置き換えている。

## 構成
```
templazy-desktop/
├── src/index.html          # フロントエンド（既存UI流用。Apiレイヤのみ Tauri invoke 化）
└── src-tauri/
    ├── Cargo.toml
    ├── tauri.conf.json     # frontendDist=../src（Node/devサーバ不要）
    ├── capabilities/default.json
    ├── icons/              # アプリアイコン（make_icons.py から生成済み）
    ├── seed_templates.json # 初回起動時に取り込む初期テンプレ（既定は空 []）
    └── src/main.rs         # load/save コマンド + seed初期化
```

## 前提ツール（ビルドする人のみ）
- **Rust / cargo**（導入済み）
- **Tauri CLI**：`cargo install tauri-cli --version "^2"`（初回のみ・コンパイルに少し時間）
- 各OSの WebView ランタイム
  - Windows: **WebView2**（多くのPCに既存。無ければMSのランタイムを導入）
  - macOS: WKWebView（OS標準・追加不要）
  - Linux: `libwebkit2gtk-4.1-dev` 等（開発時）

### 社内 Artifactory 経由（cargo）
`~/.cargo/config.toml` に社内ミラーを設定：
```toml
[source.crates-io]
replace-with = "artifactory"

[source.artifactory]
registry = "sparse+https://<host>/artifactory/api/cargo/<repo>/index/"
```
※ 本構成は **Node不要**なので npm レジストリの問題は発生しない。

## 開発・ビルド
```bash
cd src-tauri
cargo tauri dev      # 開発起動（ホットでウィンドウが立つ）
cargo tauri build    # 配布物を生成
```
生成物：
- Windows: `.msi` / NSIS `.exe`（`src-tauri/target/release/bundle/…`）
- macOS: `.app` / `.dmg`（**.app なので Finder/Dock にアイコンが出る**）
- Linux: `.deb` / AppImage

> クロスビルドは不可（Tauriも同様）。**mac版はMacで、win版はWindowsで**ビルドする。

## 初期テンプレート（seed）
`src-tauri/seed_templates.json` に**テンプレートの配列(JSON)**を入れてビルドすると、
利用者の初回起動時に取り込まれる（2回目以降や既存ユーザーには影響しない）。
- 既定は `[]`（空）。
- 社内向け内容を載せたくない場合は、ビルド直前に中身を差し替え、リポジトリにはコミットしない運用に。
- 旧版の `data/personal.json` をそのまま貼り付ければ移行できる。

## アイコン
`../make_icons.py` の描画を流用して `src-tauri/icons/` に生成済み
（`32x32.png` / `128x128.png` / `128x128@2x.png` / `icon.icns` / `icon.ico`）。
デザイン変更時は `make_icons.py` を編集 → アイコンを再生成。

## データ保存先（OSユーザーごとに分離）
- Windows: `%APPDATA%\jp.templazy.app\`
- macOS: `~/Library/Application Support/jp.templazy.app/`
- Linux: `~/.local/share/jp.templazy.app/`
ファイル：`personal.json`（テンプレ）/ `tabs.json`（作業状態）。書き込みはアトミック。

## 状態 / TODO
- [x] 雛形（Rust load/save + seed、tauri.conf、capabilities、アイコン）
- [x] 既存UI流用・Apiレイヤを invoke 化
- [x] 依存解決（Cargo.lock）確認済み
- [x] **起動確認（Linux / WebKitGTK）**：`cargo build` → 起動 → ウィンドウ表示 → クリーン終了
- [x] **データ層動作確認**：テンプレ作成→`personal.json`/`tabs.json` に保存（ユーザー別領域・アトミック）
- [x] **リッチコピー → 貼り付けOK（Linux / WebKitGTK で確認）** ← 移行の最大リスクをクリア
- [ ] Windows(WebView2) / macOS(WKWebView) でのリッチコピー実機確認
  - WebView2(Win) は Chromium ベースで効く可能性大。
  - WKWebView(mac) はクリップボード制約が出ることがあるため要確認。不可なら Tauri clipboard プラグイン or Rust 実装にフォールバック。
- [x] **パッケージング検証（Linux）**：`cargo tauri build --bundles deb` → `Templazy_0.1.0_amd64.deb`（インストール ~3.5MB、アイコン＋.desktop 同梱）
- [ ] Windows で `cargo tauri build` → `.msi`/`.exe`（ビルド機で実施）
- [ ] macOS で `cargo tauri build` → `.app`/`.dmg`（ビルド機で実施）
- [ ] 受け入れ条件（PLAN.md 第4章）を順に検証

### Linux配布物の作り方（確認済み手順）
```bash
cargo install tauri-cli --version "^2"      # 初回のみ
cd src-tauri
cargo tauri build --bundles deb             # → target/release/bundle/deb/*.deb
```
インストール: `sudo apt install ./Templazy_0.1.0_amd64.deb`（依存 webkit2gtk/gtk は apt が解決）
