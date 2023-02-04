use repo::{check_svc_repo, RepoMeta};
use std::{env, fs, io, process};
mod repo;
mod storage;

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
    println!("command init");
    match check_svc_repo() {
        Ok(_) => {
            eprintln!("error: svc repo already exists!");
            process::exit(1)
        }
        Err(_) => {
            println!("-----------------------------------------");
            let current_dir = env::current_dir().unwrap();
            fs::create_dir(current_dir.join(".svc")).unwrap();
            println!("notice: create .svc directory successfully.");
            let file = fs::File::create(current_dir.join(".svc/repo")).unwrap();
            drop(file);
            println!("notice: create .svc/repo successfully.");
            let mut repo_name = String::from("");
            let mut repo_intro = String::from("");
            let mut remote_url = String::from("");
            println!("-----------------------------------------");
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
    println!("command log")
}

pub fn commit() {
    println!("command commit")
}

pub fn checkout(version: &String) {
    println!("Command checkout");
    println!("{}", version)
}
