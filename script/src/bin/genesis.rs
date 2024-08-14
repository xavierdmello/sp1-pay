use sp1_pay_script::fetch_google_jwt_cert;
use tokio;

#[tokio::main]
async fn main() {
    let cert = fetch_google_jwt_cert().await.unwrap();
    println!("cert: {:?}", cert);
}