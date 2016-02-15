extern crate curl;

use curl::http;
use std::io::{self, BufRead};

fn main() {
    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let line = line.unwrap();
        if line.is_empty() {
            return;
        }
        test_url(&line);
    }
}

fn test_url(url: &str) {
    let res = http::handle()
        .head(url.trim())
        .exec();
    match res {
        Ok(response) => println!("Status code: {}", response.get_code()),
        Err(err) => println!("Error: {}", err),
    }
}
