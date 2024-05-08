use chrono::{Datelike, Local};
use serde::Deserialize;

#[derive(Deserialize, PartialEq, Copy, Clone)]
pub enum Status {
    CURRENT,
    PLANNING,
    COMPLETED,
    DROPPED,
    PAUSED,
    REPEATING,
}
#[derive(Deserialize, Clone)]
#[allow(non_camel_case_types)]
enum Format {
    TV,
    TV_SHORT,
    MOVIE,
    SPECIAL,
    OVA,
    ONA,
    MUSIC,
    MANGA,
    NOVEL,
    ONE_SHOT,
    UNKNOWN, // UNKNOWN reserved because of Rust
}

#[derive(Deserialize, Clone)]
struct Date {
    year: Option<u16>,
    month: Option<u8>,
    day: Option<u8>,
}
#[derive(Deserialize, Clone, PartialEq)]
pub struct Title {
    romaji: String,
}
#[derive(Deserialize, Clone)]
#[allow(non_snake_case)]
pub struct Media {
    idMal: Option<u64>,
    pub isAdult: bool,
    pub title: Title,
    format: Option<Format>,
    episodes: Option<u64>,
    chapters: Option<u64>,
    volumes: Option<u64>,
}
#[derive(Deserialize, Copy, Clone)]
struct StatusEntry {
    status: Status,
    count: u32,
}

#[derive(Deserialize, Clone)]
#[allow(non_snake_case)]
pub struct MediaEntry {
    status: Status,
    repeat: u32, // just to be safe
    progress: u32,
    progressVolumes: Option<u32>,
    customLists: serde_json::Value,
    pub hiddenFromStatusLists: bool,
    startedAt: Date,
    completedAt: Date,
    score: f32,
    notes: Option<String>,
    pub media: Media,
}

#[derive(Deserialize)]
#[allow(non_snake_case)]
pub struct MediaListGroup {
    pub entries: Vec<MediaEntry>,
    // name: String,
    pub isCustomList: bool,
    // isSplitCompletedList: bool,
    // pub status: Option<Status>,
}

#[derive(Deserialize)]
#[allow(non_snake_case)]
pub struct UserStatistics {
    count: u64,
    statuses: Vec<StatusEntry>,
}

impl std::string::ToString for Status {
    fn to_string(&self) -> String {
        match self {
            Status::CURRENT => String::from("Watching"),
            Status::PLANNING => String::from("Plan to Watch"),
            Status::COMPLETED => String::from("Completed"),
            Status::DROPPED => String::from("Dropped"),
            Status::PAUSED => String::from("On-Hold"),
            Status::REPEATING => String::from("Completed"),
        }
    }
}

fn to_string_manga(status: Status) -> String {
    match status {
        Status::CURRENT => String::from("Reading"),
        Status::PLANNING => String::from("Plan to Read"),
        _ => status.to_string(),
    }
}

impl std::string::ToString for Format {
    fn to_string(&self) -> String {
        match self {
            Format::TV => String::from("TV"),
            Format::TV_SHORT => String::from("TV_SHORT"),
            Format::MOVIE => String::from("MOVIE"),
            Format::SPECIAL => String::from("SPECIAL"),
            Format::OVA => String::from("OVA"),
            Format::ONA => String::from("ONA"),
            Format::MUSIC => String::from("MUSIC"),
            Format::MANGA => String::from("MANGA"),
            Format::NOVEL => String::from("NOVEL"),
            Format::ONE_SHOT => String::from("ONE_SHOT"),
            Format::UNKNOWN => String::from("Unknown"),
        }
    }
}

impl std::string::ToString for Date {
    fn to_string(&self) -> String {
        let mut buffer: String = String::new();
        match self.year {
            Some(i) => buffer.push_str(&i.to_string()),
            None => buffer.push_str("0000"),
        };
        buffer.push_str("-");
        match self.month {
            Some(i) => buffer.push_str(&i.to_string()),
            None => buffer.push_str("00"),
        };
        buffer.push_str("-");
        match self.day {
            Some(i) => buffer.push_str(&i.to_string()),
            None => buffer.push_str("00"),
        };

        buffer
    }
}

fn lists_to_tags(custom_lists: serde_json::Value) -> String {
    match custom_lists.as_object() {
        Some(map) => map
            .iter()
            .filter_map(|(key, value)| {
                if value.as_bool().unwrap_or(false) {
                    Some(key.clone())
                } else {
                    None
                }
            })
            .collect::<Vec<String>>()
            .join(", "),
        None => String::new(),
    }
}

fn xml_str(input: &str) -> String {
    // sanitize XML strings
    // as per https://www.w3resource.com/xml/prohibited-character-literals.php
    let mut out = str::replace(input, "&", "&amp;");
    out = str::replace(&out, "<", "&lt;");
    out = str::replace(&out, ">", "&gt;");
    out = str::replace(&out, "'", "&apos;");
    out = str::replace(&out, "\"", "&quot;");

    out
}

fn xml_tag(indent: Option<i8>, tag: &str, contents: &str) -> String {
    let mut xml: String = String::new();
    xml.extend((0..(indent.unwrap_or(0))).map(|_| String::from("\t"))); // sorry

    xml.push_str("<");
    xml.push_str(tag);
    xml.push_str(">");

    // possible optimization to consider
    // for this application, only one type of tag will need to be escaped
    xml.push_str(&xml_str(contents));

    xml.push_str("</");
    xml.push_str(tag);
    xml.push_str(">\n");
    xml
}

fn string_option_unwrap(string: Option<String>) -> String {
    match string {
        None => String::new(),
        Some(content) => content,
    }
}

pub fn xml_animeheader(stats: UserStatistics, id: u64, name: String) -> String {
    let default_status: StatusEntry = StatusEntry {
        status: Status::CURRENT, // ignored
        count: 0,
    };
    let default_status_ref = &&default_status;

    let closure = |entry: &Vec<StatusEntry>, status: Status| {
        **entry
            .into_iter()
            .filter(|s| s.status == status)
            .collect::<Vec<_>>()
            .get(0)
            .unwrap_or(default_status_ref)
    };
    let status_watching: StatusEntry = closure(&stats.statuses, Status::CURRENT);
    let status_rewatching: StatusEntry = closure(&stats.statuses, Status::REPEATING);
    let status_completed: StatusEntry = closure(&stats.statuses, Status::COMPLETED);
    let status_onhold: StatusEntry = closure(&stats.statuses, Status::PAUSED);
    let status_dropped: StatusEntry = closure(&stats.statuses, Status::DROPPED);
    let status_plantowatch: StatusEntry = closure(&stats.statuses, Status::PLANNING);

    let mut header: String = String::new();
    header.push_str(&xml_tag(Some(2), "user_id", &id.to_string()));
    header.push_str(&xml_tag(Some(2), "user_name", &name));
    header.push_str(&xml_tag(Some(2), "user_export_type", "1"));
    header.push_str(&xml_tag(
        Some(2),
        "user_total_anime",
        &stats.count.to_string(),
    ));
    header.push_str(&xml_tag(
        Some(2),
        "user_total_watching",
        &(status_watching.count + status_rewatching.count).to_string(),
    ));
    header.push_str(&xml_tag(
        Some(2),
        "user_total_completed",
        &status_completed.count.to_string(),
    ));
    header.push_str(&xml_tag(
        Some(2),
        "user_total_onhold",
        &status_onhold.count.to_string(),
    ));
    header.push_str(&xml_tag(
        Some(2),
        "user_total_dropped",
        &status_dropped.count.to_string(),
    ));
    header.push_str(&xml_tag(
        Some(2),
        "user_total_plantowatch",
        &status_plantowatch.count.to_string(),
    ));

    header
}

pub fn xml_mangaheader(stats: UserStatistics, id: u64, name: String) -> String {
    let default_status: StatusEntry = StatusEntry {
        status: Status::CURRENT, // ignored
        count: 0,
    };
    let default_status_ref = &&default_status;

    let closure = |entry: &Vec<StatusEntry>, status: Status| {
        **entry
            .into_iter()
            .filter(|s| s.status == status)
            .collect::<Vec<_>>()
            .get(0)
            .unwrap_or(default_status_ref)
    };

    // NOTE: AniList only seems to report Planning, Completed, Dropped, and Paused statuses. This
    // is left here for compatibility
    let status_reading: StatusEntry = closure(&stats.statuses, Status::CURRENT);
    let status_rereading: StatusEntry = closure(&stats.statuses, Status::REPEATING);
    let status_completed: StatusEntry = closure(&stats.statuses, Status::COMPLETED);
    let status_onhold: StatusEntry = closure(&stats.statuses, Status::PAUSED);
    let status_dropped: StatusEntry = closure(&stats.statuses, Status::DROPPED);
    let status_plantowatch: StatusEntry = closure(&stats.statuses, Status::PLANNING);

    let mut header: String = String::new();
    header.push_str(&xml_tag(Some(2), "user_id", &id.to_string()));
    header.push_str(&xml_tag(Some(2), "user_name", &name));
    header.push_str(&xml_tag(Some(2), "user_export_type", "2"));
    header.push_str(&xml_tag(
        Some(2),
        "user_total_manga",
        &stats.count.to_string(),
    ));
    header.push_str(&xml_tag(
        Some(2),
        "user_total_reading",
        &(status_reading.count + status_rereading.count).to_string(),
    ));
    header.push_str(&xml_tag(
        Some(2),
        "user_total_completed",
        &status_completed.count.to_string(),
    ));
    header.push_str(&xml_tag(
        Some(2),
        "user_total_onhold",
        &status_onhold.count.to_string(),
    ));
    header.push_str(&xml_tag(
        Some(2),
        "user_total_dropped",
        &status_dropped.count.to_string(),
    ));
    header.push_str(&xml_tag(
        Some(2),
        "user_total_plantoread",
        &status_plantowatch.count.to_string(),
    ));

    header
}

pub fn xml_export_comment(username: &str) -> String {
    let mut buffer = String::new();
    let now = Local::now();

    buffer.push_str("<!--\n");
    buffer.push_str("Export done by ");
    buffer.push_str(env!("CARGO_PKG_NAME"));
    buffer.push_str(" v");
    buffer.push_str(env!("CARGO_PKG_VERSION"));
    buffer.push_str(" by ");
    buffer.push_str(env!("CARGO_PKG_AUTHORS"));
    buffer.push_str(&format!(
        " on {}-{:02}-{:02}",
        now.year(),
        now.month(),
        now.day(),
    ));
    buffer.push_str(" for the AniList account named ");
    buffer.push_str(username);
    buffer.push_str("\n-->");

    buffer
}

pub fn xml_anime(anime_entry: MediaEntry, update: bool) -> String {
    let mut xmlout: String = String::new();
    match anime_entry.media.idMal {
        None => xmlout.push_str("<!--\n"),
        Some(_) => {}
    }
    xmlout.push_str("\t<anime>\n");
    xmlout.push_str(&xml_tag(
        Some(2),
        "series_animedb_id",
        &anime_entry.media.idMal.unwrap_or(0).to_string(),
    ));
    xmlout.push_str(&xml_tag(
        Some(2),
        "series_title",
        &anime_entry.media.title.romaji,
    ));
    xmlout.push_str(&xml_tag(
        Some(2),
        "series_type",
        &anime_entry
            .media
            .format
            .unwrap_or(Format::UNKNOWN)
            .to_string(),
    ));
    xmlout.push_str(&xml_tag(
        Some(2),
        "series_episodes",
        &anime_entry.media.episodes.unwrap_or(0).to_string(),
    ));
    xmlout.push_str(&xml_tag(Some(2), "my_id", "0")); // keep it zero
    xmlout.push_str(&xml_tag(
        Some(2),
        "my_watched_episodes",
        &anime_entry.progress.to_string(),
    ));
    xmlout.push_str(&xml_tag(
        Some(2),
        "my_start_date",
        &anime_entry.startedAt.to_string(),
    ));
    xmlout.push_str(&xml_tag(
        Some(2),
        "my_finish_date",
        &anime_entry.completedAt.to_string(),
    ));
    xmlout.push_str(&xml_tag(Some(2), "my_rated", ""));
    xmlout.push_str(&xml_tag(
        Some(2),
        "my_score",
        &anime_entry.score.to_string(),
    ));
    xmlout.push_str(&xml_tag(Some(2), "my_dvd", ""));
    xmlout.push_str(&xml_tag(Some(2), "my_storage", ""));
    xmlout.push_str(&xml_tag(
        Some(2),
        "my_status",
        &anime_entry.status.to_string(),
    ));
    xmlout.push_str(&xml_tag(
        Some(2),
        "my_comments",
        &string_option_unwrap(anime_entry.notes),
    ));
    xmlout.push_str(&xml_tag(
        Some(2),
        "my_times_watched",
        &(anime_entry.repeat).to_string(),
    ));
    xmlout.push_str(&xml_tag(Some(2), "my_rewatch_value", "")); // MAL only
    xmlout.push_str(&xml_tag(Some(2), "my_priority", ""));
    xmlout.push_str(&xml_tag(
        Some(2),
        "my_tags",
        &lists_to_tags(anime_entry.customLists),
    ));
    if anime_entry.status == Status::REPEATING {
        xmlout.push_str(&xml_tag(Some(2), "my_rewatching", "1"));
        xmlout.push_str(&xml_tag(
            Some(2),
            "my_rewatchin_ep",
            &anime_entry.progress.to_string(),
        ));
    } else {
        xmlout.push_str(&xml_tag(Some(2), "my_rewatching", "0"));
        xmlout.push_str(&xml_tag(Some(2), "my_rewatching_ep", "0"));
    }
    xmlout.push_str(&xml_tag(Some(2), "my_discuss", "1"));
    xmlout.push_str(&xml_tag(Some(2), "my_sns", "default"));
    if update {
        xmlout.push_str(&xml_tag(Some(2), "update_on_import", "1"));
    } else {
        xmlout.push_str(&xml_tag(Some(2), "update_on_import", "0"));
    }
    xmlout.push_str("\t</anime>");
    match anime_entry.media.idMal {
        None => xmlout.push_str("\n-->\n"),
        Some(_) => {}
    }

    xmlout
}

pub fn xml_manga(manga_entry: MediaEntry, update: bool) -> String {
    let mut xmlout: String = String::new();
    match manga_entry.media.idMal {
        None => xmlout.push_str("<!--\n"),
        Some(_) => {}
    }
    xmlout.push_str("\t<manga>\n");
    xmlout.push_str(&xml_tag(
        Some(2),
        "manga_mangadb_id",
        &manga_entry.media.idMal.unwrap_or(0).to_string(),
    ));
    xmlout.push_str(&xml_tag(
        Some(2),
        "manga_title",
        &manga_entry.media.title.romaji,
    ));
    xmlout.push_str(&xml_tag(
        Some(2),
        "manga_volumes",
        &manga_entry.media.volumes.unwrap_or(0).to_string(),
    ));
    xmlout.push_str(&xml_tag(
        Some(2),
        "manga_chapters",
        &manga_entry.media.chapters.unwrap_or(0).to_string(),
    ));
    xmlout.push_str(&xml_tag(Some(2), "my_id", "0")); // keep it zero
    xmlout.push_str(&xml_tag(
        Some(2),
        "my_read_volumes",
        &manga_entry.progressVolumes.unwrap_or(0).to_string(),
    ));
    xmlout.push_str(&xml_tag(
        Some(2),
        "my_read_chapters",
        &manga_entry.progress.to_string(),
    ));
    xmlout.push_str(&xml_tag(
        Some(2),
        "my_start_date",
        &manga_entry.startedAt.to_string(),
    ));
    xmlout.push_str(&xml_tag(
        Some(2),
        "my_finish_date",
        &manga_entry.completedAt.to_string(),
    ));
    xmlout.push_str(&xml_tag(Some(2), "my_scanalation_group", ""));
    xmlout.push_str(&xml_tag(
        Some(2),
        "my_score",
        &manga_entry.score.to_string(),
    ));
    xmlout.push_str(&xml_tag(Some(2), "my_storage", ""));
    xmlout.push_str(&xml_tag(Some(2), "my_retail_volumes", "0"));
    xmlout.push_str(&xml_tag(
        Some(2),
        "my_status",
        &to_string_manga(manga_entry.status),
    ));
    xmlout.push_str(&xml_tag(
        Some(2),
        "my_comments",
        &string_option_unwrap(manga_entry.notes),
    ));
    xmlout.push_str(&xml_tag(
        Some(2),
        "my_times_read",
        &(manga_entry.repeat).to_string(),
    ));
    xmlout.push_str(&xml_tag(
        Some(2),
        "my_tags",
        &lists_to_tags(manga_entry.customLists),
    ));
    xmlout.push_str(&xml_tag(Some(2), "my_priority", ""));
    xmlout.push_str(&xml_tag(Some(2), "my_reread_value", "")); // MAL only
    if manga_entry.status == Status::REPEATING {
        xmlout.push_str(&xml_tag(Some(2), "my_rereading", "YES"));
    } else {
        xmlout.push_str(&xml_tag(Some(2), "my_rereading", "NO"));
    }
    xmlout.push_str(&xml_tag(Some(2), "my_discuss", "YES"));
    xmlout.push_str(&xml_tag(Some(2), "my_sns", "default"));
    if update {
        xmlout.push_str(&xml_tag(Some(2), "update_on_import", "1"));
    } else {
        xmlout.push_str(&xml_tag(Some(2), "update_on_import", "0"));
    }
    xmlout.push_str("\t</manga>");
    match manga_entry.media.idMal {
        None => xmlout.push_str("\n-->\n"),
        Some(_) => {}
    }

    xmlout
}
