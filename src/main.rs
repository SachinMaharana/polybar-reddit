#[global_allocator]
static GLOBAL: jemallocator::Jemalloc = jemallocator::Jemalloc;

use anyhow::Result;
use crossbeam_channel as channel;
use itertools::{concat, Itertools};
use serde::{Deserialize, Serialize};
use std::thread;
use std::time::Duration;
use std::{
    env, fs,
    path::{Path, PathBuf},
};
use threadpool::ThreadPool;
// const DEFAULT_CONFIG_PATH = ""
const DEFAULT_CONFIG_FILE_NAME: &str = "default.toml";
const SAVED_PATH_FILE: &str = "current_post.txt";

pub fn get_polybar_reddit_home_dir() -> Result<PathBuf> {
    let config_dir = if let Ok(value) = env::var("POLYBAR_REDDIT") {
        println!("Using $POLYBAR_REDDIT: {}", value);
        Path::new(&value).to_path_buf()
    } else {
        println!("No $POLYBAR_REDDIT detected, using $HOME");
        dirs::home_dir()
            .expect("Could not find home directory")
            .join(".polybarreddit")
    };
    Ok(config_dir)
}

fn get_global_config_path() -> Result<PathBuf> {
    let home_dir = get_polybar_reddit_home_dir()?;
    let global_config_file = home_dir.join("config").join(DEFAULT_CONFIG_FILE_NAME);
    println!("Using global config file: {}", global_config_file.display());
    Ok(global_config_file)
}
fn get_saved_path() -> Result<PathBuf> {
    let home_dir = get_polybar_reddit_home_dir()?;
    let saved_file = home_dir.join("config").join(SAVED_PATH_FILE);
    println!("Using global config file: {}", saved_file.display());
    Ok(saved_file)
}

#[derive(Debug, Deserialize, Serialize)]
struct Config {
    subreddit: Vec<String>,
}

impl Config {
    fn to_file(&self, config_path: &Path) -> Result<()> {
        let toml = toml::to_string(self)?;
        fs::create_dir_all(&config_path.parent().unwrap())?;
        fs::write(&config_path, toml)?;
        Ok(())
    }
}

fn main() -> Result<()> {
    let saved_path = "/home/sachin/.config/polybar/current_post.txt";
    let config_file = get_global_config_path()?;
    if !config_file.exists() {
        println!(
            "Config File Doesn't exist. Run polybar-reddit init to create one with default values"
        );
        std::process::exit(1)
    }
    println!("File {:?}", config_file);

    let mut subreddits = Vec::new();
    subreddits.push("politics".to_string());
    subreddits.push("movies".to_string());

    let config = Config {
        subreddit: subreddits,
    };

    config.to_file(&config_file)?;

    let (tx, rx) = channel::unbounded();
    let pool = ThreadPool::new(4);

    for sub in config.subreddit {
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

fn request_url_builder(subreddit: String) -> String {
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
