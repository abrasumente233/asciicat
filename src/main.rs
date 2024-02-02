use color_eyre::eyre::eyre;
use serde::Deserialize;

#[tokio::main]
async fn main() {
    let cat = get_ascii_cat().await.unwrap();
    println!("{cat}");
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
    let ascii_art = artem::convert(image, &artem::ConfigBuilder::new().build());

    Ok(ascii_art)
}
