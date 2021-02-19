#[global_allocator]
static GLOBAL: jemallocator::Jemalloc = jemallocator::Jemalloc;

use anyhow::Result;
use crossbeam_channel as channel;
use itertools::{concat, Itertools};
use serde::Deserialize;
use std::fs;
use std::thread;
use std::time::Duration;
use threadpool::ThreadPool;

fn main() -> Result<()> {
    let saved_path = "/home/sachin/.config/polybar/current_post.txt";
    let (tx, rx) = channel::unbounded();
    let pool = ThreadPool::new(4);

    let mut subreddits = Vec::new();
    subreddits.push("politics");
    subreddits.push("movies");

    for sub in subreddits {
        let url = request_url_builder(sub);
        let tx = tx.clone();

        pool.execute(move || {
            make_request(tx, url.as_str()).unwrap();
        });
    }

    drop(tx);

    let subreddit_collection = rx.into_iter().collect_vec();
    let all_collection = concat(subreddit_collection);

    loop {
        for post in &all_collection {
            println!("[{}]{}", post.data.subreddit, post.data.title);
            fs::write(saved_path, &post.data.url)?;
            thread::sleep(Duration::from_millis(10000));
        }
    }
}

fn request_url_builder(subreddit: &str) -> String {
    format!("https://www.reddit.com/r/{}.json?limit=10", subreddit)
}

fn make_request(tx: channel::Sender<Vec<ChildrenData>>, url: &str) -> Result<()> {
    let resp = ureq::get(&url).call().into_json_deserialize::<Response>()?;
    tx.send(resp.data.children)?;
    Ok(())
}

#[derive(Debug, Deserialize)]
pub struct Response {
    data: Children,
}

#[derive(Debug, Deserialize)]
pub struct Children {
    children: Vec<ChildrenData>,
}

#[derive(Debug, Deserialize)]
pub struct ChildrenData {
    data: Post,
}

#[derive(Debug, Deserialize)]
pub struct Post {
    title: String,
    url: String,
    subreddit: String,
}
