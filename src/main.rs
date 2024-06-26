use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::{io, panic};

use clap::{Parser, ValueEnum};
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, AUTHORIZATION, CONTENT_TYPE};
use reqwest::Client;
use serde_json::json;

mod oauth;
mod xmlformat;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum ListType {
    Anime,
    Manga,
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, help = "Username on AniList")]
    user: String,
    #[arg(short = 'l', long = "list")]
    list_type: ListType,
    #[arg(short, long, value_name = "FILE", help = "Example: anime-list.xml")]
    file: PathBuf,
    #[arg(long = "no-update", action = clap::ArgAction::SetFalse)]
    update: bool,
    #[arg(short, long, help = "Use OAuth to export hidden entries")]
    oauth: bool,
    #[arg(long = "no-nsfw", action = clap::ArgAction::SetFalse)]
    nsfw: bool,
}

const LIST_QUERY: &str = "
query ($userName: String, $type: MediaType) {
  MediaListCollection(userName: $userName, type: $type) {
    user {
      id
    }
    lists {
      entries {
        id
        status
        repeat
        progress
        progressVolumes
        customLists
        hiddenFromStatusLists
        startedAt {
          year
          month
          day
        }
        completedAt {
          year
          month
          day
        }
        createdAt
        updatedAt
        score
        notes
        media {
          idMal
          isAdult
          title {
            romaji
          }
          format
          episodes
          chapters
          volumes
        }
        priority
      }
      isCustomList
    }
  }
}
";

const ANISTATS_QUERY: &str = "
query ($name: String) {
  User(name: $name) {
    id
    name
    statistics {
      anime {
        count
        statuses {
          status
          count
        }
      }
    }
  }
}
";

const MANGASTATS_QUERY: &str = "
query ($name: String) {
  User(name: $name) {
    id
    name
    statistics {
      manga {
        count
        statuses {
          status
          count
        }
      }
    }
  }
}";

#[derive(PartialEq)]
enum QueryType {
    LIST,
    STATS,
}

async fn make_query(
    query: &str,
    client: &reqwest::Client,
    username: &String,
    qtype: QueryType,
    list_type: Option<&str>,
    auth_pin: &String,
) -> serde_json::Value {
    let stats_query_json = json!({
        "query" : query,
        "variables" : {
            "name" : username
        }
    });
    let list_query_json = json!({
        "query" : query,
        "variables" : {
            "userName" : username,
            "type" : list_type
        }
    });
    let mut headers = HeaderMap::new();
    if auth_pin != "" {
        let headerv = HeaderValue::from_str(auth_pin);
        match headerv {
            Ok(_) => {}
            Err(a) => {
                println!("{}", a)
            }
        }
        headers.insert(AUTHORIZATION, HeaderValue::from_str(auth_pin).unwrap());
    }
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    headers.insert(ACCEPT, HeaderValue::from_static("application/json"));

    let resp = client
        .post("https://graphql.anilist.co/")
        .headers(headers)
        .body(if qtype == QueryType::LIST {
            list_query_json.to_string()
        } else {
            stats_query_json.to_string()
        })
        .send()
        .await
        .unwrap()
        .text()
        .await;
    serde_json::from_str(&resp.unwrap()).unwrap()
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    panic::set_hook(Box::new(|p| {
        eprintln!("Panic: {}", p);
        std::process::exit(1);
    }));

    let args = Args::parse();
    let client = Client::new();
    let mut auth_pin: String = String::new();
    if args.oauth {
        println!("OAuth was enabled, please visit and authenticate through the following link in your browser: {}", oauth::gen_url("18309"));
        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read input");
        let input = input.trim();
        auth_pin.push_str("Bearer ");
        auth_pin.push_str(&input);
    }

    let path: &str = args.file.to_str().expect("couldn't decode file path");
    // create file / flush contents of an existing file
    #[allow(unused_assignments)]
    let mut f = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(path)?;
    f = OpenOptions::new().write(true).append(true).open(path)?;

    #[allow(unused_assignments)]
    let mut result: serde_json::Value = json!(null);
    let pre_user_statistics: Result<xmlformat::UserStatistics, _> = match args.list_type {
        ListType::Anime => {
            result = make_query(
                ANISTATS_QUERY,
                &client,
                &args.user,
                QueryType::STATS,
                None,
                &auth_pin,
            )
            .await;
            serde_json::from_value(result["data"]["User"]["statistics"]["anime"].to_owned())
        }
        ListType::Manga => {
            result = make_query(
                MANGASTATS_QUERY,
                &client,
                &args.user,
                QueryType::STATS,
                None,
                &auth_pin,
            )
            .await;
            serde_json::from_value(result["data"]["User"]["statistics"]["manga"].to_owned())
        }
    };
    match pre_user_statistics {
        Ok(_) => {}
        Err(_) => {
            panic!("OAuth token usage failed")
        }
    };
    let user_statistics =
        pre_user_statistics.expect("an error has occured while parsing UserStatistics from API");

    // header
    writeln!(f, "<?xml version=\"1.0\" encoding=\"UTF-8\"?>")?;
    writeln!(f, "<myanimelist>")?;
    writeln!(f, "{}", xmlformat::xml_export_comment(&args.user))?;
    writeln!(f, "\t<myinfo>")?;
    writeln!(f, "{}", {
        match args.list_type {
            ListType::Anime => xmlformat::xml_animeheader(
                user_statistics,
                result["data"]["User"]["id"].as_u64().unwrap(),
                result["data"]["User"]["name"].as_str().unwrap().to_string(),
            ),
            ListType::Manga => xmlformat::xml_mangaheader(
                user_statistics,
                result["data"]["User"]["id"].as_u64().unwrap(),
                result["data"]["User"]["name"].as_str().unwrap().to_string(),
            ),
        }
    })?;
    writeln!(f, "\t</myinfo>")?;

    let mut status_media_list: Vec<xmlformat::MediaEntry> = Vec::new();
    let mut custom_media_list: Vec<xmlformat::MediaEntry> = Vec::new();
    let result = match args.list_type {
        ListType::Anime => {
            make_query(
                LIST_QUERY,
                &client,
                &args.user,
                QueryType::LIST,
                Some("ANIME"),
                &auth_pin,
            )
            .await
        }
        ListType::Manga => {
            make_query(
                LIST_QUERY,
                &client,
                &args.user,
                QueryType::LIST,
                Some("MANGA"),
                &auth_pin,
            )
            .await
        }
    };

    let lists: Vec<xmlformat::MediaListGroup> =
        serde_json::from_value::<Vec<xmlformat::MediaListGroup>>(
            result["data"]["MediaListCollection"]["lists"].clone(),
        )
        .expect("unexpected error occured while parsing user lists");

    for list in &lists {
        if list.isCustomList {
            custom_media_list.extend(list.entries.clone())
        } else {
            status_media_list.extend(list.entries.clone())
        }
    }

    for media_entry in status_media_list {
        if !(args.nsfw == false && media_entry.media.isAdult == true) {
            match args.list_type {
                ListType::Anime => {
                    writeln!(f, "{}", xmlformat::xml_anime(media_entry, args.update))?
                }
                ListType::Manga => {
                    writeln!(f, "{}", xmlformat::xml_manga(media_entry, args.update))?
                }
            }
        }
    }
    for media_entry in custom_media_list {
        if media_entry.hiddenFromStatusLists
            && !(args.nsfw == false && media_entry.media.isAdult == true)
        {
            match args.list_type {
                ListType::Anime => {
                    writeln!(f, "{}", xmlformat::xml_anime(media_entry, args.update))?
                }
                ListType::Manga => {
                    writeln!(f, "{}", xmlformat::xml_manga(media_entry, args.update))?
                }
            }
        }
    }
    writeln!(f, "</myanimelist>")?;

    f.flush()?;
    drop(f);

    Ok(())
}
