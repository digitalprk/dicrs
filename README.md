# dic.rs
A simple dictionary application written in Rust

<img src="http://digitalnk.com/blog/wp-content/uploads/2020/05/2020-05-09_15-00.png" width="70%" height="70%" />

<hr>

### Overview

Dic.rs is a simple TUI developed in Rust with [tui-rs](https://github.com/fdehau/tui-rs) to browse dictionaries obtained as part of [this project](https://digitalnk.com/blog/2020/05/08/porting-north-korean-dictionaries-with-rust/). Dictionaries are stored as SQLite databases in the `dics/` folder. See below, [Dictionaries](#dictionaries) to download dictionaries.

### Usage

Clone the repository, create a `dics/` folder inside of it. Copy [dictionary files](#dictionaries) into the `dics/` folder, then run

`cargo run dicrs`

The application will only run on GNU/Linux. The application will panic if no dictionaries are available under `dics/`.

### Dictionaries

Dictionaries are SQLite database files with the .db extension. Dictionaries contain two tables
1. A `name` table with a single `dicname` text column whose first entry contains the name of the dictionary (currently not used)
2. A `dictionary` table with two text columns `word` and `definition`. The `word` column contains words from which the index will be built. The `definition` columns contains the corresponding definition(s).

The dictionaries are stored in the `dics/` folder. The name of the dictionary's file will be the name displayed in the app. To rename or remove a dictionary, simply rename or remove the corresponding file.

Dictionaries available for the project:
* [The GNU Collaborative International Dictionary of English](https://digitalnk.com/dics/GCIDE.db), available under the terms of the [GNU General Public License](http://www.gnu.org/licenses/gpl-3.0.html)
* [North Korean monolingual reference dictionary](https://digitalnk.com/dics/KK.db)
* [North Korean technoscientific dictionary (비약과학기술용어사전)](https://digitalnk.com/dics/biyak.db)
* [North Korean - English dictionary (영조사전)](https://digitalnk.com/dics/KEEK.db)
* [North Korean - Chinese dictionary (중조사전)](https://digitalnk.com/dics/KCCK.db)
* [North Korean - French dictionary (불조사전)](https://digitalnk.com/dics/KFFK.db)
* [North Korean - Japanese dictionary (일조사전)](https://digitalnk.com/dics/KJJK.db)
* [North Korean - German dictionary (독조사전)](https://digitalnk.com/dics/KDDK.db)
* [North Korean - Russian dictionary (로조사전)](https://digitalnk.com/dics/KRRK.db)

### Controls

Shortcut | Action
-------- | -------
`Ctrl + C` | Close the app
`Ctrl + Y` | Copy a definition to the clipboard
`Up / Down` | Move one word up/down in the index
`PgUp / PgDn` | Move ten words up/down in the index
`Left / Right` | Move one dictionary up or down

Mouse control (scrollwheel, click) is available to browse through the index

### Search

Search is case-insensitive. The SQL wildcard `%` is applied by default at the end of every word searched.

No filtering is applied to the text input, so any SQL wildcard (and injection) can currently be used directly from the dictionary's search box. In case of multiple results, only the first result will be used.
