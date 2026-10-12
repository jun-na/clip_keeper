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

## リクエスト処理の構造

画面入力・トレイ操作・120ms周期のクリップボード監視・ホットキー入力を `Request` に変換し、`Mediator` を通じて単一の `Controller` へ送信。

```text
画面入力   → RequestSender → Mediator.send ─┐
トレイ操作 → Mediator.send ────────────────┼→ Slintイベントループ → Controller.dispatch → 各Service → UiServiceで画面更新
周期監視   → Mediator.send ────────────────┤
ホットキー → Mediator.send ────────────────┘
```

- `src/app/request.rs`: リクエストの種類と引数
- `src/app/mediator.rs`: イベントループへの転送
- `src/app/controller.rs`: リクエストのディスパッチとサービスの実行順序の制御
- `UiService`: 画面の生成・コールバック登録・画面操作。入力は注入された `RequestSender` へ送信
- `MonitorRuntime`: 120ms周期のクリップボード監視・ホットキー入力取得・Request送信
- `TrayService`: トレイ生成
- `TrayRuntime`: トレイのイベント待ち・メニュー入力のRequest送信
- `LoggingService`: ホットキー発動ログの出力（既定は無効）
- `AppState`: 履歴・設定・ホットキー判定状態・画面の弱参照を管理

`app::build()` で共有状態を生成・復元し、Controller・Mediatorを組み立て、イベントループ上でリクエストを逐次処理。ホットキーのダブルタップ判定には入力取得時の時刻を使用。

## Controller・Service・Domain

機能単位の `features/` を層単位の配置へ変更。

```text
src/app/
├── controller.rs         # リクエストの受付・ディスパッチ・サービスの連携
├── services/
│   ├── clipboard.rs      # 履歴操作の連携・保存・OSクリップボード・貼り付け
│   ├── settings.rs       # 設定の保存・ホットキー判定の呼び出し
│   ├── ui.rs             # 画面生成・コールバック登録・画面操作・表示モデルへの変換
│   ├── tray.rs           # トレイ生成
│   └── logging.rs        # ホットキー発動ログの出力
├── domain/
│   ├── clipboard.rs      # ClipboardState・履歴と保存グループのルール
│   ├── settings.rs       # SettingsState・設定値の正規化
│   └── hotkey.rs         # キー入力型・ダブルタップとキー組み合わせの判定
├── mod.rs                # App・Impl<App>・buildによる依存解決
├── state.rs              # AppState・共有状態
├── platform/             # OS入力からDomainの型への変換
└── runtime/              # サービスの起動・セッション保持・OS入力の監視とRequest送信
```

ControllerはRequestを振り分け、ClipboardService・SettingsService・UiService・LoggingServiceの実行順序を制御する。ClipboardServiceとSettingsServiceがAppState内のDomainを操作し、UiServiceがSlintの表示モデルへの変換・画面の生成・登録・表示・非表示・更新・フォーカス・入力コールバック接続を担当。画面操作はUIスレッド上で実行し、入力をRequestに変換して `RequestSender` へ送信する。Domainは標準ライブラリと同じDomain内の型だけに依存。

ホットキーの経路は `OS入力 → KeyEvent → Request → Mediator → Controller → SettingsService → HotkeyState`。判定結果をControllerへ返し、UiServiceで履歴画面を表示。

## entrait による依存注入

`services/` の各モジュールにentrait属性を適用し、`ClipboardService`・`SettingsService`・`UiService`・`TrayService`・`LoggingService` のtraitを生成。公開関数からサービスtraitと `Impl<App>` に対する実装を生成する。各関数の依存は必要なサービスと `AccessState` のtrait境界で宣言し、`deps.state()` から共有状態にアクセスする。ログ出力先はAppに保持し、`AccessLogPath` から取得する。TrayRuntimeとMonitorRuntimeは注入された `RequestSender` で入力を送信する。

`app::build()` をComposition Rootとして、次の順に組み立てる。

1. `AppState` を生成し、`App` に `Arc<AppState>` として保持
2. `Arc<Impl<App>>` を生成し、履歴・設定をディスクから復元（読み込み失敗時は初期状態で継続）
3. 同じ `Arc<Impl<App>>` をControllerへ注入し、MediatorとApplicationRuntimeを生成

Controllerは `ClipboardService + SettingsService + UiService + LoggingService` のtrait境界で依存を受け取り、Slintや共有状態を直接操作しない。サービスは状態を持たない関数で、AppState内の `Arc<Mutex<T>>` をロックしてDomainを操作する。

ApplicationRuntimeはAppの外側で `Arc<Impl<App>>` とMediatorを保持し、MediatorがControllerを保持。UiService・TrayRuntime・MonitorRuntimeにMediatorへ転送する共通の `RequestSender` を注入する。サービスから実行系への具体型依存を持たせず、Appから実行系への強参照も持たせない。

ServiceRuntimeは `UiSession`・`TraySession`・`TrayRuntime`・`MonitorRuntime` を保持し、サービスの初期化・監視開始のタイミングを管理する。`main` は `ServiceRuntime::new(app)` と `start_background_services()` で起動する。監視開始は同じruntimeに対して1回だけ実行する。

クリップボード監視はMonitorRuntime内の別スレッドで120ms（0.12秒）周期。ホットキーはOSリスナーで取得する。監視スレッドは従来どおりプロセス終了まで動き、MonitorRuntimeの破棄だけでは停止しない。ログの有効状態はAppStateに保持し、既定ではファイルに書き込まない。

監視の周期・スレッド・OSリスナーはruntimeで管理し、取得した入力をRequestへ変換する。履歴更新・ホットキー判定・ログ出力はControllerからサービスへ委譲する。
