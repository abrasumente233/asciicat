#[derive(serde::Deserialize, Debug)]
struct CatImage {
    id: String,
    url: String,
    width: u32,
    height: u32,
}

#[tokio::main]
async fn main() {
    let resp = reqwest::get("https://api.thecatapi.com/v1/images/search")
        .await
        .unwrap();
    if !resp.status().is_success() {
        panic!("The request was not successful: {}", resp.status());
    }

    let cat_images: Vec<CatImage> = resp.json().await.unwrap();
    let cat_image = cat_images
        .first()
        .expect("the cat image API should at least return one image");
    println!("The cat image URL is: {}", cat_image.url);
}
