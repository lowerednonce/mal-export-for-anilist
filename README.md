# MAL exporter for AniList accounts

This package is a MAL syle anime list exporter for AniList account written in Rust that uses the official GraphQL API. There is no need for oauth or any other type of authentication for the user when using it, just the username on AniList. 

The reason why this was built instead of using one of the other exporters is twofold:
- Using the official API instead of web-scraping means it is faster, more stable, and provides more complete data.
- Other tools ([1](https://anilist.co/forum/thread/4381) [2](https://malscraper.azurewebsites.net/) [3](https://github.com/nathanwentworth/anilist-to-mal)) are old and/or do not make correct exports and/or are discontinued.

The package was built using the latest Rust version at the time of writing (rustc && cargo v1.77, Rust edition 2021), but cargo should automatically configure the dependencies as needed. There are four primary dependencies that the package uses: *tokio* for asynchronicity, *reqwests* for making the API requests, *clap* for handling comand line arguments, and *serde* & *serde\_json* for creating, reading, and parsing JSON data.

## Usage

The package is a CLI tool, meaning it needs to be used from the command line. Its usage is automatically documented by clap, invoke **--help** or **-h** on the program for the up-to-date details. But in general the usage is the following:

```bash
mal-export-for-anilist --user <username> --file <output-file>
```

If the supplied file already exists, it will be automatically overwritten. Re-exporting will only overwrite the old export and will not merge new information. The filename can also be a file path as long as it is compliant with the standards of your OS. 

## Generated document

After the program finishes running and exits correctly without any errors, there will be an XML file with the name supplied which will contain the export. After that the file is read to be imported on [MAL](https://myanimelist.net/import.php) or [AniDB](https://anidb.net/user/import/).

In its current state, the only export the anime list. Custom lists are transcoded as tags. For more information on the output see [mal-standard.md](mal-standard.md).

## Errors

The only error that should be able to occur is going over the AniList API rate-limit. That kicks in when you make too many requests. The package makes 1 query per 50 entries in your list. People with more than 4500 entries in their lists will likely face currently unresolved issues when exporting. 

Any other case in which the program panics and exits incorrectly, a bug report should be filed so the issue can be fixed.
