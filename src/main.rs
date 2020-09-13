use anyhow::Result;
use serde::Deserialize;
use std::fs;
use std::thread;
use std::time::Duration;

fn main() -> Result<()> {
    let saved_path = "/home/sachin/.config/polybar/current_post.txt";
    
    let politics = "politics";
    let movies = "movies";
    let tele = "television";
    let indiainvestments = "indiainvestments";

    let politics_url = request_url_builder(politics);
    let movies_url = request_url_builder(movies);
    let tele_url = request_url_builder(tele);
    let indiainvestments_url = request_url_builder(indiainvestments);

    let  politics_response = make_request(&politics_url.as_str())?;
    let  movies_response = make_request(&movies_url.as_str())?;
    let  tele_response = make_request(&tele_url.as_str())?;
    let  indiainvestments_response = make_request(&indiainvestments_url.as_str())?;

    let response: Vec<Child> = politics_response
        .into_iter()
        .chain(movies_response.into_iter())
        .chain(indiainvestments_response.into_iter())
        .chain(tele_response.into_iter())
        .collect();

    loop {
        for post in &response {
            println!("[{}]{}", post.data.subreddit, post.data.title);
            fs::write(saved_path, &post.data.url)?;
            thread::sleep(Duration::from_millis(10000));
        }
    }
}

fn request_url_builder(subreddit: &str) -> String {
    format!("https://www.reddit.com/r/{}.json?limit=10", subreddit)
}

fn make_request(url: &str) -> Result<Vec<Child>> {
    let resp = ureq::get(&url).call().into_json_deserialize::<Response>()?;
    return Ok(resp.data.children);
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
    subreddit: String,
}
