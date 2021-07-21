#[macro_use]
extern crate tokio;

use lib_mal::MALClient;

#[tokio::main]
async fn main() {
    let client = MALClient::new(include_str!("secret"));
    let (url, challenge) = client.get_auth_parts();
    println!("This will look very pretty one day :) ===> {}", url);
    client.auth(&challenge).await.expect("Auth failed");
    println!("Logged in successfully");
}
