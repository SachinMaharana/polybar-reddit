use anyhow::{bail, Result};
use channel::Sender;
use crossbeam_channel as channel;
use itertools::{concat, Itertools};
use serde::{Deserialize, Serialize};
use std::{borrow::Cow, thread};
use std::{
    env, fs,
    path::{Path, PathBuf},
};

use log::info;
use rustop::opts;

use std::{str::FromStr, time::Duration};
use threadpool::ThreadPool;

const DEFAULT_CONFIG_FILE_NAME: &str = "default.toml";
const SAVED_PATH_FILE: &str = "current_post.txt";

enum UrlType<'a> {
    JsonUrl(Cow<'a, str>),
    HealthUrl(Cow<'a, str>),
}

impl<'a> UrlType<'a> {
    fn value(&self) -> String {
        match &*self {
            UrlType::JsonUrl(subreddit) => {
                format!("https://www.reddit.com/r/{}.json?limit=10", subreddit)
            }
            UrlType::HealthUrl(subreddit) => format!("https://www.reddit.com/r/{}", subreddit),
        }
    }
}

pub fn get_polybar_reddit_home_dir() -> Result<PathBuf> {
    let config_dir = if let Ok(value) = env::var("POLYBAR_REDDIT") {
        info!("Using $POLYBAR_REDDIT: {}", value);
        Path::new(&value).to_path_buf()
    } else {
        info!("No $POLYBAR_REDDIT detected, using $HOME");
        dirs::home_dir()
            .expect("Could not find home directory")
            .join(".polybarreddit")
    };
    Ok(config_dir)
}

fn get_global_config_path() -> Result<PathBuf> {
    let home_dir = get_polybar_reddit_home_dir()?;
    let global_config_file = home_dir.join("config").join(DEFAULT_CONFIG_FILE_NAME);
    info!("Using global config file: {}", global_config_file.display());
    Ok(global_config_file)
}

fn get_saved_path() -> Result<PathBuf> {
    let home_dir = get_polybar_reddit_home_dir()?;
    let saved_file = home_dir.join("config").join(SAVED_PATH_FILE);
    info!("Using saved path: {}", saved_file.display());
    Ok(saved_file)
}

#[derive(Debug, Deserialize, Serialize, Default)]
struct Config<'a> {
    subreddits: Vec<Cow<'a, str>>,
}

impl<'a> FromStr for Config<'a> {
    type Err = toml::de::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        toml::from_str(s)
    }
}

impl<'a> Config<'a> {
    fn to_file(&self, config_path: &Path) -> Result<()> {
        let toml = toml::to_string(self)?;
        fs::create_dir_all(config_path.parent().unwrap())?;
        fs::write(&config_path, toml)?;
        Ok(())
    }

    fn get_config<S: AsRef<Path>>(&self, config_file: S) -> Option<Config<'static>> {
        match fs::read_to_string(config_file) {
            Ok(contents) => match Config::from_str(&contents) {
                Ok(config) => Some(config),
                Err(_) => None,
            },
            Err(_) => None,
        }
    }

    fn init<S: AsRef<Path>>(&self, config_file: S) -> Result<()> {
        let subreddits = vec!["politics".into(), "movies".into()];
        let config = Config { subreddits };
        config.to_file(config_file.as_ref())?;
        Ok(())
    }
}

fn main() -> Result<()> {
    env_logger::init();

    let saved_path = get_saved_path()?;
    let config_file = get_global_config_path()?;
    let config = Config::default();

    let (args, _) = opts! {
        synopsis "polybar-reddit. show top reddit posts in polybar.";
        version env!("CARGO_PKG_VERSION");
        opt init:bool=false, desc: "initilaize this cli";
    }
    .parse_or_exit();

    if args.init {
        if config_file.exists() {
            eprintln!(
                "already initialized. find the config file at {}",
                config_file.display()
            );
            std::process::exit(0);
        }
        config.init(&config_file)?;
        eprintln!(
            "successfully initialized. add your subreddits in the config file at {}.
            polybar config snippet:
            ...
            exec = // polybar-reddit binary location
            click-left = < {}  xargs -I % xdg-open %
            ...
            ",
            config_file.display(),
            saved_path.display()
        );
        std::process::exit(1);
    }

    if !config_file.exists() {
        eprintln!(
            "config file doesn't exist. run `polybar-reddit --init` in you command line to create one with default values."
        );
        std::process::exit(1)
    }

    let subreddits = match config.get_config(config_file) {
        Some(config) => config.subreddits,
        None => bail!("Not valid Reddits Found"),
    };

    if subreddits.is_empty() || subreddits.contains(&Cow::from("")) {
        bail!("empty reddits not allowed");
    }

    eprintln!("Verifying...");

    bail_if_subredits_doesnt_exists(&subreddits)?;

    let (tx, rx) = channel::unbounded();
    let pool = ThreadPool::new(2);

    for sub in subreddits {
        let url = UrlType::JsonUrl(sub).value();
        let tx = tx.clone();

        pool.execute(move || {
            make_request(tx, &url).unwrap();
        });
    }

    drop(tx);

    let subreddit_collection = concat(rx.into_iter().collect_vec());
    eprintln!("Launching...");
    loop {
        for post in &subreddit_collection {
            let reddit_url = format!("https://reddit.com{}", post.data.permalink);
            eprintln!("[{}]{}", post.data.subreddit, post.data.title);
            fs::write(&saved_path, reddit_url)?;
            thread::sleep(Duration::from_millis(10_000));
        }
    }
}

fn bail_if_subredits_doesnt_exists(subreddits: &[Cow<str>]) -> Result<()> {
    for s in &subreddits.to_owned() {
        let url = UrlType::HealthUrl(s.to_owned()).value();
        let resp = ureq::get(&url).timeout_connect(10_000).call();
        if resp.status() != 200 {
            bail!("not valid response!check valid subreddit/connected to internet",)
        }
    }
    Ok(())
}

fn make_request(tx: Sender<Vec<ChildrenData>>, url: &str) -> Result<()> {
    let resp = ureq::get(&url)
        .timeout_connect(8_000)
        .call()
        .into_json_deserialize::<Response>()?;
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
    permalink: String,
    subreddit: String,
}
