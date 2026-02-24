# ClipboardTool (clip_keeper) - 機能とファイル関連マップ

## 機能と関連ファイル

### クリップボード履歴管理
- `src/app/services/clipboard_service.rs` - 履歴の追加・取得・永続化
- `src/app/states/app_state.rs` - 履歴状態（VecDeque、最大50件）
- `src/app/services/monitor_runtime.rs` - 120ms ポーリング監視
- `ui/app-window.slint` - 履歴表示UI

### ホットキー検出
- `src/app/services/monitor_runtime.rs` - グローバルキー監視
- `src/app/services/detectors.rs` - ダブルタップ検出（450ms 判定）
- `src/app/services/settings_service.rs` - 設定読み書き
- `src/app/states/settings_state.rs` - 設定状態

### UI表示・操作
- `ui/app-window.slint` - HistoryWindow / SettingsWindow
- `src/app/services/ui_gateway.rs` - Rust ↔ Slint 双方向通信
- `src/main.rs` - Window 生成・ライフサイクル

### タスクトレイ
- `src/app/services/tray_runtime.rs` - メニュー・アイコン生成
- `src/app/services/ui_gateway.rs` - メニュー → UI 橋渡し

### 依存関係・初期化
- `src/app/contexts/composition_root.rs` - DI コンテナ
- `src/app/contexts/service_context.rs` - Service 定義
- `src/app/contexts/state_context.rs` - State 管理
- `src/main.rs` - 起動シーケンス

---

## ファイル役割早見表

| ファイル | 責務 |
|---------|------|
| `main.rs` | 起動・初期化 |
| `clipboard_service.rs` | 履歴 CRUD・永続化 |
| `settings_service.rs` | 設定 CRUD・永続化 |
| `ui_gateway.rs` | UI 通信集約 |
| `monitor_runtime.rs` | 監視ループ（クリップボード・ホットキー） |
| `tray_runtime.rs` | タスクトレイ |
| `app_state.rs` | 履歴状態 |
| `settings_state.rs` | 設定状態 |
| `detectors.rs` | ダブルタップ判定 |
| `app-window.slint` | UI 定義 |
| `composition_root.rs` | DI コンテナ |
| `service_context.rs` | Service インスタンス化 |
| `state_context.rs` | State 中央管理 |

---

## 初期化順序

```
main() 
  ↓ CompositionRoot::build()
  ↓ load_history_from_disk()
  ↓ load_from_disk() (settings)
  ↓ Window 生成
  ↓ ServiceRuntime::new()
  ↓ MonitorRuntime::start()
  ↓ slint::run_event_loop_until_quit()
```

---

## マルチスレッド構成

- **Main Thread**: Slint UI イベント
- **Clipboard Thread**: 120ms ポーリング
- **Hotkey Thread**: グローバルキー監視
- **Tray Menu Thread**: タスクトレイメニュー監視

すべて `Arc<Mutex<>>` で同期保護

---

## 永続化ファイル

- `%LOCALAPPDATA%/clip_keeper/clipboard_history.json` - 履歴
- `%LOCALAPPDATA%/clip_keeper/settings.json` - 設定
