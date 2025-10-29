mod hello;
mod otel;

use utoipa::OpenApi;
#[derive(OpenApi)]
#[openapi(info(
    title = env!("PROJECT_NAME"),
    version = env!("CARGO_PKG_VERSION"),
    description = "sample api description",
))]
struct ApiDocs;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    opentelemetry::global::set_text_map_propagator(
        opentelemetry_sdk::propagation::TraceContextPropagator::new(),
    );

    let resouce: opentelemetry_sdk::Resource = otel::init_resource();
    let tracer_provider: opentelemetry_sdk::trace::SdkTracerProvider =
        otel::init_tracer_provider(resouce.clone());
    let logger_provider: opentelemetry_sdk::logs::SdkLoggerProvider = otel::init_logger_provider(resouce);
    otel::init_tracing_subscriber(&tracer_provider, &logger_provider);

    //let state: StateContainer = StateContainer::new(tracer_provider, logger_provider);

    const API_BASE_PATH: &str = env!("API_BASE_PATH");
    // クレートバージョンが 0.1.2 ならば、メジャーバージョンは 0
    let api_major_version: usize = env!("CARGO_PKG_VERSION")
        .split('.')
        .next()
        .unwrap()
        .parse()
        .unwrap();

    use utoipa_axum::router::OpenApiRouter;
    let api_base_path = format!("{}/v{}",API_BASE_PATH, api_major_version);
    let (api_router, api_docs) = OpenApiRouter::with_openapi(ApiDocs::openapi())
        .nest(api_base_path.as_str(), hello::create_hello_router())
        .split_for_parts();

    use utoipa_scalar::{Scalar, Servable};
    let app_router = axum::Router::new()
        .merge(api_router)
        .merge(Scalar::with_url(format!("{}/docs", API_BASE_PATH), api_docs))
        .layer(tower_http::cors::CorsLayer::permissive())
        .layer(
            tower_http::trace::TraceLayer::new_for_http()
                .make_span_with(otel::make_span_with_impl)
                .on_request(otel::on_request_impl)
                .on_response(otel::on_response_impl)
        );

    #[cfg(not(feature = "lambda"))]
    {
        use tokio::net::TcpListener;
        let listener: TcpListener = TcpListener::bind("localhost:3030").await.unwrap();
        axum::serve(listener, app_router).await.unwrap();
    }

    #[cfg(feature = "lambda")]
    {
        // lambda_http::run(app_router).await.unwrap();
        use lambda_http::lambda_runtime::layers::{
            OpenTelemetryFaasTrigger, OpenTelemetryLayer as OTelLayer,
        };
        let runtime =
            lambda_http::lambda_runtime::Runtime::new(lambda_http::Adapter::from(app_router))
                .layer(
                    OTelLayer::new(|| {
                        tracing::info!("OpenTelemetry provider flush on lambda shutdown");
                        tracer_provider.force_flush().unwrap();
                        logger_provider.force_flush().unwrap();
                    })
                    .with_trigger(OpenTelemetryFaasTrigger::Http),
                );
        runtime.run().await.unwrap();
    }

    Ok(())
}
