use crate::clear_spinner;
use clap::ArgMatches;
use cli_table::{print_stdout, Cell, Table};
use lib_mal::{
    model::{fields::AnimeField, options::StatusUpdate},
    MALClient,
};
use spinners::{Spinner, Spinners};

pub async fn do_command<'a>(args: &'a ArgMatches<'a>, client: &'a MALClient) -> Result<(), String> {
    if let Some(l) = args.subcommand_matches("list") {
        list(l, client).await?;
    }
    if let Some(i) = args.subcommand_matches("increment") {
        inc(i, client).await?;
    }

    Ok(())
}

async fn list<'a>(_args: &'a ArgMatches<'a>, client: &'a MALClient) -> Result<(), String> {
    let sp = Spinner::new(&Spinners::Dots5, "Working...".into());
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
            clear_spinner(sp);
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

async fn inc<'a>(args: &'a ArgMatches<'a>, client: &'a MALClient) -> Result<(), String> {
    let amnt: i32 = args.value_of("amount").unwrap().parse().unwrap();
    let sp = Spinner::new(&Spinners::Dots5, "Working...".into());
    if let Some(id) = args.value_of("ID") {
        let current = client
            .get_anime_details(id.parse().unwrap(), Some(vec![AnimeField::MyListStatus]))
            .await?;
        let eps = current
            .my_list_status
            .unwrap()
            .num_episodes_watched
            .unwrap() as i32;
        let mut update = StatusUpdate::default();
        update.num_watched_episodes((eps + amnt) as u32);
        client
            .update_user_anime_status(id.parse().unwrap(), update)
            .await?;
        clear_spinner(sp);
        println!("List updated!");
    } else if let Some(title) = args.value_of("title") {
        sp.message(format!(
            "Searching for a show with the title \"{}\"...",
            title
        ));
        let list = client.get_anime_list(title, Some(1)).await?;
        sp.message(format!("Found anime {}", list.data[0].node.title));

        let current = client
            .get_anime_details(list.data[0].node.id, Some(vec![AnimeField::MyListStatus]))
            .await?;

        if let Some(mls) = current.my_list_status {
            let eps = mls.num_episodes_watched.unwrap() as i32;
            let mut update = StatusUpdate::default();
            update.num_watched_episodes((eps + amnt) as u32);
            client
                .update_user_anime_status(list.data[0].node.id, update)
                .await?;
            clear_spinner(sp);
            println!("List updated!");
        } else {
            clear_spinner(sp);
            println!(
                "Couldn't find anime \"{}\" in your anime list",
                list.data[0].node.title
            );
        }
    }

    Ok(())
}
