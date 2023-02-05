use crate::util::get_str_hash;
use crate::bucket::tree::Tree;
use chrono::Local;
use std::{
    collections::HashMap,
    fs::{File, OpenOptions},
    io::{BufRead, BufReader, Error, Read, Write},
    path::PathBuf
};

pub struct Commit {
    pub hash: String,
    pub parent_hash: String,
    pub tree_hash: String,
    pub date: String,
    pub message: String,
}

impl Commit {
    pub fn new(message: String, svc_path: PathBuf) -> Commit {
        let date = Local::now().format("%Y-%m-%d %H:%M").to_string();
        let str = date.clone() + &message[..] + "commit";
        let hash = get_str_hash(&str);
        let mut parent_hash = Commit::get_head_hash(svc_path.clone());
        // first commit has no parent
        if parent_hash == "" {
            parent_hash = String::from("0000000000000000000000000000000000000000");
        }
        // exclude files declared in '.svcignore'
        let mut exclude: HashMap<PathBuf, bool> = HashMap::new();
        if let Ok(file) = File::open(svc_path.parent().unwrap().join(".svcignore")) {
            let reader = BufReader::new(file);
            for line in reader.lines() {
                let line = line.unwrap();
                let path = svc_path.parent().unwrap().join(line.trim());
                println!("exclude {:?}", path);
                exclude.insert(path, true);
            }
        }
        let tree_hash = Tree::new(
            svc_path.clone().parent().unwrap().to_path_buf(),
            svc_path,
            &exclude,
        )
        .hash;
        Commit {
            hash,
            parent_hash,
            tree_hash,
            message,
            date,
        }
    }

    pub fn write_to_log(commit: &Commit, svc_path: PathBuf) -> Result<(), Error> {
        let mut file = OpenOptions::new().append(true).open(svc_path.join("log"))?;
        file.write_fmt(format_args!(
            "{} {} {} {} {}\n",
            commit.hash, commit.parent_hash, commit.tree_hash, commit.date, commit.message
        ))?;
        Ok(())
    }

    pub fn read_from_log(svc_path: PathBuf) -> Vec<Commit> {
        let mut commits: Vec<Commit> = Vec::new();
        let file = File::open(svc_path.join("log")).unwrap();
        let reader = BufReader::new(file);
        for line in reader.lines() {
            let line = line.unwrap();
            let line: Vec<&str> = line.split_whitespace().collect();
            commits.push(Commit {
                hash: line[0].to_string(),
                parent_hash: line[1].to_string(),
                tree_hash: line[2].to_string(),
                date: line[3].to_string() + " " + line[4],
                message: line[5..].join(" "),
            })
        }
        commits
    }

    pub fn get_head_hash(svc_path: PathBuf) -> String {
        let mut file = File::open(svc_path.join("head")).unwrap();
        let mut buf = String::from("");
        file.read_to_string(&mut buf).unwrap();
        buf.trim().to_string()
    }
}
