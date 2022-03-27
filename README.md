# About
Alfred workflow to search for bookmarks and open them.


## Installation
### source for bookmarks
Right now it reads a json file in the following format:
```json
   {
	"category1": [{
			"href": "https://interesting.domain.blub",
			"title": "intersting domain"
		},
		{
			"href": "https://fantastic.domain.bla",
			"title": "fantastic domain"
		}
	],
	"category2": [{
		"href": "http://test.foo",
		"title": "test url"
	}, ]
}
```
### Building from source
1. install Rust and Cargo using [rustup](https://rustup.rs/). 
2. clone
3. install in alfred: `make install`
4. set environment variables according to your setup:
   1. `DEFAULT_SEARCH_URL`: link to the website to open, when nothing matched.
   2. `BOOKMARKS_FILE`: Path to json file in the following format:

## Debugging issues

To see all output from the workflow you can run the following open the workflwo in debug mode.

## Credits
* This repo used https://github.com/rossmacarthur/crates.alfredworkflow/ as a template. 
* To generate the binary: https://github.com/rossmacarthur/powerpack