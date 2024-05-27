[![CC BY-SA 4.0][cc-by-sa-shield]][cc-by-sa]
[![Latest Release][release-shield]][build]
[![Build Status][build-shield]][build]
![Formatting Status][format-shield]

[cc-by-sa]: http://creativecommons.org/licenses/by-sa/4.0/
[cc-by-sa-shield]: https://img.shields.io/badge/License-CC%20BY--SA%204.0-lightgrey.svg
[build-shield]: https://img.shields.io/github/actions/workflow/status/lowerednonce/mal-export-for-anilist/.github%2Fworkflows%2Frelease.yml?label=release%20build
[build]: https://github.com/lowerednonce/mal-export-for-anilist/releases/latest
[format-shield]: https://img.shields.io/github/actions/workflow/status/lowerednonce/mal-export-for-anilist/.github%2Fworkflows%2Fformat.yml?label=formatting
[release-shield]: https://img.shields.io/github/v/release/lowerednonce/mal-export-for-anilist

# MAL exporter for AniList accounts

This package is a MAL syle anime list exporter for AniList accounts written in Rust that uses the official GraphQL API. In principle, there is no need for oauth or any other type of special setup when using it, just the username on AniList. The exporter includes every feature one can reasonably expect, see the features section.

The reason why this was built instead of using one of the other exporters is twofold:
- Using the official API instead of web-scraping means that this is faster, more stable, and provides more complete data.
- Other tools ([1](https://anilist.co/forum/thread/4381) [2](https://malscraper.azurewebsites.net/) [3](https://github.com/nathanwentworth/anilist-to-mal)) are old and/or do not make correct exports and/or are discontinued.

The package was built using the latest Rust version at the time of writing (rustc && cargo v1.77, Rust edition 2021), but cargo should automatically configure the dependencies as needed. There are four primary dependencies that the package uses: *tokio* for asynchronicity, *reqwests* for making the API requests, *clap* for handling comand line arguments, and *serde* & *serde\_json* for creating, reading, and parsing JSON data.

## Features

As far as the API and the export standard are concerned, this is a full export tool. This means that everything that can fit into a MAL export is exported (including custom lists), entries which don't appear in MAL's database are left commented out. Other features include:

- Export of either anime or manga lists
- Export of private entries
- Hiding of adult entries
- Generating a merge-ready list
- Generating general statistics

## Usage

The package is a CLI tool, meaning it needs to be used from the command line. Its usage is automatically documented by clap, invoke **--help** or **-h** on the program for up-to-date details. The mandatory arguments look like the following:

```bash
mal-export-for-anilist --user <username> --file <output-file> --list <anime|manga>
```

If the supplied file already exists, it will be automatically overwritten. Re-exporting will only overwrite the old export and will not merge new information. The filename can also be a file path as long as it is compliant with the standards of your OS.

## Flags

For the purposes of merging lists between accounts, it is recommended to use `-n` or `--no-update`, which disables *update_on_import* being automatically set to 1. This means that when importing the list, only entries which aren't in the preexisting list are updated.

In order to export private entries, the program needs to be authorized. To do that, set the `--oauth` flag. This will prompt the program to display an authentication URL. Visit it, authorize the bot, copy and paste the text displayed (it will be relatively large and unreadable). With the way this is set up, this doesn't grant any persistent permission.

Adult entries are by default exported. If this is an undesired outcome (for example, when using the `--oauth` flag), set the `--no-nsfw` flag to disable their export.


## Generated document

After the program finishes running and exits correctly without any errors, there will be an XML file with the name supplied which will contain the export. After that the file is read to be imported on [MAL](https://myanimelist.net/import.php) or [AniDB](https://anidb.net/user/import/).

Custom lists are transcoded as tags. For more information on the output see [mal-standard.md](mal-standard.md).

## Building

A standard cargo configuration is used in this package, clone the repository and run

```sh
cargo build --release
```

After cargo is done compiling the package, the executable can be found in the `target/release/` folder under the same name as the repository. Alternatively, you can also use the shortcut

```sh
cargo run --release -- [args]
```

## Errors

The only expected error has to do with OAuth. In case you input the authorization token badly, AniList can't accept it, hence causing a failure in making the queries.

Any other type of error should never occur unless you are spamming the command/using automation. That would be caused by the AniList API rate-limit. That kicks in when you make too many requests. However, the package will only make 2 queries (a *User* and a *MediaListCollection*) even for lists reaching into the thousands in terms of entries, so an average user should never encounter any error.

Any other case in which the program panics and exits incorrectly, a bug report should be filed so that the issue can be fixed.
