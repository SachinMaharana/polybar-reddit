use anyhow::Result;
use dotenv::dotenv;
use serde::Deserialize;
use std::thread;
use std::time::Duration;
use std::fs;

fn main() -> Result<()> {
    dotenv().ok();

    let saved_path = "/home/sachin/.config/polybar/current_post.txt";
    let subreddit = "politics+movies+television";

    let request_url = request_url_builder(subreddit);

    let response = make_request(&request_url.as_str())?;

    let data = get_data(response);

    loop {
        for post in &data {
            println!(
           "[{}]{}",
           post.subreddit,
           post.title,
       );
       fs::write(saved_path, &post.url)?;
       thread::sleep(Duration::from_millis(10000));
       }
    }
    Ok(())
}


fn request_url_builder(subreddit: &str) -> String {
    format!(
        "https://www.reddit.com/r/{}.json?limit=75",
        subreddit
    )
}

fn make_request(url: &str) -> Result<Vec<Child>>  {
    let resp = ureq::get(&url).call().into_json_deserialize::<Response>()?;
    return Ok(resp.data.children)
}

fn get_data(data: Vec<Child>) -> Vec<Post> {
    let mut titles = vec![];
    for post in data {
        titles.push(post.data)
    }
    titles
}

#[derive(Debug, Deserialize)]
pub struct Response {
    data: Children,
}

#[derive(Debug, Deserialize)]
pub struct Children {
    children: Vec<Child>,
}

#[derive(Debug, Deserialize)]
pub struct Child {
    data: Post,
}

#[derive(Debug, Deserialize)]
pub struct Post {
    title: String,
    url: String,
    subreddit: String
}

