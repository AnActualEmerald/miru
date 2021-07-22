use lib_mal::MALClient;
use tokio;
use webbrowser;

#[tokio::main]
async fn main() {
    let mut client = MALClient::new(include_str!("secret"), true).await;
    if client.need_auth {
        let (url, challenge) = client.get_auth_parts();
        println!("Opening browser to log in...");
        if let Err(e) = webbrowser::open(&url) {
            println!(
                "Unable to open web browser: {}\nGo to this URL to log in => {}",
                e, url
            );
        }
        client.auth(&challenge).await.expect("Auth failed");
        println!("Logged in successfully!");
    }
    let list = client
        .get_anime_details(&80, None)
        .await
        .expect("Couldn't get anime list");
    println!("{}", list.alternative_titles.unwrap().synonyms[0]);
}
