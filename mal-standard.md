# Informal attempt to define what the "MAL standard" is

Between the various anime list sites, the standard for exports that has emerged is the one in use by MAL. However, they do not officially document how their standard works, and neither is there a publicly available unofficial documentation. So for the purposes of this project, this document is an unofficial attempt at defining what the "MAL standard" consists of.

## Briefly on the XML

Firstly, [the export option on MAL](https://myanimelist.net/panel.php?go=export) generates an XML document, version 1.0 with UTF-8 encoding. As such, exports must be well formatted XML documents compliant with the W3C's standard for XML 1.0 with properly escaped special characters such as the case for the ampersand. MAL uses CDATA to escape text in its fields, however both MAL and AniDB are fine with escaping special characters normally, like *\&amp;*.

The document's head is a \<myanimelist\> tag, in which there is a \<myinfo\> tag and a list of either \<anime\> or \<manga\> tags depending on the export's type. There is difference between \<manga\> and \<anime\> entries, but it isn't that large.

## Tag structure

### Date format

The dates used in the document are formatted to follow YYYY-MM-DD. In case a date is *null* or isn't known, 0000-00-00 is used.

## anime list

### The myinfo tag

The \<myinfo\> tag is used for storing general statistics and information about the user, it is unclear if sites utilize the information provided by this tag. However, information contained within might still be useful for other 3rd party programs. Its structure is as follows:

- **user\_id**: the user's id on MAL
- **user\_name**: the user's name on MAL
- **user\_export\_type**: left at 1 for anime list exports and 2 for manga lists
- **user\_total\_anime**: the total number of anime on a user's list, includes all entries
- **user\_total\_watching**: the number of anime the user is currently watching
- **user\_total\_completed**: the number of anime the user has already completed
- **user\_total\_onhold**: the number of anime the user has on-hold/paused
- **user\_total\_dropped**: the number of anime the user has dropped
- **user\_total\_plantowatch**: the number of anime the user is planning on watching

### the anime tag

The \<anime\> tag is an entry on the anime list with general and personal information. The structure is as follows: 

- **series\_animedb\_id**: the entry's ID on MAL
- **series\_title**: the entry's title in romaji
- **series\_type**: the type of the series in capital letters
- **series\_episodes**: the number of episodes of an entry, it's 1 for movies
- **my\_id**: left at 0, "a sequential global number that's given to each anime entry when you add it to your list" [source](https://myanimelist.net/forum/?goto=post&topicid=267660&id=9784885)
- **my\_watched\_episodes**: the number of episodes watched
- **my\_start\_date**: date the entry's first watch was originally started
- **my\_finish\_date**: date the entry's first watch was originally finished
- **my\_rated**: purpose unknown, unused, left empty
- **my\_score**: 1-10 whole number. The number can be left a two-digit integrer, but both MAL and aniDB will round integers to whole numbers (even though the latter has support for halves)
- **my\_storage**: left empty, type of storage used to store the entry
- **my\_storage\_value**: left empty, drive space in GB or the number of disks
- **my\_status**: status of the entry. Value is one of the following: Watching, Plan to Watch, Completed, Dropped, On-Hold
- **my\_comments**: Comments on the entry
- **my\_times\_watched**: number of times re-watched 
- **my\_rewatch\_value**: left empty, values range from "Very Low" to "Very High"
- **my\_priority**: left empty
- **my\_tags**: tags separated by commas, can include spaces
- **my\_rewatching**: 1 if entry is being rewatched, not set or 0 otherwise 
- **my\_rewatching\_ep**: supposedly the episode in the rewatch, MAL leaves it at 0 and is unused. See [here](https://myanimelist.net/forum/?goto=post&topicid=294806&id=10649513)
- **my\_discuss**: left at "default"
- **my\_sns**: left at "default"
- **update\_on\_import**: whether or not to update the entry on the import, set to 1

And to clarify once again, if an entry is being rewatched, \<my\_rewatching\> is set to 1, \<my\_status\> is left at Completed, and \<my\_watched\_episodes\> is used to keep track of the progress. Once rewatched, \<my\_times\_watched\> is incremeneted by one. Even though \<my\_rewatching\_ep\> is not used by MAL, it is still set in this case.

## manga list

### The myinfo tag

The \<myinfo\> tag is really similar to that of anime lists, with a few terms changed. Also do note that for whatever reason AniList's API request does not display reading statuses for this query.

- **user\_id**: the user's id on MAL
- **user\_name**: the user's name on MAL
- **user\_export\_type**: 1 for anime list exports and 2 for manga lists
- **user\_total\_manga**: the total number of manga on a user's list, includes all entries
- **user\_total\_reading**: the number of manga the user is currently reading (see limitations above)
- **user\_total\_completed**: the number of manga the user has already completed
- **user\_total\_onhold**: the number of manga the user has on-hold/paused
- **user\_total\_dropped**: the number of manga the user has dropped
- **user\_total\_plantoread**: the number of manga the user is planning on reading

### the manga tag

The \<manga\> tag is also similiar to its counterpart, but with the order and tags slightly altered.

- **manga\_mangadb\_id**: the entry's ID on MAL
- **manga\_title**: the entry's title in romaji
- **manga\_volumes**: the number of volumes of an entry
- **manga\_chapters**: the number of chapters of an entry
- **my\_id**: left at 0, "a sequential global number that's given to each anime entry when you add it to your list" [source](https://myanimelist.net/forum/?goto=post&topicid=267660&id=9784885)
- **my\_read\_volumes**: the number of volumes read
- **my\_read\_volumes**: the number of chapters read
- **my\_start\_date**: date the entry's first watch was originally started
- **my\_finish\_date**: date the entry's first watch was originally finished
- **my\_scanslation_group**: MAL always leaves it as an empty tag, might be legacy. Left empty
- **my\_score**: 1-10 whole number. The number can be left a two-digit integrer, but both MAL and aniDB will round integers to whole numbers (even though the latter has support for halves)
- **my\_storage**: left empty, type of storage used to store the entry
- **my\_retail\_volumes**: left empty, in case the previous option was "Retail Manga", it is the number of volumes owned
- **my\_status**: status of the entry. Value is one of the following: Reading, Plan to Read, Completed, Dropped, On-Hold
- **my\_comments**: Comments on the entry
- **my\_times\_read**: number of times re-read
- **my\_tags**: tags separated by commas, can include spaces
- **my\_priority**: left empty
- **my\_reread\_value**: left empty, values range from "Very Low" to "Very High"
- **my\_rereading**: "YES" if entry is being re-read, "NO" if it isn't
- **my\_discuss**: left at "YES"
- **my\_sns**: left at "default"
- **update\_on\_import**: whether or not to update the entry on the import, set to 1

Re-reading a manga entry works the same way but without an unused tag.
