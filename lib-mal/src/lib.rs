#[cfg(test)]
mod test;

pub mod model;

use model::AnimeList;

use directories::{self, ProjectDirs};
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
    pub need_auth: bool,
}

impl MALClient {
    pub async fn new(secret: &str) -> Self {
        let client = reqwest::Client::new();
        let mut n_a = false;
        let dir = if let Some(d) = ProjectDirs::from("com", "EmeraldActual", "miru") {
            if !d.data_dir().exists() {
                fs::create_dir(d.data_dir()).expect("Unable to create data dir");
            }
            if !d.config_dir().exists() {
                fs::create_dir(d.config_dir()).expect("Unable to create config dir");
            }
            if !d.cache_dir().exists() {
                fs::create_dir(d.cache_dir()).expect("Unable to create cache dir");
                n_a = true;
            }

            d
        } else {
            panic!("Unable to locate application directory");
        };
        let mut token = String::new();
        if dir.cache_dir().join("tokens.json").exists() {
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
        };

        me
    }

    pub fn get_auth_parts(&self) -> (String, String) {
        let verifier = pkce::code_verifier(128);
        let challenge = pkce::code_challenge(&verifier);
        let url = format!("https://myanimelist.net/v1/oauth2/authorize?response_type=code&client_id={}&code_challenge={}&state=bruh", self.client_secret, challenge);
        (url, challenge)
    }

    pub async fn auth(&self, challenge: &str) -> Result<(), String> {
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

    async fn get_tokens(&self, code: &str, verifier: &str) {
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
        let tjson = Tokens {
            access_token: tokens.access_token,
            refresh_token: tokens.refresh_token,
            expires_in: tokens.expires_in,
            today: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };
        {
            let mut f = File::create(self.dirs.cache_dir().join("tokens.json"))
                .expect("Unable to create token file");
            f.write_all(serde_json::to_string(&tjson).unwrap().as_bytes())
                .expect("Unable to write tokens");
        }
    }

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
