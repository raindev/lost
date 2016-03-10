extern crate curl;
extern crate regex;

use curl::http;
use curl::http::{Handle, Response};
use std::fs::File;
use std::env;
use std::io::{self, BufRead, Read, Result};
use std::string::ToString;
use regex::Regex;

fn main() {
    let mut input = String::new();
    let mut args = env::args();
    let stdin = io::stdin();
    let mut handle = http::handle();
    let lines =
    if args.len() > 1 {
        let target = args.nth(1).unwrap();
        if target.starts_with("http") {
            input = url_body(&mut handle, &target);
            Box::new(input.lines().map(ToString::to_string)) as Box<Iterator<Item=String>>
        } else {
            File::open(target).unwrap().read_to_string(&mut input).unwrap();
            Box::new(input.lines().map(ToString::to_string)) as Box<Iterator<Item=String>>
        }
    } else {
        Box::new(stdin.lock().lines().map(Result::unwrap)) as Box<Iterator<Item=String>>
    };
    let mut line_num = 0;
    let url_regex = Regex::new("https?://\\w+.\\w+[^\\s'\"]+").unwrap();
    for line in lines {
        line_num += 1;
        for cap in url_regex.captures_iter(&line) {
            let url = cap.at(0).unwrap();
            url_error(&mut handle, url)
                .map(|error| println!("{} {} {}", line_num, url, error));
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
