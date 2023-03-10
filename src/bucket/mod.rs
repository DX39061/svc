use std::{env, fs::{self, File}, io::{self, Write}, process, path::PathBuf};
use chrono::Local;
use repo::{check_svc_repo, RepoMeta};
use log::{Commit, check_blob_state, get_file_paths_in_dir};

mod repo;
mod log;
mod tree;

pub fn info() {
    // println!("command info");
    match check_svc_repo() {
        Ok(svc_path) => {
            let repo_meta = RepoMeta::read_repo_meta(svc_path);
            println!("-----------------------------------------");
            println!("{}", repo_meta);
            println!("-----------------------------------------");
            println!("notice: you can edit .svc/repo to set metadata mannually.")
        }
        Err(err) => {
            eprintln!("{}", err);
            process::exit(1)
        }
    }
}

pub fn init() {
    // println!("command init");
    match check_svc_repo() {
        Ok(_) => {
            eprintln!("error: svc repo already exists!");
            process::exit(1)
        }
        Err(_) => {
            println!("-----------------------------------------");
            let current_dir = env::current_dir().unwrap();
            fs::create_dir(current_dir.join(".svc")).unwrap();
            println!("notice: .svc create successfully.");
            fs::create_dir(current_dir.join(".svc/objects")).unwrap();
            println!("notice: .svc/objects create successfully.");
            fs::File::create(current_dir.join(".svc/repo")).unwrap();
            println!("notice: .svc/repo create successfully.");
            fs::File::create(current_dir.join(".svc/head")).unwrap();
            println!("notice: .svc/head create successfully.");
            fs::File::create(current_dir.join(".svc/log")).unwrap();
            println!("notice: .svc/log create successfully.");
            fs::File::create(current_dir.join(".svc/latest")).unwrap();
            println!("notice: .svc/latest create successfully.");
            println!("-----------------------------------------");
            
            let mut repo_name = String::from("");
            let mut repo_intro = String::from("");
            let mut remote_url = String::from("");
            println!("now you could set some metadata for this repo, or just type 'enter' to ignore them.");
            println!("repo_name: ");
            io::stdin().read_line(&mut repo_name).unwrap();
            println!("repo_intro: ");
            io::stdin().read_line(&mut repo_intro).unwrap();
            println!("remote_url: ");
            io::stdin().read_line(&mut remote_url).unwrap();
            println!("-----------------------------------------");

            let new_meta = RepoMeta::new(repo_name, repo_intro, remote_url);
            match RepoMeta::update_repo_meta(new_meta, current_dir.join(".svc")) {
                Ok(()) => {
                    println!("notice: svc repo initialize completely");
                }
                Err(err) => {
                    eprintln!("error: {}", err);
                }
            }
        }
    }
}

pub fn log() {
    // println!("command log");
    match check_svc_repo() {
        Ok(svc_path) => {
            let head_hash = Commit::get_head_hash(svc_path.clone());
            let commits = Commit::read_from_log(svc_path);
            if commits.len() == 0 {
                eprintln!("error: no commit yet");
            }
            for commit in commits.iter().rev() {
                if commit.hash == head_hash {
                    println!("commit {} (HEAD)", commit.hash);
                } else {
                    println!("commit {}", commit.hash);
                }
                println!("Date:  {}", commit.date);
                println!("\n\t{}\n", commit.message);
            }
        }
        Err(err) => {
            eprintln!("{}", err);
            process::exit(1)
        }
    }
}

pub fn status() {
    match check_svc_repo() {
        Ok(svc_path) => {
            let mut untracked: Vec<PathBuf> = Vec::new();
            let mut modified: Vec<PathBuf> = Vec::new();
            let root_path = svc_path.clone().parent().unwrap().to_path_buf();
            let mut files: Vec<PathBuf> = Vec::new();
            let exclude = Commit::read_ignore(svc_path.clone());
            files = get_file_paths_in_dir(root_path, &mut files, exclude).to_vec();
            for file_path in files {
                // println!("{:?}", file_path);
                if let Err(err) = check_blob_state(file_path.clone(), svc_path.clone()) {
                    if err == "not found" {
                        untracked.push(file_path.clone());
                    } else if err == "doesn't match" {
                        modified.push(file_path);
                    }
                }
            }
            if modified.len() == 0 && untracked.len() == 0 {
                println!("clean workspace.");
                return;
            }
            if modified.len() != 0 {
                println!("\nmodified but not saved:");
                println!("  (run \"svc checkout\" will get an error)");
                for file in modified {
                    println!("  {}", file.to_str().unwrap());
                }
            }
            if untracked.len() != 0 {
                println!("\nunntracked:");
                println!("  (run \"svc commmit\" will discard commits after HEAD)");
                for file in untracked {
                    println!("  {}", file.to_str().unwrap());
                }
            }
            println!("\nnotice: run \"svc commit\" to save current workspace");
        }
        Err(err) => {
            eprintln!("{}", err);
            process::exit(1)
        }
    }
}

pub fn commit(message: &String) {
    // println!("command commit");
    match check_svc_repo() {
        Ok(svc_path) => {
            Commit::check_and_update_latest(svc_path.clone());
            let commit = Commit::new(message.to_string(), svc_path.clone());
            let mut file_latest = File::create(svc_path.join("latest")).unwrap();
            file_latest.write(commit.hash.as_bytes()).unwrap();
            if let Err(err) = Commit::write_to_log(&commit, svc_path.clone()) {
                eprintln!("error: {}", err);
                process::exit(1);
            }
            let mut file = File::create(svc_path.clone().join("head")).unwrap();
            file.write(commit.hash.as_bytes()).unwrap();

            if let Err(err) = RepoMeta::update_repo_meta(RepoMeta{
                repo_name: "\n".to_string(),
                repo_intro: "\n".to_string(),
                remote_url: "\n".to_string(),
                created_at: "\n".to_string(),
                updated_at: Local::now().format("%Y-%m-%d %H:%M").to_string()
            }, svc_path) {
                eprintln!("error: {}", err);
                process::exit(1);
            }
            println!("workspace save successfully.");
        }
        Err(err) => {
            eprintln!("{}", err);
            process::exit(1)
        }
    }
}

pub fn checkout(version: &String) {
    // println!("Command checkout");
    // println!("{}", version);
    match check_svc_repo() {
        Ok(svc_path) => {
            let commits = Commit::read_from_log(svc_path.clone());
            let mut target: Option<Commit> = None;
            let mut target_cnt = 0;
            for commit in commits {
                if commit.hash.starts_with(&version.to_string()) {
                    target = Some(commit);
                    target_cnt += 1
                }
            }
            
            if target_cnt == 1 {
                let target = target.unwrap();
                let target_hash = target.hash;
                if let Err(err) = Commit::restore_tree(svc_path.parent().unwrap().to_path_buf(), svc_path.clone(), target.tree_hash) {
                    eprintln!("error: {}", err);
                    process::exit(1);
                } else {
                    Commit::reset_head(svc_path, target_hash.clone());
                    println!("switch to commit {}.", target_hash);
                }
            } else if target_cnt < 1 {
                eprintln!("error: version not found.");
                process::exit(1)
            } else if target_cnt > 1 {
                eprintln!("error: found more than one version matches.");
                process::exit(1)
            }
            
        }
        Err(err) => {
            eprintln!("{}", err);
            process::exit(1)
        }
    }
}
