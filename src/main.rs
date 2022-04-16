extern crate json;

use std::env;
use std::fs;
use std::ops::Neg;

use anyhow::Result;
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use itertools::Itertools;
use json::JsonValue;
use powerpack::Item;

#[derive(Debug, Clone, Eq, PartialEq)]
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

    pub fn calculate_matching_score(&self, query: String) -> i64 {
        let matcher = SkimMatcherV2::default();
        return matcher
            .fuzzy_match(&self.name[..], &query[..])
            .get_or_insert(0)
            .to_owned()
            .neg();
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

fn sort_and_filter_matching_bookmarks(bookmarks: Vec<Bookmark>, query: String) -> Vec<Bookmark> {
    return bookmarks
        .iter()
        .sorted_by_key(|bookmark| bookmark.calculate_matching_score(query.to_owned()))
        .filter(|bookmark| bookmark.calculate_matching_score(query.to_owned()) < 0)
        .map(|bookmark| bookmark.to_owned())
        .collect();
}

fn to_items(bookmarks: Vec<Bookmark>, query: String, default_search_url: String) -> Vec<Item> {
    let matched_bookmarks: Vec<Item> = sort_and_filter_matching_bookmarks(bookmarks, query.clone())
        .iter()
        .map(|bookmark| bookmark.to_item())
        .collect();
    return if matched_bookmarks.is_empty() {
        vec![default(query, default_search_url)]
    } else {
        matched_bookmarks
    };
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
    use powerpack::Item;

    use crate::{sort_and_filter_matching_bookmarks, Bookmark};

    #[test]
    fn does_not_matches_the_query() {
        let bookmark = Bookmark {
            name: String::from("Dashboard"),
            link: String::from("http://www.test.blub"),
        };

        let score = bookmark.calculate_matching_score("z".to_string());

        assert_eq!(score, 0);
    }

    #[test]
    fn matches_the_query() {
        let bookmark = Bookmark {
            name: String::from("Dashboard"),
            link: String::from("http://www.test.blub"),
        };

        let score = bookmark.calculate_matching_score("d".to_string());

        assert_eq!(score, -29);
    }

    #[test]
    fn transforms_to_item() {
        let bookmark = Bookmark {
            name: String::from("Dashboard"),
            link: String::from("http://www.test.blub"),
        };
        let expected_item = Item::new("Dashboard")
            .subtitle("Open in browser →")
            .arg("http://www.test.blub");

        let item = bookmark.to_item();

        assert_eq!(item, expected_item);
    }

    #[test]
    fn sorts_and_keep_matchting_bookmarks() {
        let bookmark1 = Bookmark {
            name: String::from("Dashboard"),
            link: String::from("http://www.test.blub"),
        };
        let bookmark2 = Bookmark {
            name: String::from("Bookmarks"),
            link: String::from("http://www.bookmarks.blub"),
        };
        let bookmarks = vec![bookmark1.clone(), bookmark2.clone()];
        let expected_bookmarks = vec![bookmark1.clone(), bookmark2.clone()];

        let matching_bookmarks = sort_and_filter_matching_bookmarks(bookmarks, "o".to_owned());

        assert_eq!(matching_bookmarks, expected_bookmarks);
    }

    #[test]
    fn removes_not_matchting_bookmarks() {
        let bookmark1 = Bookmark {
            name: String::from("Dashboard"),
            link: String::from("http://www.test.blub"),
        };
        let bookmark2 = Bookmark {
            name: String::from("Bookmarks"),
            link: String::from("http://www.bookmarks.blub"),
        };
        let bookmarks = vec![bookmark1.clone(), bookmark2.clone()];
        let expected_bookmarks = vec![bookmark1.clone()];

        let matching_bookmarks = sort_and_filter_matching_bookmarks(bookmarks, "d".to_owned());

        assert_eq!(matching_bookmarks, expected_bookmarks);
    }
}
