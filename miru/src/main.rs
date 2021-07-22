#[macro_use]
extern crate tokio;

use lib_mal::MALClient;

#[tokio::main]
async fn main() {
    let client = MALClient::new(include_str!("secret"));
    if client.need_auth {
        let (url, challenge) = client.get_auth_parts();
        println!("This will look very pretty one day :) ===> {}", url);
        client.auth(&challenge).await.expect("Auth failed");
        println!("Logged in successfully");
    }
    let list = client
        .get_anime_list()
        .await
        .expect("Couldn't get anime list");
    println!("{:?}", list);
}
