#[cfg(test)]
mod test;

use crypto::{digest::Digest, sha3::*};
use directories::{self, ProjectDirs};
use pkce;
use rand::{self, random};
use reqwest::Method;
use serde::Deserialize;
use std::{
    fs::{self, File},
    io::Write,
    path::Path,
    str,
};
use tiny_http::{Response, Server};

pub struct MALClient {
    client_secret: String,
    dirs: ProjectDirs,
    need_auth: bool,
}

impl MALClient {
    pub fn new(secret: &str) -> Self {
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

        MALClient {
            client_secret: secret.to_owned(),
            dirs: dir,
            need_auth: n_a,
        }
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
        let client = reqwest::Client::new();
        let params = [
            ("client_id", self.client_secret.as_str()),
            ("grant_type", "authorization_code"),
            ("code_verifier", verifier),
            ("code", code),
        ];
        let rec = client
            .request(Method::POST, "https://myanimelist.net/v1/oauth2/token")
            .form(&params)
            .build()
            .unwrap();
        let res = client.execute(rec).await.unwrap();
        let tokens: TokenResponse = serde_json::from_str(&res.text().await.unwrap()).unwrap();

        {
            let mut access = File::create(self.dirs.cache_dir().join("access_token.tok"))
                .expect("Unable to create token file");
            access
                .write_all(tokens.access_token.as_bytes())
                .expect("Unable to write access token");
        }
        {
            let mut refresh = File::create(self.dirs.cache_dir().join("refresh_token.tok"))
                .expect("Unable to create token file");
            refresh
                .write_all(tokens.refresh_token.as_bytes())
                .expect("Unable to write refresh token");
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
