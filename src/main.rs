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
}

const LIST_QUERY: &str = "
query ($page : Int, $perPage : Int, $userName : String, $type: MediaType) {
  Page(page: $page, perPage: $perPage) {
    pageInfo {
      total
      currentPage
      lastPage
      hasNextPage
      perPage
    }
    mediaList(userName: $userName, sort: SCORE_DESC, type: $type) {
      status
      repeat
      progress
      customLists
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
    }
  }
}";

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
    page: Option<i8>,
    qtype: QueryType,
) -> serde_json::Value {
    let list_query_json = json!({
        "query" : query,
        "variables" : {
            "page" : page,
            "perPage" : 50, // maximum per page
            "userName" : username,
            "type" : "ANIME"
        }
    });
    let stats_query_json = json!({
        "query" : query,
        "variables" : {
            "name" : username
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

    let mut page_counter: i8 = 0;
    let path: &str = args.file.to_str().expect("couldn't decode file path");
    // create file / flush contents of an existing file
    #[allow(unused_assignments)]
    let mut f = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(path)?;
    f = OpenOptions::new().write(true).append(true).open(path)?;

    let result = make_query(STATS_QUERY, &client, &args.user, None, QueryType::STATS).await;
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

    let mut media_list: Vec<serde_json::Value> = Vec::new();
    while {
        page_counter += 1;
        let result = make_query(
            LIST_QUERY,
            &client,
            &args.user,
            Some(page_counter),
            QueryType::LIST,
        )
        .await;
        media_list.extend(
            result["data"]["Page"]["mediaList"]
                .as_array()
                .expect("unexpected error occured while deserializing mediaList")
                .to_owned(),
        );
        match result["data"]["Page"]["pageInfo"]["hasNextPage"].as_bool() {
            Some(v) => v,
            None => false,
        }
    } {} // do-while
    for media_entry in media_list {
        let media_entry_parsed: xmlformat::AnimeEntry = serde_json::from_value(media_entry)
            .expect("unexpected error occured while deserializing anime list entry");
        writeln!(f, "{}", xmlformat::xml_anime(media_entry_parsed))?;
    }
    writeln!(f, "</myanimelist>")?;

    f.flush()?;
    drop(f);

    Ok(())
}
