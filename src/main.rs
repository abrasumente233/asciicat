use color_eyre::eyre::eyre;
use pretty_hex::PrettyHex;
use serde::Deserialize;

#[tokio::main]
async fn main() {
    let image = get_cat_bytes().await.unwrap();
    dbg!(image.len());
    println!("{:?}", &image[..200].hex_dump());
    //println!("The cat image URL is: {}", image_url);
}

async fn get_cat_bytes() -> color_eyre::Result<Vec<u8>> {
    #[derive(Deserialize, Debug)]
    struct CatImage {
        url: String,
    }

    let url = reqwest::get("https://api.thecatapi.com/v1/images/search")
        .await?
        .error_for_status()?
        .json::<Vec<CatImage>>()
        .await?
        .into_iter()
        .next()
        .ok_or(eyre!("The cat API did not return any images."))
        .map(|cat_image| cat_image.url)?;

    Ok(reqwest::get(&url)
        .await?
        .error_for_status()?
        .bytes()
        .await?
        .to_vec())
}
