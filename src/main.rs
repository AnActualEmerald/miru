use std::fs;

use clap::{crate_version, App, Arg, SubCommand};
use directories::ProjectDirs;
use lib_mal::{MALClient, MALError};
use spinners::{self, Spinner, Spinners};
use std::io::{stdout, Write};
use tokio;
use webbrowser;

use crate::utils::clear_spinner;

mod subcommands;
mod utils;

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
        .subcommand(
            SubCommand::with_name("increment")
                .aliases(&["inc", "i"])
                .arg(
                    Arg::with_name("ID")
                        .help("ID of the anime to increment")
                        .takes_value(true)
                        .required_unless("title"),
                )
                .arg(
                    Arg::with_name("title")
                        .long("title")
                        .short("t")
                        .help("Search for the show to increment by title \nNot as reliable as using the ID")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("amount")
                        .short("n")
                        .long("num")
                        .help("Amount to increment the show by")
                        .default_value("1")
                        .validator(|v| {
                            if let Err(e) = v.parse::<i32>() {
                                Err(format!("amount must be a number: {}", e))
                            } else {
                                Ok(())
                            }
                        }),
                )
                .about("Increment the episodes watched of a show on your anime list"),
        )
            .subcommand(SubCommand::with_name("search").alias("s").arg(Arg::with_name("TITLE").help("Title to search for").required(true)))
            .subcommand(SubCommand::with_name("add").alias("a").help("Add an anime to your anime list").arg(Arg::with_name("ID").help("ID of the anime to add to your list").required_unless("title").takes_value(true).validator(|v| {
                if let Err(_) = v.parse::<u32>() {
                    Err(format!("ID must be a number!"))
                } else {
                    Ok(())
                }
            })).arg(Arg::with_name("title").takes_value(true).long("title").short("t").help("Title to add NOTE: not as reliable as using the ID")))
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
    let mut client = MALClient::init(
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

async fn login(client: &mut MALClient) -> Result<(), MALError> {
    let (url, challenge, state) = client.get_auth_parts();
    let sp = Spinner::new(&Spinners::Arrow3, "Opening browser to log in...".into());
    let mut stdout = stdout();
    if let Err(e) = webbrowser::open(&url) {
        clear_spinner(sp);
        println!("\rUnable to open browser due to: {}", e);
        println!("Open this link to log in... => {}", url);
        client.auth("localhost:2561", &challenge, &state).await?;
    } else {
        client.auth("localhost:2561", &challenge, &state).await?;
        clear_spinner(sp);
    }
    stdout.flush().expect("Unable to flush stdout");
    Ok(())
}