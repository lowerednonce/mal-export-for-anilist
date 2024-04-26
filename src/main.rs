use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;

use clap::Parser;
use reqwest::Client;
use serde_json::json;

mod xmlformat;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    user: String,
    #[arg(short, long, value_name = "FILE")]
    file: PathBuf,
    #[arg(short = 'n', long = "no-update", action= clap::ArgAction::SetFalse)]
    update: bool,
}

const LIST_QUERY: &str = "
query ($userName : String, $type: MediaType) {
  MediaListCollection (userName: $userName, type: $type) {
    user {
      id
    }
    lists {
      # name
      entries {
        id
        status
      	repeat
      	progress
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
	        title {
	          romaji
	        }
	        format
	        episodes
	    }
	    priority
      }
      isCustomList
      # isSplitCompletedList
      # status
    }
    hasNextChunk
  }
}
";

const STATS_QUERY: &str = "
query ($name : String) {
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
            "type" : "ANIME"
        }
    });

    let resp = client
        .post("https://graphql.anilist.co/")
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
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
    let args = Args::parse();
    let client = Client::new();

    let path: &str = args.file.to_str().expect("couldn't decode file path");
    // create file / flush contents of an existing file
    #[allow(unused_assignments)]
    let mut f = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(path)?;
    f = OpenOptions::new().write(true).append(true).open(path)?;

    let result = make_query(STATS_QUERY, &client, &args.user, QueryType::STATS).await;
    let user_statistics: xmlformat::UserStatistics =
        serde_json::from_value(result["data"]["User"]["statistics"]["anime"].to_owned())
            .expect("unexpected error occured while parsing in user statistics");

    // header
    writeln!(f, "<?xml version=\"1.0\" encoding=\"UTF-8\"?>")?;
    writeln!(f, "<myanimelist>")?;
    writeln!(f, "{}", xmlformat::xml_export_comment(&args.user))?;
    writeln!(f, "\t<myinfo>")?;
    writeln!(
        f,
        "{}",
        xmlformat::xml_header(
            user_statistics,
            result["data"]["User"]["id"].as_u64().unwrap(),
            result["data"]["User"]["name"].as_str().unwrap().to_string()
        )
    )?;
    writeln!(f, "\t</myinfo>")?;

    let mut status_media_list: Vec<xmlformat::AnimeEntry> = Vec::new();
    let mut custom_media_list: Vec<xmlformat::AnimeEntry> = Vec::new();
    let result = make_query(LIST_QUERY, &client, &args.user, QueryType::LIST).await;

    let lists: Vec<xmlformat::AnimeList> = serde_json::from_value::<Vec<xmlformat::AnimeList>>(
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
        writeln!(f, "{}", xmlformat::xml_anime(media_entry, args.update))?;
    }
    for media_entry in custom_media_list {
        if media_entry.hiddenFromStatusLists {
            writeln!(f, "{}", xmlformat::xml_anime(media_entry, args.update))?;
        }
    }
    writeln!(f, "</myanimelist>")?;

    f.flush()?;
    drop(f);

    Ok(())
}
