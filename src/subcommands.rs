use clap::ArgMatches;
use cli_table::{format::Justify, print_stdout, Cell, Style, Table};
use lib_mal::MALClient;

pub async fn do_command<'a>(args: &'a ArgMatches<'a>, client: &'a MALClient) -> Result<(), String> {
    if let Some(l) = args.subcommand_matches("list") {
        list(l, client).await?;
    }

    Ok(())
}

async fn list<'a>(args: &'a ArgMatches<'a>, client: &'a MALClient) -> Result<(), String> {
    match client.get_user_anime_list().await {
        Ok(mal) => {
            let mut table = vec![];
            for anime in mal.data {
                let ls = anime.list_status.unwrap();

                table.push(vec![
                    anime.node.id.cell(),
                    anime.node.title.cell(),
                    ls.score.unwrap().to_string().cell(),
                    ls.status.unwrap().cell(),
                    ls.num_episodes_watched.unwrap().to_string().cell(),
                    ls.is_rewatching.unwrap().to_string().cell(),
                ]);
            }
            if let Err(_) = print_stdout(table.table().title(vec![
                "Anime ID".cell(),
                "Title".cell(),
                "Score".cell(),
                "Status".cell(),
                "Episodes Watched".cell(),
                "Rewatching?".cell(),
            ])) {
                println!("unable to print table...");
            }
        }
        Err(e) => {
            eprintln!("Error getting anime list {}", e);
        }
    }

    Ok(())
}
