# OpenTelemetry Lambda Rust Project

:warning: この README は Copilot によって自動生成され、人の手で軽く修正しました

Rust製Lambda関数でOpenTelemetryを使用したサンプルプロジェクトです。AWS Lambda上でOpenTelemetryのトレーシングとログ機能を実装し、CloudWatch と連携してモニタリング機能を提供します。

## 📚 元記事

正確性の面では下記の記事を参考にしてください



## 🏗️ アーキテクチャ

このプロジェクトは以下の構成で動作します：

- **API Lambda**: メインのRust製Lambda関数
- **API Lambda Remote**: リモート呼び出し用のLambda関数
- **OpenTelemetry Collector**: トレースとメトリクスの収集
- **Pulumi**: インフラストラクチャ管理

## 📁 プロジェクト構造

```
.
├── api/                    # Rust Lambda API
│   ├── src/
│   │   ├── main.rs        # エントリーポイント
│   │   ├── hello.rs       # Hello API エンドポイント
│   │   └── otel.rs        # OpenTelemetry設定
│   ├── aws/
│   │   ├── lambda.ts      # Lambda関数定義（メイン）
│   │   ├── lambda-remote.ts # Lambda関数定義（リモート）
│   │   └── collector-config.yaml # OTel Collector設定
│   ├── Cargo.toml
│   └── build.rs          # ビルド時設定
├── monitoring/
│   └── aws/
│       └── transaction-search.ts # Transaction Search 設定
├── index.ts              # Pulumiエントリーポイント
├── awspp                 # AWS/Pulumi切り替えスクリプト
└── README.md
```

## 🚀 API エンドポイント

### Hello API

- `GET /api/v0/hello` - シンプルな挨拶を返す
- `GET /api/v0/hello/remote` - リモートLambdaを呼び出す
- `POST /api/v0/greet` - カスタム挨拶メッセージを作成

### API仕様
- **Base Path**: `/api/v0`
- **ドキュメント**: `/api/docs` (Scalar UI)
- **レスポンス形式**: JSON

## 🛠️ 技術スタック

### Backend (Rust)
- **axum**: 高性能なWebフレームワーク
- **tokio**: 非同期ランタイム
- **utoipa**: OpenAPI仕様生成
- **opentelemetry**: 分散トレーシング
- **tracing**: 構造化ログ
- **lambda_http**: AWS Lambda統合

### Infrastructure (TypeScript)
- **Pulumi**: Infrastructure as Code
- **AWS Lambda**: サーバーレス実行環境
- **AWS X-Ray**: 分散トレーシング
- **AWS CloudWatch**: ログ管理

## 📋 前提条件

- Rust
- Node.js
- AWS CLI
- Pulumi CLI
- `cargo-zigbuild` (ARM64対応)

```bash
# Rust toolchain
rustup target add aarch64-unknown-linux-musl

# cargo-zigbuild
cargo install cargo-zigbuild

# Node.js dependencies
pnpm i
```

## 🔧 セットアップ

### 1. AWSプロファイル設定
```bash
# AWS SSOログイン
aws configure sso

# プロファイル確認
aws configure list-profiles
```

### 2. Pulumi設定

```bash
# Pulumiスタック作成
pulumi stack init <stack name>

# AWS設定
pulumi config set aws:region ap-northeast-1
pulumi config set aws:profile your-profile-name
pulumi config set aws-native:region ap-northeast-1
pulumi config set aws-native:profile your-profile-name
```

### 3. awsppスクリプト使用

```bash
# 対話的にスタック/プロファイル選択
source ./awspp
```

## 🏃‍♂️ 実行方法

### ローカル開発
```bash
cd api
cargo run
```

### デプロイ

```bash
# インフラストラクチャデプロイ
pulumi up

# lambda 向けビルド
cd api
cargo zigbuild --release --target aarch64-unknown-linux-musl --features lambda
```

## 📊 OpenTelemetry設定

### トレーシング
- **プロバイダー**: [`init_tracer_provider`](api/src/otel.rs)
- **エクスポーター**: OTLP over gRPC
- **サンプリング**: Always On
- **プロパゲーション**: TraceContext

### ログ
- **プロバイダー**: [`init_logger_provider`](api/src/otel.rs)
- **フォーマット**: JSON
- **出力**: CloudWatch Logs

### リソース属性
- **Service**: サービス名、バージョン、ネームスペース
- **Cloud**: AWS Lambda、リージョン情報
- **VCS**: Git情報（ブランチ、コミット）
- **Deployment**: 環境名（Pulumiスタック）

## 🔍 モニタリング

### CloudWatch 統合

- [`lambdaXrayMonitoringPolicy`](api/aws/lambda.ts)でX-Rayアクセス権限設定
- [`transaction-search.ts`](monitoring/aws/transaction-search.ts)でトランザクション検索設定

### ログ

- CloudWatch Logsに構造化ログを出力
- アプリケーションログレベル: INFO
- システムログレベル: WARN

## 📝 設定

### 環境変数

- `RUST_LOG`: ログレベル
- `OPENTELEMETRY_COLLECTOR_CONFIG_URI`: OTel Collector設定ファイルパス
- `TZ`: タイムゾーン

### ビルド時変数

- `PULUMI_STACK`: デプロイメント環境
- `PROJECT_NAME`: プロジェクト名
- `API_LAMBDA_ARN`: Lambda ARN
- `REMOTE_ENDPOINT`: リモートエンドポイントURL

## 🧪 テスト

```bash
# 単体テスト
cd api
cargo test

# Scalar による手動統合テスト
curl https://your-lambda-url.lambda-url.ap-northeast-1.on.aws/api/docs
```

## 📄 ライセンス

Apache License 2.0 - 詳細は [LICENSE](LICENSE) ファイルを参照してください。
