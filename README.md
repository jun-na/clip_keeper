# ClipKeeper

Windows 向け軽量クリップボード履歴アプリです。  
タスクトレイに常駐し、ホットキーで履歴をすばやく呼び出せます。

macOS ではローカルビルドして、メニューバー常駐アプリとして利用できます。

## ダウンロード・インストール

1. [Releases](https://github.com/jun-na/clip_keeper/releases) から最新の `ClipKeeper.exe` をダウンロードしてください。
2. 任意のフォルダに置くだけで使えます。インストーラーは不要です。
3. `ClipKeeper.exe` を実行するとタスクトレイに常駐します。

## macOS ローカル実行

`cargo run` で起動するとターミナル上のプロセスとして実行されるため、ターミナル表示は消えません。
macOS でターミナルを表示せずに使う場合は、`.app` バンドルを生成して Finder から起動してください。

1. `zsh scripts/build-macos-app.sh`
2. `target/release/ClipKeeper.app` を Finder から開く

メニューバー常駐アプリとして起動し、Dock には表示されません。

## 主な機能

- コピーしたテキストを自動的に最大 50 件の履歴として保存
- 保存アイテム管理（タイトル・本文・グループ）
- グローバルホットキーで履歴ウィンドウを表示
  - Shift ダブルタップ（既定）
  - Ctrl ダブルタップ
  - Ctrl / Shift + 任意キーの組み合わせ
- タスクトレイメニュー（履歴を開く / 設定 / 終了）
- 履歴ウィンドウはリサイズ可能

## データ保存先

Windows では設定と履歴は `ClipKeeper.exe` と同じフォルダに自動保存されます。アンインストール時はフォルダごと削除してください。

- `ClipKeeper.exe` と同じフォルダ内の `clipboard_history.json`
- `ClipKeeper.exe` と同じフォルダ内の `settings.json`

macOS では以下に保存されます。

- `~/Library/Application Support/ClipKeeper/clipboard_history.json`
- `~/Library/Application Support/ClipKeeper/settings.json`

## 動作環境

- Windows 10 / 11

## ライセンス

このリポジトリは `LICENSE` に従います。
