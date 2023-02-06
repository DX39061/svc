use crate::util::{get_str_hash, get_file_hash};
use crate::bucket::tree::{Tree, TreeEntry, ObjectType};
use chrono::Local;
use std::path::Component;
use std::{
    collections::HashMap,
    fs::{self, File, OpenOptions},
    io::{BufRead, BufReader, Error, Read, Write},
    path::PathBuf,
    process
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

    pub fn reset_head(svc_path: PathBuf, head_hash: String) {
        let mut file = File::create(svc_path.join("head")).unwrap();
        file.write(head_hash.as_bytes()).unwrap();
    }

    pub fn restore_tree(dir: PathBuf, svc_path: PathBuf, tree_hash: String) -> Result<(), Error> {
        let tree_dir = svc_path.join("objects").join(&tree_hash[0..2]);
        let tree_path = tree_dir.join(&tree_hash[2..]);
        let tree_entries = TreeEntry::read_tree(tree_path);
        for entry in tree_entries {
            match entry.object_type {
                ObjectType::ObjectBlob => {
                    let blob_dir = svc_path.join("objects").join(&entry.hash[0..2]);
                    let blob_path = blob_dir.join(&entry.hash[2..]);
                    confirm_blob_restore(dir.clone(), svc_path.clone(), entry.name.clone());
                    TreeEntry::restore_blob(dir.clone().join(entry.name), blob_path)?;
                }
                ObjectType::ObjectTree => {
                    match fs::create_dir(dir.join(entry.name.clone())) {
                        Ok(_) => (),
                        Err(ref e) if e.kind() == std::io::ErrorKind::AlreadyExists => (),
                        Err(e) => return Err(e),
                    };
                    Commit::restore_tree(dir.join(entry.name), svc_path.clone(), entry.hash)?;
                }
            }
        }
        Ok(())
    }

    pub fn check_and_update_latest(svc_path: PathBuf) {
        let mut head_commit_hash = String::new();
        let mut latest_commit_hash = String::new();
        let mut file_head = File::open(svc_path.join("head")).unwrap();
        let mut file_latest = File::open(svc_path.join("latest")).unwrap();
        file_head.read_to_string(&mut head_commit_hash).unwrap();
        file_latest.read_to_string(&mut latest_commit_hash).unwrap();

        if latest_commit_hash == head_commit_hash {
            return;
        }
        let commits = Commit::read_from_log(svc_path.clone());
        fs::remove_file(svc_path.join("log")).unwrap();
        File::create(svc_path.join("log")).unwrap();
        let mut file_log = OpenOptions::new().append(true).open(svc_path.join("log")).unwrap();
        let mut file_log_bak = File::create(svc_path.join("log.bak")).unwrap();
        let mut remove_flag = false;
        
        for commit in commits {
            println!("commit: {:?}", commit.hash);
            if remove_flag {
                file_log_bak.write_fmt(format_args!(
                    "{} {} {} {} {}\n",
                    commit.hash, commit.parent_hash, commit.tree_hash, commit.date, commit.message
                )).unwrap();
            } else {
                file_log.write_fmt(format_args!(
                    "{} {} {} {} {}\n",
                    commit.hash, commit.parent_hash, commit.tree_hash, commit.date, commit.message
                )).unwrap();
            }
            if commit.hash == head_commit_hash {
                remove_flag = true;
            }
        }
    }
}

fn confirm_blob_restore(dir: PathBuf, svc_path: PathBuf, filename: String) {
    let mut relative_path = Vec::new();
    let file_path = dir.join(filename.clone());
    let mut file_components = file_path.components();
    let mut svc_components = svc_path.parent().unwrap().components();
    while let Some(c1) = file_components.next() {
        if let Some(c2) = svc_components.next() {
            if c1 == c2 {
                continue;
            }
        }
        relative_path.push(c1);
    }
    // println!("{:?}", relative_path);
    let head_hash = Commit::get_head_hash(svc_path.clone());
    if let Ok(tree_hash) = get_tree_of_commit(svc_path.clone(), head_hash) {
        if let Ok(blob_hash) = get_blob_hash_from_entry(svc_path.clone(), tree_hash, relative_path) {
            let file_hash = get_file_hash(file_path);
            if blob_hash == file_hash {
                return;
            }
            eprintln!("error: \'{}\' was modified but not saved." , filename);
            eprintln!("error: forced version switching will result in data loss.");
            process::exit(1);
        }
    }
    
}

fn get_tree_of_commit(svc_path: PathBuf, commit_hash: String) -> Result<String, ()> {
   let commits = Commit::read_from_log(svc_path);
   for commit in commits {
    if commit_hash == commit.hash {
        return Ok(commit.tree_hash);
    }
   } 
   Err(())
}

fn get_blob_hash_from_entry(svc_path: PathBuf, tree_hash: String, relative_path: Vec<Component>) -> Result<String, ()>{
    let tree_path = svc_path.join("objects").join(&tree_hash[0..2]).join(&tree_hash[2..]);
    let tree_entries = TreeEntry::read_tree(tree_path);
    
    for tree_entry in tree_entries {
        if tree_entry.name == relative_path[0].as_os_str().to_str().unwrap() {
            match tree_entry.object_type {
                ObjectType::ObjectBlob => {
                    return Ok(tree_entry.hash);
                }
                ObjectType::ObjectTree => {
                    return get_blob_hash_from_entry(svc_path, tree_entry.hash, relative_path[1..].to_vec());
                }
            }
        }
    }
    Err(())
}