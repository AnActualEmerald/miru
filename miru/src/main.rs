use std::fs;

use lib_mal::MALClient;
use lib_mal::model::fields::AnimeField;
use lib_mal::model::options::{RankingType, Status, StatusUpdate, Params};
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
    let mut update = StatusUpdate::new();
    update.status(Status::Dropped);
    update.score(10);
    update.priority(0);
    update.rewatch_value(3);
    update.comments("Pretty good show lol");
    println!("{:?}", client.update_user_anime_status(80, update).await);
}
