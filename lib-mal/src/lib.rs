#[cfg(test)]
mod test;

pub mod model;

use model::{AnimeDetails, AnimeList};

use directories::ProjectDirs;
use pkce;
use reqwest::Method;
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File},
    io::Write,
    str,
    time::SystemTime,
};
use tiny_http::{Response, Server};

pub struct MALClient {
    client_secret: String,
    dirs: ProjectDirs,
    access_token: String,
    client: reqwest::Client,
    caching: bool,
    pub need_auth: bool,
}

impl MALClient {
    pub async fn new(secret: &str, caching: bool) -> Self {
        let client = reqwest::Client::new();
        let mut n_a = false;
        let dir = if let Some(d) = ProjectDirs::from("com", "EmeraldActual", "miru") {
            if !d.data_dir().exists() {
                println!("{}", d.data_dir().display());
                fs::create_dir_all(d.data_dir()).expect("Unable to create data dir");
            }
            if !d.config_dir().exists() {
                fs::create_dir_all(d.config_dir()).expect("Unable to create config dir");
            }
            if !d.cache_dir().exists() {
                fs::create_dir_all(d.cache_dir()).expect("Unable to create cache dir");
                n_a = true;
            }

            d
        } else {
            panic!("Unable to locate application directory");
        };
        let mut token = String::new();
        if dir.cache_dir().join("tokens.json").exists() && caching {
            if let Ok(tokens) = fs::read_to_string(dir.cache_dir().join("tokens.json")) {
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
                            dir.cache_dir().join("tokens.json"),
                            serde_json::to_string(&tok).expect("Unable to parse token struct"),
                        )
                        .expect("Unable to write token file")
                    } else {
                        token = tok.access_token;
                    }
                }
            }
        } else {
            n_a = true;
        }

        let me = MALClient {
            client_secret: secret.to_owned(),
            dirs: dir,
            need_auth: n_a,
            access_token: token,
            client,
            caching,
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
            let mut f = File::create(self.dirs.cache_dir().join("tokens.json"))
                .expect("Unable to create token file");
            f.write_all(serde_json::to_string(&tjson).unwrap().as_bytes())
                .expect("Unable to write tokens");
        }
    }

    //Begin API functions

    ///Returns the user's full anime list as an `AnimeList` struct.
    ///If the request fails for any reason, an `Err` object with a string describing the error is returned instead
    pub async fn get_anime_list(&self) -> Result<AnimeList, String> {
        match self
            .client
            .get("https://api.myanimelist.net/v2/users/@me/animelist?fields=list_status&limit=4")
            .bearer_auth(&self.access_token)
            .send()
            .await
        {
            Ok(res) => Ok(serde_json::from_str(&res.text().await.unwrap()).unwrap()),
            Err(e) => Err(format!("{}", e)),
        }
    }

    ///Gets the deatils for an anime by the show's ID.
    ///Only returns the fields specified in the `fields` parameter
    ///Returns all fields when supplied `None`
    ///
    ///Field options are:
    ///
    ///id,title,main_picture,alternative_titles,
    ///start_date,end_date,synopsis,mean,rank,popularity,num_list_users,
    ///num_scoring_users,nsfw,created_at,updated_at,media_type,status,
    ///genres,my_list_status,num_episodes,start_season,broadcast,source,
    ///average_episode_duration,rating,pictures,background,related_anime,
    ///related_manga,recommendations,studios,statistics
    ///
    pub async fn get_anime_details(
        &self,
        id: &u32,
        fields: Option<Vec<&str>>,
    ) -> Result<AnimeDetails, String> {
        let url = if let Some(f) = fields {
            format!(
                "https://api.myanimelist.net/v2/anime/{}?fields={}",
                id,
                f.join(",")
            )
        } else {
            format!("https://api.myanimelist.net/v2/anime/{}", id)
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
