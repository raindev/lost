extern crate curl;

use curl::http;
use curl::http::{Handle, Response};
use std::io::{self, BufRead};

fn main() {
    let stdin = io::stdin();
    let mut handle = http::handle();
    let mut line_num = 0;
    for line in stdin.lock().lines() {
        line_num += 1;
        let line = line.unwrap();
        if line.is_empty() {
            continue;
        }
        url_error(&mut handle, &line)
            .map(|error| println!("{} {} {}", line_num, line, error));
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
