use axum::{
    extract::State,
    http::{header, HeaderMap, StatusCode},
    response::IntoResponse,
    routing::get,
    Router,
};
use color_eyre::{eyre::eyre, Result};
use locat::Locat;
use opentelemetry::{
    global,
    trace::{get_active_span, FutureExt, Span, Status, TraceContextExt, Tracer},
    Context, KeyValue,
};
use serde::Deserialize;
use std::{net::IpAddr, str::FromStr, sync::Arc};
use tracing::{info, warn, Level};
use tracing_subscriber::{filter::Targets, layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Clone)]
struct ServerState {
    client: reqwest::Client,
    locat: Arc<Locat>,
}

#[tokio::main]
async fn main() {
    let _guard = sentry::init((
        std::env::var("SENTRY_DSN").expect("$SENTRY_DSN should be set"),
        sentry::ClientOptions {
            release: sentry::release_name!(),
            ..Default::default()
        },
    ));

    let (_honeyguard, _tracer) = opentelemetry_honeycomb::new_pipeline(
        std::env::var("HONEYCOMB_API_KEY").expect("$HONEYCOMB_API_KEY should be set"),
        "asciicat".into(),
    )
    .install()
    .unwrap();

    let filter = Targets::from_str(std::env::var("RUST_LOG").as_deref().unwrap_or("info"))
        .expect("Invalid RUST_LOG value");
    tracing_subscriber::fmt()
        .with_max_level(Level::TRACE)
        .json()
        .finish()
        .with(filter)
        .init();

    let quit_sig = async {
        _ = tokio::signal::ctrl_c().await;
        warn!("Initiating graceful shutdown!");
    };

    let country_db_path =
        std::env::var("GEOLITE2_COUNTRY_DB").expect("$GEOLITE2_COUNTRY_DB should be set");
    let analytics_db_path = std::env::var("ANALYTICS_DB").expect("$ANALYTICS_DB should be set");
    let state = ServerState {
        client: reqwest::Client::default(),
        locat: Arc::new(Locat::new(&country_db_path, &analytics_db_path).unwrap()),
    };

    let app = Router::new()
        .route("/", get(root_get))
        .route("/analytics", get(analytics_get))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    info!("Listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app)
        .with_graceful_shutdown(quit_sig)
        .await
        .unwrap();
}

fn get_client_addr(headers: &HeaderMap) -> Option<IpAddr> {
    let header = headers.get("fly-client-ip")?;
    let header = header.to_str().ok()?;
    let addr = header.parse().ok()?;
    Some(addr)
}

async fn analytics_get(State(state): State<ServerState>) -> impl IntoResponse {
    let mut response = String::new();
    for (country, count) in state.locat.get_analytics().await.unwrap() {
        response.push_str(&format!("{}: {}\n", country, count));
    }
    response.into_response()
}

async fn root_get(headers: HeaderMap, State(state): State<ServerState>) -> impl IntoResponse {
    let tracer = global::tracer("");
    let mut span = tracer.start("root_get");
    span.set_attribute(KeyValue::new(
        "user-agent",
        headers
            .get(header::USER_AGENT)
            .map(|h| h.to_str().unwrap_or_default().to_owned())
            .unwrap_or_default(),
    ));

    // get geo-location from IP
    if let Some(addr) = get_client_addr(&headers) {
        match state.locat.ip_to_iso_code(addr).await {
            Some(country) => {
                info!("Got request from {}", country);
                span.set_attribute(KeyValue::new("country", country.to_owned()));
            }
            None => warn!("Could not find country for IP address"),
        }
    }

    root_get_inner(state)
        .with_context(Context::current_with_span(span))
        .await
}

async fn root_get_inner(state: ServerState) -> impl IntoResponse {
    let tracer = global::tracer("");

    match get_ascii_cat(&state.client)
        .with_context(Context::current_with_span(tracer.start("get_ascii_cat")))
        .await
    {
        Ok(art) => (
            StatusCode::OK,
            [(header::CONTENT_TYPE, "text/html; charset=utf-8")],
            art,
        )
            .into_response(),
        Err(e) => {
            get_active_span(|span| {
                span.set_status(Status::Error {
                    description: format!("{e}").into(),
                })
            });
            (StatusCode::INTERNAL_SERVER_ERROR, format!("Error: {}", e)).into_response()
        }
    }
}

#[derive(Deserialize, Debug)]
struct CatImage {
    url: String,
}

async fn get_ascii_cat(client: &reqwest::Client) -> Result<String> {
    let tracer = global::tracer("");

    let url = get_cat_url(client)
        .with_context(Context::current_with_span(tracer.start("get_cat_url")))
        .await?;

    let image_bytes = download_url(client, &url)
        .with_context(Context::current_with_span(tracer.start("download_url")))
        .await?;

    let image = tracer.in_span(
        "image::load_from_memory",
        |cx| -> std::result::Result<image::DynamicImage, _> {
            let img = image::load_from_memory(&image_bytes)?;
            cx.span()
                .set_attribute(KeyValue::new("width", img.width() as i64));
            cx.span()
                .set_attribute(KeyValue::new("height", img.height() as i64));
            Ok::<_, color_eyre::Report>(img)
        },
    )?;

    let ascii_art = tracer.in_span("artem::convert", |_cx| {
        artem::convert(
            image,
            &artem::ConfigBuilder::new()
                .target(artem::config::TargetType::HtmlFile(true, true))
                .build(),
        )
    });

    Ok(ascii_art)
}

async fn get_cat_url(client: &reqwest::Client) -> Result<String> {
    client
        .get("https://api.thecatapi.com/v1/images/search")
        .send()
        .await?
        .error_for_status()?
        .json::<Vec<CatImage>>()
        .await?
        .into_iter()
        .next()
        .ok_or(eyre!("The cat API did not return any images."))
        .map(|cat_image| cat_image.url)
}

async fn download_url(client: &reqwest::Client, url: &str) -> Result<Vec<u8>> {
    Ok(client
        .get(url)
        .send()
        .await?
        .error_for_status()?
        .bytes()
        .await?
        .to_vec())
}
