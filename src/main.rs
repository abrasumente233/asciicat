use axum::{
    http::{header, StatusCode},
    response::IntoResponse,
    routing::get,
    Router,
};
use color_eyre::eyre::eyre;
use serde::Deserialize;
use std::str::FromStr;
use tracing::{info, Level};
use tracing_subscriber::{filter::Targets, layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    let filter = Targets::from_str(std::env::var("RUST_LOG").as_deref().unwrap_or("info"))
        .expect("Invalid RUST_LOG value");
    tracing_subscriber::fmt()
        .with_max_level(Level::TRACE)
        .json()
        .finish()
        .with(filter)
        .init();

    let app = Router::new().route("/", get(root_get));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    info!("Listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
    // let cat = get_ascii_cat().await.unwrap();
    // println!("{cat}");
}

async fn root_get() -> impl IntoResponse {
    match get_ascii_cat().await {
        Ok(art) => (
            StatusCode::OK,
            [(header::CONTENT_TYPE, "text/html; charset=utf-8")],
            art,
        )
            .into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Error: {}", e)).into_response(),
    }
}

async fn get_ascii_cat() -> color_eyre::Result<String> {
    #[derive(Deserialize, Debug)]
    struct CatImage {
        url: String,
    }

    let client = reqwest::Client::new();

    let url = client
        .get("https://api.thecatapi.com/v1/images/search")
        .send()
        .await?
        .error_for_status()?
        .json::<Vec<CatImage>>()
        .await?
        .into_iter()
        .next()
        .ok_or(eyre!("The cat API did not return any images."))
        .map(|cat_image| cat_image.url)?;

    let image_bytes = client
        .get(&url)
        .send()
        .await?
        .error_for_status()?
        .bytes()
        .await?;

    let image = image::load_from_memory(&image_bytes)?;
    let ascii_art = artem::convert(
        image,
        &artem::ConfigBuilder::new()
            .target(artem::config::TargetType::HtmlFile(true, true))
            .build(),
    );

    Ok(ascii_art)
}
