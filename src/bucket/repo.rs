use chrono::Local;
use std::{
    fmt::Display,
    fs::File,
    io::{BufRead, BufReader, Error, Write},
    path::PathBuf,
};
pub struct RepoMeta {
    repo_name: String,
    repo_intro: String,
    remote_url: String,
    created_at: String,
    updated_at: String,
}

impl Display for RepoMeta {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "repo_name = {}\nrepo_intro = {}\nremote_url = {}\ncreated_at = {}\nupdated_at = {}",
            self.repo_name, self.repo_intro, self.remote_url, self.created_at, self.updated_at
        )
    }
}

impl RepoMeta {
    pub fn new(repo_name: String, repo_intro: String, remote_url: String) -> RepoMeta {
        let created_at = Local::now().format("%Y-%m-%d %H:%M").to_string();
        let updated_at = created_at.clone();

        // println!("{:?}", created_at);
        RepoMeta {
            repo_name,
            repo_intro,
            remote_url,
            created_at,
            updated_at,
        }
    }

    pub fn read_repo_meta(svc_path: PathBuf) -> RepoMeta {
        // println!("{:?}", svc_path.join("repo"));
        let file =
            File::open(svc_path.join("repo")).expect("can not read .svc/repo, maybe it lost?");
        let file = BufReader::new(file);
        let mut repo_name = String::from("");
        let mut repo_intro = String::from("");
        let mut remote_url = String::from("");
        let mut created_at = String::from("");
        let mut updated_at = String::from("");
        for line in file.lines() {
            let line = line.unwrap();
            let line: Vec<&str> = line.split('=').collect();
            if line.len() != 2 {
                continue;
            }
            let k = line[0].trim();
            let v = line[1].trim().to_string();
            match k {
                "repo_name" => {
                    repo_name = v;
                }
                "repo_intro" => {
                    repo_intro = v;
                }
                "remote_url" => {
                    remote_url = v;
                }
                "created_at" => {
                    created_at = v;
                }
                "updated_at" => {
                    updated_at = v;
                }
                _ => (),
            }
        }
        RepoMeta {
            repo_name,
            repo_intro,
            remote_url,
            created_at,
            updated_at,
        }
    }

    pub fn update_repo_meta(new_meta: RepoMeta, svc_path: PathBuf) -> Result<(), Error> {
        let old_meta = RepoMeta::read_repo_meta(svc_path.clone());
        let mut repo_name = old_meta.repo_name;
        let mut repo_intro = old_meta.repo_intro;
        let mut remote_url = old_meta.remote_url;
        let mut created_at = old_meta.created_at;
        let mut updated_at = old_meta.updated_at;
        if new_meta.repo_name != "\n" {
            repo_name = new_meta.repo_name;
        }
        if new_meta.repo_intro != "\n" {
            repo_intro = new_meta.repo_intro;
        }
        if new_meta.remote_url != "\n" {
            remote_url = new_meta.remote_url;
        }
        if new_meta.created_at != "\n" {
            created_at = new_meta.created_at;
        }
        if new_meta.updated_at != "\n" {
            updated_at = new_meta.updated_at;
        }
        // println!("{:?}", svc_path.join("repo"));
        let mut file =
            File::create(svc_path.join("repo")).expect("can not open .svc/repo, maybe it lost?");
        
        file.write_fmt(format_args!("repo_name = {}\n", repo_name.trim()))?;
        file.write_fmt(format_args!("repo_intro = {}\n", repo_intro.trim()))?;
        file.write_fmt(format_args!("remote_url = {}\n", remote_url.trim()))?;
        file.write_fmt(format_args!("created_at = {}\n", created_at.trim()))?;
        file.write_fmt(format_args!("updated_at = {}\n", updated_at.trim()))?;

        Ok(())
    }
}

pub fn check_svc_repo() -> Result<PathBuf, &'static str> {
    let mut path = std::env::current_dir().unwrap();
    // println!("{:?}", path);
    loop {
        let svc_path = path.join(".svc");
        if let Ok(true) = svc_path.try_exists() {
            return Ok(svc_path);
        }
        if let Some(parent) = path.parent() {
            // println!("{:?}", parent);
            path = parent.to_path_buf();
        } else {
            return Err("not a svc repo yet");
        }
    }
}
