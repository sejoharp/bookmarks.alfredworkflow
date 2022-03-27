extern crate json;

use std::env;
use std::fs;

use anyhow::Result;

use json::JsonValue;
use powerpack::Item;

#[derive(Debug, Clone)]
pub struct Bookmark {
    name: String,
    link: String,
}

impl Bookmark {
    pub fn from_json_value(value: &JsonValue) -> Bookmark {
        let name = value["title"].as_str().unwrap().to_owned();
        let link = value["href"].as_str().unwrap().to_owned();
        Bookmark {
            name: name,
            link: link,
        }
    }

    pub fn to_item(&self) -> Item {
        Item::new(self.name.to_string())
            .subtitle("Open in browser →")
            .arg(self.link.to_owned())
    }

    pub fn find(&self, query: String) -> bool {
        return self.name.to_lowercase().contains(query.as_str());
    }
}

pub fn read_bookmarks(json: String) -> Vec<Bookmark> {
    let parsed = json::parse(&json).unwrap();
    let json_arrays = parsed
        .entries()
        .map(|entry| entry.1)
        .collect::<Vec<&JsonValue>>();

    return json_arrays
        .into_iter()
        .map(|entry| {
            entry
                .members()
                .map(|entry| Bookmark::from_json_value(entry))
        })
        .flatten()
        .collect();
}

/// Returns an Alfred item for when no query has been typed yet.
fn empty(default_search_url: String) -> Item {
    Item::new("Search for bookmarks")
        .subtitle("Open them →")
        .arg(default_search_url)
}

/// Returns an Alfred item for when the query doesn't match any crates.
fn default(query: String, default_search_url: String) -> Item {
    Item::new(format!(
        "nothing found for {}, try search on website",
        query
    ))
    .subtitle("Open them →")
    .arg(default_search_url)
}

fn find_matching_bookmarks(bookmarks: Vec<Bookmark>, query: String) -> Vec<Bookmark> {
    bookmarks
        .into_iter()
        .filter(|bookmark| bookmark.find(query.clone()))
        .collect()
}

fn to_items(bookmarks: Vec<Bookmark>, query: String, default_search_url: String) -> Vec<Item> {
    let matched_bookmarks: Vec<Item> = find_matching_bookmarks(bookmarks, query.clone())
        .iter()
        .take(10)
        .map(|bookmark| bookmark.to_item())
        .collect();
    if matched_bookmarks.is_empty() {
        vec![default(query, default_search_url)]
    } else {
        matched_bookmarks
    }
}

fn main() -> Result<()> {
    let bookmarks_file = env::var("BOOKMARKS_FILE").expect("BOOKMARKS_FILE not set");
    let default_search_url = env::var("DEFAULT_SEARCH_URL").expect("DEFAULT_SEARCH_URL not set");

    let contents =
        fs::read_to_string(bookmarks_file).expect("Something went wrong reading the file");
    let bookmarks = read_bookmarks(contents);
    let arg = env::args()
        .nth(1)
        .as_deref()
        .map(str::trim)
        .map(str::to_ascii_lowercase);

    let items: Vec<Item> = match arg.as_deref() {
        None | Some("") => vec![empty(default_search_url)],
        Some(query) => to_items(bookmarks, String::from(query), default_search_url),
    };
    powerpack::output(items)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::Bookmark;

    #[test]
    fn finds_bookmark() {
        let bookmark = Bookmark {
            name: String::from("Dashboard"),
            link: String::from("http://www.test.blub"),
        };
        assert_eq!(bookmark.find(String::from("dash")), true);
    }
}
