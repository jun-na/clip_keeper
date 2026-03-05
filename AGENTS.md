# AI指示ガイドライン - 依存注入とコンテキスト設計パターン

## AIへの指示テンプレート

新しいプロジェクトでこのパターンを使う場合、AIに以下を指示してください：

> このプロジェクトでは、以下の設計パターンを採用しています：
>
> 1. **AppContext**: StateContextとServiceContextを統合するコンテナ
> 2. **StateContext**: すべてのアプリケーション状態を中央管理（`Arc<Mutex<T>>`でラップ）
> 3. **ServiceContext**: ビジネスロジックを実装するサービスを管理
> 4. **Composition Root**: DIコンテナとして、すべてのコンテキストとサービスを組み立てる
> 5. **依存注入**: 各Serviceはコンストラクタを通じて必要な依存（StateやService）を受け取る
>
> 新しいServiceやStateを追加する際は、このパターンに従い、Composition Rootで依存を解決してください。

## 初期化順序

1. StateContext を生成（すべての状態を先に用意）
2. 依存関係のないServiceから順に生成（下流から上流へ）
3. ServiceContext に集約（すべてのServiceをまとめる）
4. AppContext で統合（StateContextとServiceContextを束ねる）

## 実装例

```rust
// コンストラクタインジェクション
pub struct MyService {
    state_context: Arc<StateContext>,
    other_service: Arc<OtherService>,
}

impl MyService {
    pub fn new(state_context: Arc<StateContext>, other_service: Arc<OtherService>) -> Self {
        Self { state_context, other_service }
    }
}

// Composition Root
pub fn create_app_context() -> Arc<AppContext> {
    let state_context = Arc::new(StateContext::new());
    let service_a = Arc::new(ServiceA::new(state_context.clone()));
    let service_b = Arc::new(ServiceB::new(state_context.clone(), service_a.clone()));
    let service_context = Arc::new(ServiceContext { service_a, service_b });
    Arc::new(AppContext { state_context, service_context })
}
```

## 設計の契約

- 状態はStateContext経由でのみアクセス
- サービス間の依存はServiceContext経由で解決
- UI層はAppContextのみに依存
