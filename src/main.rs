extern crate curl;
extern crate regex;

use curl::http;
use curl::http::{Handle, Response};
use std::io::{self, BufRead};
use regex::Regex;

fn main() {
    let stdin = io::stdin();
    let mut handle = http::handle();
    let mut line_num = 0;
    let url_regex = Regex::new(r"https?://\w+.\w+[^\s]+").unwrap();
    for line in stdin.lock().lines() {
        line_num += 1;
        let line = line.unwrap();
        for cap in url_regex.captures_iter(&line) {
            let url = cap.at(0).unwrap();
            url_error(&mut handle, url)
                .map(|error| println!("{} {} {}", line_num, url, error));
        }
    }
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
