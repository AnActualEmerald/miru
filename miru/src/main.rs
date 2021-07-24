use std::fs;

use lib_mal::MALClient;
use lib_mal::model::fields::AnimeField;
use lib_mal::model::options::RankingType;
use tokio;
use webbrowser;
use directories::ProjectDirs;

#[tokio::main]
async fn main() {
    let cache_dir = if let Some(d) = ProjectDirs::from("com", "EmeraldActual", "miru"){
        if d.cache_dir().exists() {
           Some(d.cache_dir().to_path_buf())
        } else {
            fs::create_dir_all(d.cache_dir()).expect("Unable to create cache directory");
            Some(d.cache_dir().to_path_buf())
        }
    }else {
        None
    };
    let mut client = MALClient::new(include_str!("secret"), true, cache_dir).await;
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
        .get_anime_details(&80, Some(vec![AnimeField::Title, AnimeField::Studios]))
        .await
        .expect("Couldn't get anime list");
    let rank = client.get_anime_ranking(RankingType::Airing).await.expect("Unable to get anime ranking");
    println!("{:?}", list);
    println!("{:?}", rank);
}
