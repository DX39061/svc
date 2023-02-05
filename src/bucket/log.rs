use crate::util::{get_str_hash, get_file_hash, compress_data};
use chrono::Local;
use std::{
    fs::{self, File, OpenOptions}, 
    io::{Read, Error, Write, BufReader, BufRead}, 
    path::PathBuf, 
    process, fmt::Display, collections::HashMap
};

pub struct Tree {
    hash: String,
    size: u64,
    records: Vec<TreeEntry>,
}

struct TreeEntry {
    hash: String,
    object_type: ObjectType,
    name: String,
    size: u64,
}

enum ObjectType {
    ObjectBlob,
    ObjectTree
}

impl Display for ObjectType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ObjectType::ObjectBlob => write!(f, "blob"),
            ObjectType::ObjectTree => write!(f, "tree")
        }
    }
}

impl TreeEntry {
    fn new(entry_path: PathBuf, object_type: ObjectType, svc_path: PathBuf, exclude: &HashMap<PathBuf, bool>) -> TreeEntry {
        let size;
        let hash;
        let name = entry_path.file_name().unwrap().to_str().unwrap().to_string();
        match object_type {
            ObjectType::ObjectBlob => {
                size = fs::metadata(entry_path.clone()).unwrap().len();
                hash = get_file_hash(entry_path);
            }
            ObjectType::ObjectTree => {
                let tree = Tree::new(entry_path, svc_path, exclude);
                size = tree.size;
                hash = tree.hash;
            }
        }
        TreeEntry { object_type, hash, name, size }
    }

    fn save_blob(entry_path: PathBuf, svc_path: PathBuf, hash: &str) -> Result<(), Error> {
        let dir = &hash[0..2];
        let filename = &hash[2..];
        println!("blob {:?}", hash);
        match fs::create_dir(svc_path.join("objects").join(dir)) {
            Ok(_) => (),
            Err(ref e) if e.kind() == std::io::ErrorKind::AlreadyExists => (),
            Err(e) => return Err(e)
        };
        let mut file_write = File::create(svc_path.join("objects").join(dir).join(filename))?;
        let mut file_read = File::open(entry_path)?;
        let mut buf = [0;1024];
        
        while let Ok(bytes_read) = file_read.read(&mut buf) {
            if bytes_read == 0 {
                break;
            }
            let buf = compress_data(&buf);
            file_write.write_all(&buf)?;
        }
        println!("blob {:?}", hash);
        Ok(())
    }

    fn save_tree(tree: &Tree, svc_path: PathBuf) -> Result<(), Error> {
        let dir = &tree.hash[0..2];
        let filename = &tree.hash[2..];
        println!("tree {:?}", tree.hash);
        match fs::create_dir(svc_path.join("objects").join(dir)) {
            Ok(_) => (),
            Err(ref e) if e.kind() == std::io::ErrorKind::AlreadyExists => (),
            Err(e) => return Err(e)
        };
        let mut file_write = File::create(svc_path.join("objects").join(dir).join(filename))?;
        file_write.write_fmt(format_args!("{} {}\n", tree.hash, tree.size))?;
        for entry in &tree.records {
            file_write.write_fmt(format_args!("{} {} {} {}\n", entry.hash, entry.object_type, entry.name, entry.size))?;
        }
        println!("tree {:?}", tree.hash);
        Ok(())
    }
}

pub struct Commit {
    pub hash: String,
    pub parent_hash: String,
    pub data: Tree,
    pub date: String,
    pub message: String,
}

impl Tree {
    fn new(dir: PathBuf, svc_path: PathBuf, exclude: &HashMap<PathBuf, bool>) -> Tree {
        let mut records: Vec<TreeEntry> = Vec::new();
        let mut str = String::from("");
        let mut size = 0;
        let mut tree_entry;

        
        
        for entry in fs::read_dir(dir).unwrap() {
            let entry = entry.unwrap();
            if entry.file_name().to_str().unwrap().starts_with('.') || exclude.contains_key(&entry.path()) {
                continue;
            }
            println!("{:?}", entry.file_name());
            let metadata = entry.metadata().unwrap();
            if metadata.is_dir() {
                tree_entry = TreeEntry::new(entry.path(), ObjectType::ObjectTree, svc_path.clone(), exclude);
            } else {
                tree_entry = TreeEntry::new(entry.path(), ObjectType::ObjectBlob, svc_path.clone(), exclude);
                if let Err(err) = TreeEntry::save_blob(entry.path(), svc_path.clone(), &tree_entry.hash) {
                    eprintln!("error: {}", err);
                    process::exit(1);
                }
            }
            str += &tree_entry.hash;
            size += tree_entry.size;
            records.push(tree_entry);
        }
        let hash = get_str_hash(&str);
        let tree = Tree {
            hash,
            size,
            records,
        };
        if let Err(err) = TreeEntry::save_tree(&tree, svc_path) {
            eprintln!("error: {}", err);
            process::exit(1);
        }
        tree
    }

    
}

impl Commit {
    pub fn new(message: String, svc_path: PathBuf) -> Commit {
        let date = Local::now().format("%Y-%m-%d %H:%M").to_string();
        let str = date.clone() + &message[..] + "commit";
        let hash = get_str_hash(&str);
        let parent_hash = Commit::get_head_hash(svc_path.clone());
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
        let data = Tree::new(svc_path.clone().parent().unwrap().to_path_buf(), svc_path, &exclude);
        Commit {
            hash,
            parent_hash,
            message,
            date,
            data,
        }
    }
    pub fn write_to_log(commit: &Commit, svc_path: PathBuf) -> Result<(), Error>{
        let mut file = OpenOptions::new().append(true).open(svc_path.join("log"))?;
        file.write_fmt(format_args!("{} {} {} {} {}\n", commit.hash, commit.parent_hash, commit.data.hash, commit.date, commit.message))?;
        Ok(())
    }

    fn get_head_hash(svc_path: PathBuf) -> String {
        let mut file = File::open(svc_path.join("head")).unwrap();
        let mut buf = String::from("");
        file.read_to_string(&mut buf).unwrap();
        buf.trim().to_string()
    }
}
