const HELLO_TAG: &str = "hello";

#[utoipa::path(
    get,
    path = "/hello",
    responses(
        (status = 200, body = String),
    ),
    tags = [ HELLO_TAG ]
)]
#[tracing::instrument(ret)]
pub async fn hello() -> &'static str {
    tracing::info!("Saying info hello");
    tracing::warn!("Saying warn hello");
    // tracing::error!("Saying error hello");
    "Hello, World!"
}

#[utoipa::path(
    get,
    path = "/hello/remote",
    responses(
        (status = 200, body = String),
    ),
    tags = [ HELLO_TAG ]
)]
async fn hello_remote() -> String {
    use opentelemetry_http::HeaderInjector;
    let span = tracing::info_span!("hello remote");
    let remote_endpoint: String = env!("REMOTE_ENDPOINT").to_string();
    let url = format!("{}/api/v0/hello", remote_endpoint);
    let mut headers = reqwest::header::HeaderMap::new();
    opentelemetry::global::get_text_map_propagator(|propagator| {
        propagator.inject_context(&span.context(), &mut HeaderInjector(&mut headers))
    });
    let client = reqwest::Client::new();
    let response = client.get(&url)
        .headers(headers)
        .send()
        .await;
    match response {
        Ok(res) => {
            let body = res.text().await.unwrap();
            tracing::info!("Received response from remote: {}", body);
            body
        }
        Err(err) => {
            tracing::error!("Failed to call remote endpoint: {}", err);
            format!("Error calling remote endpoint: {}", err)
        }
    }
}


use serde::{Deserialize, Serialize};
use tracing_opentelemetry::OpenTelemetrySpanExt;
use utoipa::{
    ToSchema,
    openapi::{Object, ObjectBuilder},
};
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
struct GreetContent {
    #[schema(schema_with = Self::person_schema)]
    pub person: String,
    #[schema(schema_with = Self::message_schema)]
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
enum GreetContentError {
    InvalidPerson,
    InvalidMessage,
}

impl GreetContent {
    const DESCRIPTION: &'static str = "挨拶の内容";

    const PERSON_TITLE: &'static str = "名前";
    const PERSON_DESCRIPTION: &'static str = "挨拶する人の名前";
    const PERSON_EXAMPLE: &'static str = "山田太郎";
    const PERSON_MIN_LENGTH: usize = 1;
    const PERSON_MAX_LENGTH: usize = 20;

    const MESSAGE_TITLE: &'static str = "メッセージ";
    const MESSAGE_DESCRIPTION: &'static str = "やぁ の挨拶に続く簡単なメッセージ";
    const MESSAGE_EXAMPLE: &'static str = "お元気ですか？";
    const MESSAGE_MIN_LENGTH: usize = 3;
    const MESSAGE_MAX_LENGTH: usize = 32;

    #[tracing::instrument(ret)]
    fn try_new(person: String, message: String) -> Result<Self, GreetContentError> {
        if person.len() < Self::PERSON_MIN_LENGTH || person.len() > Self::PERSON_MAX_LENGTH {
            return Err(GreetContentError::InvalidPerson);
        }
        if message.len() < Self::MESSAGE_MIN_LENGTH || message.len() > Self::MESSAGE_MAX_LENGTH {
            return Err(GreetContentError::InvalidMessage);
        }
        Ok(Self { person, message })
    }

    fn person_schema() -> Object {
        ObjectBuilder::new()
            .title(Some(GreetContent::PERSON_TITLE))
            .description(Some(GreetContent::PERSON_DESCRIPTION))
            .examples(Some(GreetContent::PERSON_EXAMPLE))
            .min_length(Some(GreetContent::PERSON_MIN_LENGTH))
            .max_length(Some(GreetContent::PERSON_MAX_LENGTH))
            .build()
    }

    fn message_schema() -> Object {
        ObjectBuilder::new()
            .title(Some(GreetContent::MESSAGE_TITLE))
            .description(Some(GreetContent::MESSAGE_DESCRIPTION))
            .examples(Some(GreetContent::MESSAGE_EXAMPLE))
            .min_length(Some(GreetContent::MESSAGE_MIN_LENGTH))
            .max_length(Some(GreetContent::MESSAGE_MAX_LENGTH))
            .build()
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
struct GreetResponse {
    greeting: String,
}

impl GreetResponse {
    #[tracing::instrument(ret)]
    fn create_greeting(greet_content: &GreetContent) -> Self {
        use std::{thread, time};
        const HEAVY_LOGIC: u64 = 1000; // [ms]
        thread::sleep(time::Duration::from_millis(HEAVY_LOGIC));
        Self {
            greeting: format!("やぁ {}, {}", greet_content.person, greet_content.message),
        }
    }
}

use axum::{Json, http::StatusCode};
#[utoipa::path(
    post,
    path = "/greet",
    request_body(
        description = GreetContent::DESCRIPTION,
        content_type = "application/json",
        content = GreetContent,
    ),
    responses(
        (
            status = StatusCode::OK,
            body = GreetContent
        ),
        (
            status = StatusCode::BAD_REQUEST,
            body = String
        )
    ),
    tags = [ HELLO_TAG ]
)]
#[tracing::instrument(ret)]
async fn greet(
    Json(payload): Json<GreetContent>,
) -> Result<(StatusCode, Json<GreetResponse>), (StatusCode, String)> {
    match GreetContent::try_new(payload.person, payload.message) {
        Err(GreetContentError::InvalidPerson) => {
            return Err((
                StatusCode::BAD_REQUEST,
                format!(
                    "person length must be between {} and {}",
                    GreetContent::PERSON_MIN_LENGTH,
                    GreetContent::PERSON_MAX_LENGTH
                ),
            ));
        }
        Err(GreetContentError::InvalidMessage) => {
            return Err((
                StatusCode::BAD_REQUEST,
                format!(
                    "message length must be between {} and {}",
                    GreetContent::MESSAGE_MIN_LENGTH,
                    GreetContent::MESSAGE_MAX_LENGTH
                ),
            ));
        }
        Ok(valid_payload) => Ok((
            StatusCode::OK,
            Json(GreetResponse::create_greeting(&valid_payload)),
        )),
    }
}

use utoipa_axum::router::OpenApiRouter;
pub fn create_hello_router() -> OpenApiRouter {
    let hello_router: OpenApiRouter = OpenApiRouter::new()
        .routes(utoipa_axum::routes!(hello))
        .routes(utoipa_axum::routes!(greet))
        .routes(utoipa_axum::routes!(hello_remote));
    hello_router
}
