extern crate curl;
extern crate regex;
extern crate url;
extern crate clap;

use curl::easy::Easy;
use std::fs::File;
use std::io::{self, BufRead, Read, Result};
use std::process;
use std::string::ToString;
use regex::Regex;
use url::Url;
use clap::{App, Arg, ArgMatches};

fn main() {
    if run() == BrokenLinks::Found {
        process::exit(1);
    }
}

#[derive(PartialEq)]
enum BrokenLinks { None, Found }

fn run() -> BrokenLinks {
    let args = parse_args();
    let mut input = String::new();
    let target = args.value_of("INPUT");
    let stdin = io::stdin();
    let mut handle = Easy::new();
    handle.follow_location(true).unwrap();
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
    let result = test_links(
        args.is_present("recursive"),
        args.is_present("verbose"),
        &location.map(ToString::to_string),
        lines,
        &mut handle);
    result
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
                  mut handle: &mut Easy) -> BrokenLinks {
    let url_regex = Regex::new("https?://\\w+.\\w+[^\\s'\"]+").unwrap();
    let mut line_num = 0;
    let mut result = BrokenLinks::None;
    for line in lines {
        line_num += 1;
        for cap in url_regex.captures_iter(&line) {
            let url = cap.at(0).unwrap();
            if verbose {
                println!("Testing {}", url);
            }
            match url_error(&mut handle, url) {
                Some(ref error) => {
                    println!("{}{} {} {}", location.clone().map_or("".to_string(), |l| l + ":"),
                                           line_num, url, error);
                    result = BrokenLinks::Found;
                },
                None if recursive => {
                    let parent_url = Url::parse(&location.clone().unwrap()).unwrap();
                    let current_url = Url::parse(url.clone()).unwrap();
                    let parent_host = parent_url.host();
                    let url_host = current_url.host();
                    if parent_host == url_host {
                        let url_body = url_body(&mut handle, &url);
                        if BrokenLinks::Found == test_links(recursive,
                                    verbose,
                                    &location.clone().map(|l| l + "->" + url),
                                    Box::new(url_body.lines().map(ToString::to_string))
                                        as Box<Iterator<Item=String>>,
                                        handle) {
                            result = BrokenLinks::Found;
                        }
                    }
                },
                _ => ()
            }
        }
    }
    return result;
}

fn url_body(handle: &mut Easy, url: &str) -> String {
    handle.url(url).unwrap();

    let mut result = String::new();
    // block is needed to drop `transfer` together with closure
    // allowing it to modify `result` without outliving the variable
    {
        let mut transfer = handle.transfer();
        transfer.write_function(|body| {
            result = String::from_utf8_lossy(body).into_owned();
            Ok(body.len())
        }).unwrap();
        transfer.perform().unwrap();
    }
    result
}

fn url_error(handle: &mut Easy, url: &str) -> Option<String> {
    handle.url(url.trim()).unwrap();
    handle.nobody(true).unwrap();
    handle.perform().err().map(|err| err.to_string()).or_else(|| {
        match handle.response_code() {
                Ok(status) if !success(status) => Some(status.to_string()),
                Err(err) => Some(err.to_string()),
                _ => None
            }
    })
}

fn success(status: u32) -> bool {
    status >= 200 && status < 300
}
