#[tokio::main]
async fn main() {
    let resp = reqwest::get("https://api.thecatapi.com/v1/images/search")
        .await
        .unwrap();
    println!("Status: {}", resp.status());
    let body = resp.text().await.unwrap();
    println!("Body: {}", body);
}
