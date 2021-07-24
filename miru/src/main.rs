use tokio;

use lib_mal::MALClient;
use lib_mal::model::fields::AnimeField;
use lib_mal::model::options::RankingType;

#[tokio::main]
async fn main() {
    let mut client = MALClient::new(include_str!("secret"), true).await;
    if client.need_auth {
        let (url, challenge) = client.get_auth_parts();
        println!("This will look very pretty one day :) ===> {}", url);
        client.auth(&challenge).await.expect("Auth failed");
        println!("Logged in successfully");
    }
    let list = client
        .get_anime_details(&80, Some(vec![AnimeField::Title, AnimeField::Studios]))
        .await
        .expect("Couldn't get anime list");
    let rank = client.get_anime_ranking(RankingType::Airing).await.expect("Unable to get anime ranking");
    println!("{:?}", list);
    println!("{:?}", rank);
}
