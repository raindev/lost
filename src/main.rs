extern crate curl;
extern crate regex;
extern crate url;

use curl::http;
use curl::http::{Handle, Response};
use std::fs::File;
use std::env;
use std::io::{self, BufRead, Read, Result};
use std::string::ToString;
use regex::Regex;
use url::Url;

fn main() {
    let mut input = String::new();
    let mut recursive = false;
    let mut verbose = false;
    let mut target = None;
    for arg in env::args() {
        if arg == "-R" {
            recursive = true
        } else if arg == "-v" {
            verbose = true;
        } else {
            target = Some(arg);
        }
    }
    let stdin = io::stdin();
    let mut handle = http::handle();
    let mut location = None;
    let lines =
    if target.is_some() {
        let target = target.unwrap();
        if recursive || target.starts_with("http") {
            input = url_body(&mut handle, &target);
            location = Some(target);
            Box::new(input.lines().map(ToString::to_string)) as Box<Iterator<Item=String>>
        } else {
            File::open(target).unwrap().read_to_string(&mut input).unwrap();
            Box::new(input.lines().map(ToString::to_string)) as Box<Iterator<Item=String>>
        }
    } else {
        Box::new(stdin.lock().lines().map(Result::unwrap)) as Box<Iterator<Item=String>>
    };
    test_links(recursive, verbose, &location, lines, &mut handle);
}

fn test_links<'a>(recursive: bool,
                  verbose: bool,
                  location: &Option<String>,
                  lines: Box<Iterator<Item=String> + 'a>,
                  mut handle: &mut Handle) {
    let mut line_num = 0;
    let url_regex = Regex::new("https?://\\w+.\\w+[^\\s'\"]+").unwrap();
    for line in lines {
        line_num += 1;
        for cap in url_regex.captures_iter(&line) {
            let url = cap.at(0).unwrap();
            if verbose {
                println!("Testing {}", url);
            }
            match url_error(&mut handle, url) {
                Some(ref error) => println!("{}{} {} {}", location.clone().map_or("".to_string(), |l| l + ":"), line_num, url, error),
                None if recursive => {
                    let parent_url = Url::parse(&location.clone().unwrap()).unwrap();
                    let current_url = Url::parse(url.clone()).unwrap();
                    let parent_host = parent_url.host();
                    let url_host = current_url.host();
                    if parent_host == url_host {
                        let url_body = url_body(&mut handle, &url);
                        test_links( recursive,
                                    verbose,
                                    &location.clone().map(|l| l + "->" + url),
                                    Box::new(url_body.lines().map(ToString::to_string)) as Box<Iterator<Item=String>>,
                                    handle);
                    }
                },
                _ => ()
            }
        }
    }
}

fn url_body(handle: &mut Handle, url: &str) -> String {
    String::from_utf8_lossy(handle
                            .get(url)
                            .follow_redirects(true)
                            .exec()
                            .unwrap()
                            .get_body()).into_owned()
}

fn url_error(handle: &mut Handle, url: &str) -> Option<String> {
    let result = handle
        .head(url.trim())
        .follow_redirects(true)
        .exec();
    match result {
        Ok(ref resp) if !success(&resp) => Some(resp.get_code().to_string()),
        Err(err) => Some(err.to_string()),
        _ => None
    }
}

fn success(response: &Response) -> bool {
    response.get_code() >= 200 && response.get_code() < 300
}
