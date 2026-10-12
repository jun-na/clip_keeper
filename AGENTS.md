# AI指示ガイドライン - entraitと共有状態

## 設計

- `App`：共有状態と外部依存を保持するアプリの依存コンテナ
- `AppState`：履歴・設定・ホットキー判定・ログの有効状態・画面の弱参照を中央管理。変更可能な状態は `Arc<Mutex<T>>` で保護
- `Application = entrait::Impl<App>`：entraitが生成したサービスtraitの実装
- `app::build()`：Composition Root。共有状態の生成・復元と実行系の組み立てを集約
- `ApplicationRuntime`：Appの外側で `Arc<Impl<App>>` とMediatorを保持する実行系。MediatorがControllerを保持

サービスは状態を持たない関数として `services/` に配置。モジュールの `#[entrait::entrait(pub ServiceName)]` でtraitを生成し、依存は引数のtrait境界で宣言する。

## 初期化順序

1. AppStateを生成し、Appに共有状態を保持
2. `Arc<Impl<App>>` を生成
3. 同じ実装で履歴・設定を復元。読み込み失敗時はエラーを記録して起動を継続
4. Controllerへ実装を注入し、Mediator・ApplicationRuntimeを組み立て
5. UiServiceで画面を生成・接続し、TrayServiceでトレイを生成し、MonitorRuntimeで監視を開始。ServiceRuntimeで各セッションを保持

## 実装例

```rust
#[entrait::entrait(pub HistoryService)]
mod operations {
    use crate::app::AccessState;

    pub fn selected_index(deps: &impl AccessState) -> i32 {
        deps.state()
            .clipboard_state
            .lock()
            .expect("app state lock poisoned")
            .selected_index()
    }
}
```

## 設計の契約

- 状態は `AccessState::state()` 経由で取得するAppStateに集約
- サービスとControllerは必要な依存をtrait境界で宣言
- ControllerはRequestの振り分けとサービスの実行順序を担当し、Slintや共有状態を直接操作しない
- UiServiceは画面生成・登録・表示・非表示・更新・フォーカス・表示モデルへの変換・入力コールバック接続を担当。画面の弱参照はAppStateに保持し、画面操作はUIスレッド上で実行
- UiService・TrayRuntime・MonitorRuntimeは注入されたRequestSenderにRequestを渡す。入力側からApplicationRuntimeやMediatorへの具体型依存を持たせない
- runtimeはサービスの起動とUiSession・TraySession・TrayRuntime・MonitorRuntimeの保持を担当。Window・トレイを直接生成・操作せず、ログ出力は実装しない。監視の周期・スレッド・OSリスナー・トレイイベント待ち・Requestへの入力変換はruntimeで管理し、履歴更新やホットキー判定はサービスへ委譲する
- ログの有効状態はAppState、出力先はAppに保持し、LoggingServiceからAccessState・AccessLogPath経由で取得する
- ステートレスなサービスのインスタンスやServiceContextを追加しない
- 新しい共有状態・外部依存はAppと `app::build()` で組み立てる
- ApplicationRuntimeはMediatorに転送するRequestSenderをUiService・TrayRuntime・MonitorRuntimeへ注入
- AppからController・Mediator・ApplicationRuntimeへの強参照を持たせない
- `contexts/` は設けず、共有状態は `state.rs`、実行系は `runtime/` に配置
