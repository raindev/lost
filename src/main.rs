extern crate curl;
extern crate regex;
extern crate url;
extern crate clap;

use curl::http;
use curl::http::{Handle, Response};
use std::fs::File;
use std::io::{self, BufRead, Read, Result};
use std::string::ToString;
use regex::Regex;
use url::Url;
use clap::{App, Arg, ArgMatches};

fn main() {
    let args = parse_args();
    let mut input = String::new();
    let target = args.value_of("INPUT");
    let stdin = io::stdin();
    let mut handle = http::handle();
    let mut location = None;
    let lines =
    if target.is_some() {
        let target = target.unwrap();
        if target.starts_with("http") {
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
    test_links(
        args.is_present("recursive"),
        args.is_present("verbose"),
        &location.map(ToString::to_string),
        lines,
        &mut handle);
}

fn parse_args<'a>() -> ArgMatches<'a> {
    App::new("lost")
        .version(env!("CARGO_PKG_VERSION"))
        .author("Andrew \"raindev\" Barchuk <andrew@raindev.io>")
        .about("Search for broken links")
        .arg(Arg::with_name("verbose")
             .short("v")
             .long("verbose")
             .help("Verbose mode"))
        .arg(Arg::with_name("recursive")
             .short("R")
             .long("recursive")
             .help("Traverse URLs on the same domain recursively. Works for web pages only"))
        .arg(Arg::with_name("INPUT")
             .help(concat!("Source of input. Could be file or URL address",
                   "{n}(needs explicit http[s] prefix). stdin is assumed if omitted.")))
        .get_matches()
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
                Some(ref error) => println!("{}{} {} {}",
                                            location.clone().map_or("".to_string(), |l| l + ":"),
                                            line_num, url, error),
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
                                    Box::new(url_body.lines().map(ToString::to_string))
                                        as Box<Iterator<Item=String>>,
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
