#[cfg(test)]
mod test;

pub mod model;

use model::{
    fields::AnimeField,
    options::{Params, RankingType, Season},
    AnimeDetails, AnimeList, ForumBoards, ForumTopics, ListStatus, TopicDetails, User,
};

use pkce;
use reqwest::{Method, StatusCode};
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File},
    io::Write,
    path::PathBuf,
    str,
    time::SystemTime,
};
use tiny_http::{Response, Server};

pub struct MALClient {
    client_secret: String,
    dirs: PathBuf,
    access_token: String,
    client: reqwest::Client,
    caching: bool,
    pub need_auth: bool,
}

impl MALClient {
    pub async fn new(secret: &str, caching: bool, cache_dir: Option<PathBuf>) -> Self {
        let client = reqwest::Client::new();
        let mut will_cache = caching;
        let mut n_a = false;
        let dir = if let Some(d) = cache_dir {
            d
        } else {
            println!("No cache directory was provided, disabling caching");
            will_cache = false;
            PathBuf::new()
        };
        let mut token = String::new();
        if will_cache && dir.join("tokens.json").exists() {
            if let Ok(tokens) = fs::read_to_string(dir.join("tokens.json")) {
                let mut tok: Tokens = serde_json::from_str(&tokens).unwrap();
                if let Ok(n) = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
                    if n.as_secs() - tok.today >= tok.expires_in as u64 {
                        let params = [
                            ("grant_type", "refresh_token"),
                            ("refesh_token", &tok.refresh_token),
                        ];
                        let res = client
                            .post("https://myanimelist.net/v1/oauth2/token")
                            .form(&params)
                            .send()
                            .await
                            .expect("Unable to refresh token");
                        let new_toks: TokenResponse =
                            serde_json::from_str(&res.text().await.unwrap())
                                .expect("Unable to parse response");
                        token = new_toks.access_token.clone();
                        tok = Tokens {
                            access_token: new_toks.access_token,
                            refresh_token: new_toks.refresh_token,
                            expires_in: new_toks.expires_in,
                            today: SystemTime::now()
                                .duration_since(SystemTime::UNIX_EPOCH)
                                .unwrap()
                                .as_secs(),
                        };

                        fs::write(
                            dir.join("tokens.json"),
                            serde_json::to_string(&tok).expect("Unable to parse token struct"),
                        )
                        .expect("Unable to write token file")
                    } else {
                        token = tok.access_token;
                    }
                }
            }
        } else {
            will_cache = false;
            n_a = true;
        }

        let me = MALClient {
            client_secret: secret.to_owned(),
            dirs: dir,
            need_auth: n_a,
            access_token: token,
            client,
            caching: will_cache,
        };

        me
    }

    pub fn get_auth_parts(&self) -> (String, String) {
        let verifier = pkce::code_verifier(128);
        let challenge = pkce::code_challenge(&verifier);
        let url = format!("https://myanimelist.net/v1/oauth2/authorize?response_type=code&client_id={}&code_challenge={}&state=bruh", self.client_secret, challenge);
        (url, challenge)
    }

    pub async fn auth(&mut self, challenge: &str) -> Result<(), String> {
        let mut code = "".to_owned();

        let server = Server::http("localhost:2561").unwrap();
        for i in server.incoming_requests() {
            if !i.url().contains("state=bruh") {
                //if the state doesn't match, discard this response
                continue;
            }
            let res_raw = i.url();
            code = res_raw
                .split_once('=')
                .unwrap()
                .1
                .split_once('&')
                .unwrap()
                .0
                .to_owned();
            let response = Response::from_string("You're logged in! You can now close this window");
            i.respond(response).unwrap();
            break;
        }

        self.get_tokens(&code, &challenge).await;
        Ok(())
    }

    async fn get_tokens(&mut self, code: &str, verifier: &str) {
        let params = [
            ("client_id", self.client_secret.as_str()),
            ("grant_type", "authorization_code"),
            ("code_verifier", verifier),
            ("code", code),
        ];
        let rec = self
            .client
            .request(Method::POST, "https://myanimelist.net/v1/oauth2/token")
            .form(&params)
            .build()
            .unwrap();
        let res = self.client.execute(rec).await.unwrap();
        let tokens: TokenResponse = serde_json::from_str(&res.text().await.unwrap()).unwrap();
        self.access_token = tokens.access_token.clone();

        let tjson = Tokens {
            access_token: tokens.access_token,
            refresh_token: tokens.refresh_token,
            expires_in: tokens.expires_in,
            today: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };
        if self.caching {
            let mut f =
                File::create(self.dirs.join("tokens.json")).expect("Unable to create token file");
            f.write_all(serde_json::to_string(&tjson).unwrap().as_bytes())
                .expect("Unable to write tokens");
        }
    }

    ///Sends a get request to the specified URL with the appropriate auth header
    async fn do_request(&self, url: String) -> Result<String, String> {
        match self
            .client
            .get(url)
            .bearer_auth(&self.access_token)
            .send()
            .await
        {
            Ok(res) => Ok(res.text().await.unwrap()),
            Err(e) => Err(format!("{}", e)),
        }
    }

    ///Sends a put request to the specified URL with the appropriate auth header and
    ///form encoded parameters
    async fn do_request_forms(
        &self,
        url: String,
        params: Vec<(&str, String)>,
    ) -> Result<String, String> {
        match self
            .client
            .put(url)
            .bearer_auth(&self.access_token)
            .form(&params)
            .send()
            .await
        {
            Ok(res) => Ok(res.text().await.unwrap()),
            Err(e) => Err(format!("{}", e)),
        }
    }

    ///Tries to parse a JSON response string into the type provided in the `::<>` turbofish
    fn parse_response<'a, T: Serialize + Deserialize<'a>>(
        &self,
        res: &'a str,
    ) -> Result<T, String> {
        match serde_json::from_str::<T>(res) {
            Ok(v) => Ok(v),
            Err(e) => Err(format!("{}", e)),
        }
    }

    //Begin API functions

    //--Anime functions--//
    ///Gets a list of anime based on the query string provided
    ///`limit` defaults to 100 if `None`
    pub async fn get_anime_list(
        &self,
        query: &str,
        limit: Option<u8>,
    ) -> Result<AnimeList, String> {
        let url = format!(
            "https://api.myanimelist.net/v2/anime?q={}&limit={}",
            query,
            limit.unwrap_or(100)
        );
        let res = self.do_request(url).await?;
        self.parse_response(&res)
    }

    ///Gets the deatils for an anime by the show's ID.
    ///Only returns the fields specified in the `fields` parameter
    ///
    ///Returns all fields when supplied `None`
    pub async fn get_anime_details(
        &self,
        id: &u32,
        fields: Option<Vec<AnimeField>>,
    ) -> Result<AnimeDetails, String> {
        let url = if let Some(f) = fields {
            format!(
                "https://api.myanimelist.net/v2/anime/{}?fields={}",
                id,
                f.iter()
                    .map(|v| v.to_string())
                    .collect::<Vec<String>>()
                    .join(",")
            )
        } else {
            format!(
                "https://api.myanimelist.net/v2/anime/{}?fields={}",
                id,
                AnimeField::ALL
            )
        };
        match self
            .client
            .get(url)
            .bearer_auth(&self.access_token)
            .send()
            .await
        {
            Ok(res) => Ok(serde_json::from_str(&res.text().await.unwrap()).unwrap()),
            Err(e) => Err(format!("{}", e)),
        }
    }

    ///Gets a list of anime ranked by `RankingType`
    ///
    ///`limit` defaults to the max of 100 when `None`
    pub async fn get_anime_ranking(
        &self,
        ranking_type: RankingType,
        limit: Option<u8>,
    ) -> Result<AnimeList, String> {
        let url = format!(
            "https://api.myanimelist.net/v2/anime/ranking?ranking_type={}&limit={}",
            ranking_type,
            limit.unwrap_or(100)
        );
        let res = self.do_request(url).await?;
        Ok(serde_json::from_str(&res).unwrap())
    }

    ///Gets the anime for a given season in a given year
    ///
    ///`limit` defaults to the max of 100 when `None`
    pub async fn get_seasonal_anime(
        &self,
        season: Season,
        year: u32,
        limit: Option<u8>,
    ) -> Result<AnimeList, String> {
        let url = format!(
            "https://api.myanimelist.net/v2/anime/season/{}/{}?limit={}",
            year,
            season,
            limit.unwrap_or(100)
        );
        let res = self.do_request(url).await?;
        self.parse_response(&res)
    }

    ///Returns the suggested anime for the current user. Can return an empty list if the user has
    ///no suggestions.
    pub async fn get_suggested_anime(&self, limit: Option<u8>) -> Result<AnimeList, String> {
        let url = format!(
            "https://api.myanimelist.net/v2/anime/suggestions?limit={}",
            limit.unwrap_or(100)
        );
        let res = self.do_request(url).await?;
        self.parse_response(&res)
    }

    //--User anime list functions--//

    ///Adds an anime to the list, or updates the element if it already exists
    pub async fn update_user_anime_status<T: Params>(
        &self,
        id: u32,
        update: T,
    ) -> Result<ListStatus, String> {
        let params = update.get_params();
        let url = format!("https://api.myanimelist.net/v2/anime/{}/my_list_status", id);
        let res = self.do_request_forms(url, params).await?;
        println!("{}", res);
        self.parse_response(&res)
    }

    ///Returns the user's full anime list as an `AnimeList` struct.
    ///If the request fails for any reason, an `Err` object with a string describing the error is returned instead
    pub async fn get_user_anime_list(&self) -> Result<AnimeList, String> {
        let url = "https://api.myanimelist.net/v2/users/@me/animelist?fields=list_status&limit=4";
        let res = self.do_request(url.to_owned()).await?;

        Ok(serde_json::from_str(&res).unwrap())
    }

    ///Deletes the anime with `id` from the user's anime list
    ///
    ///Returns 404 if the id isn't in the list.
    pub async fn delete_anime_list_item(&self, id: u32) -> Result<(), String> {
        let url = format!("https://api.myanimelist.net/v2/anime/{}/my_list_status", id);
        let res = self
            .client
            .delete(url)
            .bearer_auth(&self.access_token)
            .send()
            .await;
        match res {
            Ok(r) => {
                if r.status() == StatusCode::NOT_FOUND {
                    Err(format!("Anime {} not found", id))
                } else {
                    Ok(())
                }
            }
            Err(e) => Err(format!("{}", e)),
        }
    }

    //--Forum functions--//

    ///Returns a vector of `HashMap`s that represent all the forum boards on MAL
    pub async fn get_forum_boards(&self) -> Result<ForumBoards, String> {
        let res = self
            .do_request("https://api.myanimelist.net/v2/forum/boards".to_owned())
            .await?;
        self.parse_response(&res)
    }

    pub async fn get_forum_topic_detail(
        &self,
        topic_id: u32,
        limit: Option<u8>,
    ) -> Result<TopicDetails, String> {
        let url = format!(
            "https://api.myanimelist.net/v2/forum/topic/{}?limit={}",
            topic_id,
            limit.unwrap_or(100)
        );
        let res = self.do_request(url).await?;
        self.parse_response(&res)
    }

    pub async fn get_forum_topics(
        &self,
        board_id: Option<u32>,
        subboard_id: Option<u32>,
        query: Option<String>,
        topic_user_name: Option<String>,
        user_name: Option<String>,
        limit: Option<u32>,
    ) -> Result<ForumTopics, String> {
        let params = {
            let mut tmp = vec![];
            if let Some(bid) = board_id {
                tmp.push(format!("board_id={}", bid.to_string()));
            }
            if let Some(bid) = subboard_id {
                tmp.push(format!("subboard_id={}", bid.to_string()));
            }
            if let Some(bid) = query {
                tmp.push(format!("q={}", bid.to_string()));
            }
            if let Some(bid) = topic_user_name {
                tmp.push(format!("topic_user_name={}", bid.to_string()));
            }
            if let Some(bid) = user_name {
                tmp.push(format!("user_name={}", bid));
            }
            tmp.push(format!("limit={}", limit.unwrap_or(100)));
            tmp.join(",")
        };
        let url = format!("https://api.myanimelist.net/v2/forum/topics?{}", params);
        let res = self.do_request(url).await?;
        self.parse_response(&res)
    }

    ///Gets the details for the current user
    ///
    ///`fields` defaults to `anime_statistics` if `None`
    pub async fn get_my_user_info(&self, fields: Option<&str>) -> Result<User, String> {
        let url = format!(
            "https://api.myanimelist.net/v2/users/@me?fields={}",
            fields.unwrap_or("anime_statistics")
        );
        let res = self.do_request(url).await?;
        self.parse_response(&res)
    }
}

#[derive(Deserialize, Debug)]
struct TokenResponse {
    token_type: String,
    expires_in: u32,
    access_token: String,
    refresh_token: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Tokens {
    access_token: String,
    refresh_token: String,
    expires_in: u32,
    today: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct APIError {
    pub error: String,
    pub message: String,
}
