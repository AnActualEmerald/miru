use std::fs;

use clap::{crate_version, App, Arg, SubCommand};
use crossterm::terminal::{self, ClearType};
use crossterm::ExecutableCommand;
use directories::ProjectDirs;
use lib_mal::MALClient;
use spinners::{self, Spinner, Spinners};
use std::io::{stdout, Read, Write};
use tokio;
use webbrowser;

mod subcommands;

#[tokio::main]
async fn main() {
    let matches = App::new("Miru")
        .about("A command line MyAnimeList.net client")
        .author("Written by Emerald | emerald_actual@protonmail.com")
        .version(crate_version!())
        .arg(
            Arg::with_name("login")
                .long("login")
                .short("l")
                .help("Forces miru to log you in even if your tokens are still good"),
        )
        .arg(
            Arg::with_name("cache")
                .long("no-cache")
                .short("c")
                .help("Disables token caching"),
        )
        .subcommand(
            SubCommand::with_name("list")
                .alias("l")
                .about("Retreives your AnimeList from MAL"),
        )
        .get_matches();

    let cache_dir = if let Some(d) = ProjectDirs::from("com", "EmeraldActual", "miru") {
        if d.cache_dir().exists() {
            Some(d.cache_dir().to_path_buf())
        } else {
            fs::create_dir_all(d.cache_dir()).expect("Unable to create cache directory");
            Some(d.cache_dir().to_path_buf())
        }
    } else {
        None
    };

    //get the client ready
    let mut client = MALClient::new(
        include_str!("secret"),
        !matches.is_present("cache"),
        cache_dir,
    )
    .await;
    if client.need_auth || matches.is_present("login") {
        match login(&mut client).await {
            Err(e) => eprintln!("Unable to log in: {}", e),
            _ => {}
        }
    }

    subcommands::do_command(&matches, &client)
        .await
        .expect("Command failed");
}

async fn login(client: &mut MALClient) -> Result<(), String> {
    let (url, challenge, state) = client.get_auth_parts();
    let sp = Spinner::new(&Spinners::Arrow3, "Opening browser to log in...".into());
    let mut stdout = stdout();
    if let Err(e) = webbrowser::open(&url) {
        sp.stop();
        stdout
            .execute(terminal::Clear(ClearType::CurrentLine))
            .expect("Unable to clear line");
        println!("\rUnable to open browser due to: {}", e);
        println!("Open this link to log in... => {}", url);
        client.auth("localhost:2561", &challenge, &state).await?;
    } else {
        client.auth("localhost:2561", &challenge, &state).await?;

        sp.stop();
        stdout
            .execute(terminal::Clear(ClearType::CurrentLine))
            .expect("Unable to clear line");
        print!("\r");
    }
    stdout.flush().expect("Unable to flush stdout");
    Ok(())
}
