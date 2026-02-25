# ClipKeeper

Windows 向け軽量クリップボード履歴アプリです。  
タスクトレイに常駐し、ホットキーで履歴をすばやく呼び出せます。

## ダウンロード・インストール

1. [Releases](https://github.com/jun-na/clip_keeper/releases) から最新の `ClipKeeper.exe` をダウンロードしてください。
2. 任意のフォルダに置くだけで使えます。インストーラーは不要です。
3. `ClipKeeper.exe` を実行するとタスクトレイに常駐します。

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

設定と履歴は `ClipKeeper.exe` と同じフォルダに自動保存されます。アンインストール時はフォルダごと削除してください。

- `ClipKeeper.exe` と同じフォルダ内の `clipboard_history.json`
- `ClipKeeper.exe` と同じフォルダ内の `settings.json`

## 動作環境

- Windows 10 / 11

## ライセンス

このリポジトリは `LICENSE` に従います。
