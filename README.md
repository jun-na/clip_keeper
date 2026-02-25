# ClipKeeper

ClipKeeper は、Windows 向けの軽量クリップボード履歴アプリです。  
Rust + Slint で実装されており、タスクトレイ常駐で履歴呼び出しを行えます。

## 主な機能

- クリップボード履歴の収集・表示・永続化
- 保存アイテム（タイトル/本文）管理
- グループ管理
- グローバルホットキーで履歴ウィンドウを表示
  - Shift ダブルタップ
  - Ctrl ダブルタップ
  - 組み合わせキー（Ctrl/Shift + 任意キー）
- タスクトレイ常駐（履歴を開く / 設定 / 終了）
- 履歴ウィンドウのリサイズ対応

## 動作環境

- OS: Windows
- Rust (stable)

## セットアップ

```powershell
cargo build
```

## 実行

```powershell
cargo run
```

## リリースビルド

```powershell
cargo build --release
```

生成物:

- `target/release/ClipKeeper.exe`

## 設定/データ保存先

- `%LOCALAPPDATA%/clip_keeper/clipboard_history.json`
- `%LOCALAPPDATA%/clip_keeper/settings.json`

## アイコン

- Windows 実行ファイルアイコン: `assets/app-icon.ico`
- トレイ/ウィンドウアイコン: `assets/tray-icon.rgba`

## リリースノート

- `v0.9.0`: [RELEASE_NOTES_v0.9.0.md](RELEASE_NOTES_v0.9.0.md)

## ライセンス

このリポジトリは `LICENSE` に従います。
