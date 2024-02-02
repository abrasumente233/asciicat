use color_eyre::eyre::eyre;
use serde::Deserialize;

#[tokio::main]
async fn main() {
    let image_url = get_cat_url().await.unwrap();
    println!("The cat image URL is: {}", image_url);
}

async fn get_cat_url() -> color_eyre::Result<String> {
    let resp = reqwest::get("https://api.thecatapi.com/v1/images/search").await?;
    if !resp.status().is_success() {
        return Err(eyre!("The request was not successful: {}", resp.status()));
    }

    #[derive(Deserialize, Debug)]
    struct CatImage {
        url: String,
    }

    let cat_images: Vec<CatImage> = resp.json().await?;
    let Some(image) = cat_images.into_iter().next() else {
        return Err(eyre!("The cat API did not return any images."));
    };

    Ok(image.url)
}
