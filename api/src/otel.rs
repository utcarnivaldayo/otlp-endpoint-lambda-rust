use opentelemetry_sdk::Resource;
use opentelemetry_sdk::resource::ResourceDetector;

pub struct LambdaResourceDetector;

impl ResourceDetector for LambdaResourceDetector {
    fn detect(&self) -> Resource {
        let attributes = [
            lambda_resource_attributes(),
            deployment_environment_resource_attributes(),
            service_resource_attributes(),
            telemetry_sdk_resource_attributes(),
            vcs_resource_attributes(),
        ]
        .concat();

        let resource = opentelemetry_sdk::Resource::builder_empty()
            .with_schema_url(attributes, opentelemetry_semantic_conventions::SCHEMA_URL)
            .build();
        resource
    }
}

fn lambda_resource_attributes() -> Vec<opentelemetry::KeyValue> {
    // lambda
    let lambda_name: String = std::env::var("AWS_LAMBDA_FUNCTION_NAME").unwrap_or_default();
    if lambda_name.is_empty() {
        return vec![];
    }
    let aws_region: String = std::env::var("AWS_REGION").unwrap();
    let function_version: String = std::env::var("AWS_LAMBDA_FUNCTION_VERSION").unwrap();
    let function_memory_limit: i64 = std::env::var("AWS_LAMBDA_FUNCTION_MEMORY_SIZE")
        .map(|s: String| s.parse::<i64>().unwrap_or_default() * 1024 * 1024)
        .unwrap_or_default();
    let instance: String = std::env::var("AWS_LAMBDA_LOG_STREAM_NAME").unwrap_or_default();
    let log_group_name: String = std::env::var("AWS_LAMBDA_LOG_GROUP_NAME").unwrap_or_default();
    let lambda_arn : String = std::env::var("API_LAMBDA_ARN").unwrap_or_default();

    use opentelemetry::{Array, KeyValue, StringValue, Value};
    let attributes: Vec<KeyValue> = vec![
        KeyValue::new(
            opentelemetry_semantic_conventions::resource::CLOUD_PROVIDER,
            "aws",
        ),
        KeyValue::new(
            opentelemetry_semantic_conventions::resource::CLOUD_PLATFORM,
            "aws_lambda",
        ),
        KeyValue::new(
            opentelemetry_semantic_conventions::resource::CLOUD_REGION,
            aws_region,
        ),
        KeyValue::new(
            opentelemetry_semantic_conventions::resource::CLOUD_RESOURCE_ID,
            lambda_arn,
        ),
        KeyValue::new(
            opentelemetry_semantic_conventions::resource::FAAS_INSTANCE,
            instance,
        ),
        KeyValue::new(
            opentelemetry_semantic_conventions::resource::FAAS_NAME,
            lambda_name,
        ),
        KeyValue::new(
            opentelemetry_semantic_conventions::resource::FAAS_VERSION,
            function_version,
        ),
        KeyValue::new(
            opentelemetry_semantic_conventions::resource::FAAS_MAX_MEMORY,
            function_memory_limit,
        ),
        KeyValue::new(
            opentelemetry_semantic_conventions::resource::AWS_LOG_GROUP_NAMES,
            Value::Array(Array::from(vec![StringValue::from(log_group_name)])),
        ),
    ];
    attributes
}

fn service_resource_attributes() -> Vec<opentelemetry::KeyValue> {
    use uuid::Uuid;
    // service
    let service_name: &str = env!("CARGO_PKG_NAME");
    let service_version: &str = env!("CARGO_PKG_VERSION");
    let service_namespace: &str = env!("PROJECT_NAME");
    let service_instance_id: Uuid = Uuid::new_v4();

    use opentelemetry::KeyValue;
    let attributes: Vec<KeyValue> = vec![
        KeyValue::new(
            opentelemetry_semantic_conventions::resource::SERVICE_NAME,
            service_name,
        ),
        KeyValue::new(
            opentelemetry_semantic_conventions::resource::SERVICE_VERSION,
            service_version,
        ),
        KeyValue::new(
            opentelemetry_semantic_conventions::resource::SERVICE_NAMESPACE,
            service_namespace,
        ),
        KeyValue::new(
            opentelemetry_semantic_conventions::resource::SERVICE_INSTANCE_ID,
            service_instance_id.to_string(),
        ),
    ];
    attributes
}

fn telemetry_sdk_resource_attributes() -> Vec<opentelemetry::KeyValue> {
    let telemetry_sdk_name: &str = env!("TELEMETRY_SDK_VERSION");
    use opentelemetry::KeyValue;
    let attributes: Vec<KeyValue> = vec![
        KeyValue::new(
            opentelemetry_semantic_conventions::resource::TELEMETRY_SDK_NAME,
            "opentelemetry",
        ),
        KeyValue::new(
            opentelemetry_semantic_conventions::resource::TELEMETRY_SDK_LANGUAGE,
            "rust",
        ),
        KeyValue::new(
            opentelemetry_semantic_conventions::resource::TELEMETRY_SDK_VERSION,
            telemetry_sdk_name,
        ),
    ];
    attributes
}

fn deployment_environment_resource_attributes() -> Vec<opentelemetry::KeyValue> {
    // deployment environment
    let deployment_environment_name: String = env!("PULUMI_STACK").to_string();

    use opentelemetry::KeyValue;
    let attributes: Vec<KeyValue> = vec![KeyValue::new(
        opentelemetry_semantic_conventions::resource::DEPLOYMENT_ENVIRONMENT_NAME,
        deployment_environment_name,
    )];
    attributes
}


fn vcs_resource_attributes() -> Vec<opentelemetry::KeyValue> {
    // vcs
    let vcs_ref_head_name: &str = env!("VCS_REF_HEAD_NAME");
    let vcs_ref_head_revision: &str = env!("VCS_REF_HEAD_REVISION");
    let vcs_ref_head_type: &str = "branch";
    let vcs_repository_name: &str = env!("VCS_REPOSITORY_NAME");
    let vcs_repository_url_full: &str = env!("VCS_REPOSITORY_URL_FULL");

    use opentelemetry::KeyValue;
    let attributes: Vec<KeyValue> = vec![
        KeyValue::new(
            opentelemetry_semantic_conventions::resource::VCS_REF_HEAD_NAME,
            vcs_ref_head_name,
        ),
        KeyValue::new(
            opentelemetry_semantic_conventions::resource::VCS_REF_HEAD_REVISION,
            vcs_ref_head_revision,
        ),
        KeyValue::new(
            opentelemetry_semantic_conventions::resource::VCS_REF_TYPE,
            vcs_ref_head_type,
        ),
        KeyValue::new(
            opentelemetry_semantic_conventions::resource::VCS_REPOSITORY_NAME,
            vcs_repository_name,
        ),
        KeyValue::new(
            opentelemetry_semantic_conventions::resource::VCS_REPOSITORY_URL_FULL,
            vcs_repository_url_full,
        ),
    ];
    attributes
}

pub fn make_span_with_impl(req: &axum::extract::Request<axum::body::Body>) -> tracing::Span {
    use opentelemetry_http::HeaderExtractor;
    use tracing_opentelemetry::OpenTelemetrySpanExt;
    let empty = tracing::field::Empty;
    let method: &str = req.method().as_str();
    let route: &str = req
        .extensions()
        .get::<axum::extract::MatchedPath>()
        .map_or_else(|| "", |p| p.as_str());
    let span_name: String = format!("{} {}", method, route).trim().to_string();
    // TODO: server.address and server.port attributes
    let span: tracing::Span = tracing::info_span!(
        "",
        otel.name = span_name,
        { opentelemetry_semantic_conventions::trace::URL_PATH } = empty,
        { opentelemetry_semantic_conventions::trace::URL_SCHEME } = empty,
        { opentelemetry_semantic_conventions::trace::HTTP_ROUTE } = empty,
        { opentelemetry_semantic_conventions::trace::HTTP_REQUEST_METHOD } = empty,
        { opentelemetry_semantic_conventions::trace::HTTP_REQUEST_HEADER } = empty,
        { opentelemetry_semantic_conventions::trace::NETWORK_PROTOCOL_VERSION } = empty,
        { opentelemetry_semantic_conventions::trace::CLIENT_ADDRESS } = empty,
        { opentelemetry_semantic_conventions::trace::USER_AGENT_ORIGINAL } = empty,
        { opentelemetry_semantic_conventions::trace::HTTP_RESPONSE_STATUS_CODE } = empty,
        { opentelemetry_semantic_conventions::attribute::ERROR_TYPE } = empty,
    );
    span.set_parent(opentelemetry::global::get_text_map_propagator(
        |propagator| propagator.extract(&HeaderExtractor(req.headers())),
    ))
    .expect("Failed to set parent span from request headers");
    span
}

pub fn on_request_impl(req: &axum::extract::Request<axum::body::Body>, span: &tracing::Span) {
    span.record(
        opentelemetry_semantic_conventions::trace::URL_PATH,
        req.uri().path(),
    );
    span.record(
        opentelemetry_semantic_conventions::trace::URL_SCHEME,
        req.uri().scheme_str().unwrap_or("http"),
    );
    span.record(
        opentelemetry_semantic_conventions::trace::HTTP_ROUTE,
        req.extensions()
            .get::<axum::extract::MatchedPath>()
            .map_or_else(|| "", |p| p.as_str()),
    );
    span.record(
        opentelemetry_semantic_conventions::trace::HTTP_REQUEST_METHOD,
        req.method().as_str(),
    );
    span.record(
        opentelemetry_semantic_conventions::trace::HTTP_REQUEST_HEADER,
        tracing::field::debug(req.headers()),
    );
    span.record(
        opentelemetry_semantic_conventions::trace::NETWORK_PROTOCOL_VERSION,
        tracing::field::debug(req.version()),
    );
    // TODO: get client IP address
    span.record(
        opentelemetry_semantic_conventions::trace::CLIENT_ADDRESS,
        req.headers()
            .get(axum::http::header::HOST)
            .map(|v| v.to_str().unwrap_or_default()),
    );
    span.record(
        opentelemetry_semantic_conventions::trace::USER_AGENT_ORIGINAL,
        req.headers()
            .get(axum::http::header::USER_AGENT)
            .map(|v| v.to_str().unwrap_or_default()),
    );
}

pub fn on_response_impl(
    res: &axum::response::Response,
    _: std::time::Duration,
    span: &tracing::Span,
) {
    let status = res.status();
    span.record(
        opentelemetry_semantic_conventions::trace::HTTP_RESPONSE_STATUS_CODE,
        tracing::field::display(status),
    );
    if !status.is_success() {
        span.record(
            opentelemetry_semantic_conventions::trace::ERROR_TYPE,
            tracing::field::display(status),
        );
    }
}

pub fn init_resource() -> opentelemetry_sdk::Resource {
    let detector: LambdaResourceDetector = LambdaResourceDetector;
    let resource: opentelemetry_sdk::Resource = detector.detect();
    resource
}

pub fn init_tracer_provider(
    resource: opentelemetry_sdk::Resource,
) -> opentelemetry_sdk::trace::SdkTracerProvider {
    use opentelemetry_otlp::SpanExporter;
    use opentelemetry_otlp::WithExportConfig;
    use std::time::Duration;
    let span_exporter: SpanExporter = SpanExporter::builder()
        .with_tonic()
        .with_endpoint("http://localhost:4317")
        .with_protocol(opentelemetry_otlp::Protocol::Grpc)
        //.with_timeout(opentelemetry_otlp::OTEL_EXPORTER_OTLP_TIMEOUT_DEFAULT)
        .with_timeout(Duration::new(3, 0))
        .build()
        .expect("Failed to create OTLP exporter");

    // let span_exporter = opentelemetry_stdout::SpanExporter::default();

    // otel tracer
    opentelemetry_sdk::trace::SdkTracerProvider::builder()
        // .with_simple_exporter(span_exporter)
        .with_sampler(opentelemetry_sdk::trace::Sampler::AlwaysOn)
        .with_id_generator(opentelemetry_sdk::trace::RandomIdGenerator::default())
        .with_resource(resource)
        .with_batch_exporter(span_exporter)
        .build()
}

pub fn init_scope() -> opentelemetry::InstrumentationScope {
    opentelemetry::InstrumentationScope::builder(env!("CARGO_PKG_NAME"))
        .with_version(env!("CARGO_PKG_VERSION"))
        .with_schema_url(opentelemetry_semantic_conventions::SCHEMA_URL)
        // .with_attributes(attributes)
        .build()
}

pub fn init_tracer(
    tracer_provider: &opentelemetry_sdk::trace::SdkTracerProvider,
) -> opentelemetry_sdk::trace::SdkTracer {
    use opentelemetry::InstrumentationScope;
    use opentelemetry::trace::TracerProvider;
    let scope: InstrumentationScope = init_scope();
    tracer_provider.tracer_with_scope(scope)
}

pub fn init_logger_provider(
    resource: opentelemetry_sdk::Resource,
) -> opentelemetry_sdk::logs::SdkLoggerProvider {
    use opentelemetry_otlp::LogExporter;
    use opentelemetry_otlp::WithExportConfig;
    use std::time::Duration;
    let log_exporter: LogExporter = opentelemetry_otlp::LogExporter::builder()
        .with_tonic()
        .with_endpoint("http://localhost:4317")
        .with_protocol(opentelemetry_otlp::Protocol::Grpc)
        .with_timeout(Duration::new(3, 0))
        //.with_timeout(opentelemetry_otlp::OTEL_EXPORTER_OTLP_TIMEOUT_DEFAULT)
        .build()
        .expect("Failed to create OTLP log exporter");

    // let log_exporter = opentelemetry_stdout::LogExporter::default();

    opentelemetry_sdk::logs::SdkLoggerProvider::builder()
        .with_resource(resource)
        .with_batch_exporter(log_exporter)
        .build()
}

pub fn init_tracing_subscriber(
    tracer_provider: &opentelemetry_sdk::trace::SdkTracerProvider,
    logger_provider: &opentelemetry_sdk::logs::SdkLoggerProvider,
) {
    use tracing_subscriber::layer::SubscriberExt;
    let tracer = init_tracer(tracer_provider);
    let tracer_layer = tracing_opentelemetry::layer().with_tracer(tracer);

    use opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge;
    let logger_layer = OpenTelemetryTracingBridge::new(logger_provider);

    let subscriber = tracing_subscriber::registry()
        .with(
            tracing_subscriber::filter::EnvFilter::builder()
                .with_default_directive(tracing_subscriber::filter::LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .with(
            tracing_subscriber::fmt::layer()
                .json()
                .with_level(true)
                .with_span_events(tracing_subscriber::fmt::format::FmtSpan::ACTIVE)
                .with_timer(
                    tracing_subscriber::fmt::time::OffsetTime::local_rfc_3339()
                        .expect("Failed to create tracing subscriber timer"),
                )
                .with_file(true)
                .with_line_number(true)
                .with_thread_ids(true)
                .with_current_span(true)
                .with_ansi(false)
                .with_target(true)
                .with_writer(std::io::stdout),
        )
        .with(tracer_layer)
        .with(logger_layer);
    tracing::subscriber::set_global_default(subscriber).expect("Failed to set tracing subscriber");
}
