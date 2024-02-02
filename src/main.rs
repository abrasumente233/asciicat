use color_eyre::eyre::eyre;
use pretty_hex::PrettyHex;
use serde::Deserialize;

#[tokio::main]
async fn main() {
    let image_url = get_cat_url().await.unwrap();
    println!("The cat image URL is: {}", image_url);
}

async fn get_cat_url() -> color_eyre::Result<String> {
    #[derive(Deserialize, Debug)]
    struct CatImage {
        url: String,
    }

    reqwest::get("https://api.thecatapi.com/v1/images/search")
        .await?
        .error_for_status()?
        .json::<Vec<CatImage>>()
        .await?
        .into_iter()
        .next()
        .ok_or(eyre!("The cat API did not return any images."))
        .map(|cat_image| cat_image.url)
}
