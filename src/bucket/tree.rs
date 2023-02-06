use crate::util::{compress_data, decompress_data, get_file_hash, get_str_hash};
use std::{
    collections::HashMap,
    fmt::Display,
    fs::{self, File},
    io::{Error, Read, Write, BufReader, BufRead},
    path::PathBuf,
    process,
};

pub struct TreeEntry {
    pub hash: String,
    pub object_type: ObjectType,
    pub size: u64,
    pub name: String,
}

pub enum ObjectType {
    ObjectBlob,
    ObjectTree,
}

impl Display for ObjectType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ObjectType::ObjectBlob => write!(f, "blob"),
            ObjectType::ObjectTree => write!(f, "tree"),
        }
    }
}

impl TreeEntry {
    fn new(
        entry_path: PathBuf,
        object_type: ObjectType,
        svc_path: PathBuf,
        exclude: &HashMap<PathBuf, bool>,
    ) -> TreeEntry {
        let size;
        let hash;
        let name = entry_path
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();
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
        TreeEntry {
            object_type,
            hash,
            name,
            size,
        }
    }

    fn save_blob(entry_path: PathBuf, svc_path: PathBuf, hash: &str) -> Result<(), Error> {
        let dir = &hash[0..2];
        let filename = &hash[2..];
        println!("blob {:?}", hash);
        match fs::create_dir(svc_path.join("objects").join(dir)) {
            Ok(_) => (),
            Err(ref e) if e.kind() == std::io::ErrorKind::AlreadyExists => (),
            Err(e) => return Err(e),
        };
        let mut file_write = File::create(svc_path.join("objects").join(dir).join(filename))?;
        let mut file_read = File::open(entry_path)?;
        let mut buf = [0; 1024];

        while let Ok(bytes_read) = file_read.read(&mut buf) {
            if bytes_read == 0 {
                break;
            }
            let buf = compress_data(&buf[0..bytes_read]);
            file_write.write(&buf)?;
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
            Err(e) => return Err(e),
        };
        let mut file_write = File::create(svc_path.join("objects").join(dir).join(filename))?;
        file_write.write_fmt(format_args!("{} {}\n", tree.hash, tree.size))?;
        for entry in &tree.records {
            file_write.write_fmt(format_args!(
                "{} {} {} {}\n",
                entry.hash, entry.object_type, entry.size, entry.name
            ))?;
        }
        println!("tree {:?}", tree.hash);
        Ok(())
    }

    pub fn read_tree(tree_path: PathBuf) -> Vec<TreeEntry> {
        let file = File::open(tree_path).unwrap();
        let reader = BufReader::new(file);
        let mut tree_entries = Vec::new();
        for line in reader.lines() {
            let line = line.unwrap();
            let line: Vec<&str> = line.split(" ").collect();
            if line.len() < 4 {
                continue;
            }
            let object_type = match line[1] {
                "blob" => ObjectType::ObjectBlob,
                "tree" => ObjectType::ObjectTree,
                _ => ObjectType::ObjectBlob
            };
            let tree_entry = TreeEntry {
                hash: line[0].to_string(),
                object_type,
                size: line[2].parse::<u64>().unwrap(),
                name: line[3..].join(" ")
            };
            tree_entries.push(tree_entry);
        }
        tree_entries
    }

    pub fn restore_blob(file_path: PathBuf, blob_path: PathBuf) -> Result<(), Error> {
        let mut file_read = File::open(blob_path)?;
        let mut file_write = File::create(file_path)?;
        let mut buf = [0; 1024];

        while let Ok(bytes_read) = file_read.read(&mut buf) {
            if bytes_read == 0 {
                break;
            }
            let buf = decompress_data(&buf[0..bytes_read]);
            file_write.write(&buf)?;
        }
        Ok(())
    }
}

pub struct Tree {
    pub hash: String,
    pub size: u64,
    pub records: Vec<TreeEntry>,
}

impl Tree {
    pub fn new(dir: PathBuf, svc_path: PathBuf, exclude: &HashMap<PathBuf, bool>) -> Tree {
        let mut records: Vec<TreeEntry> = Vec::new();
        let mut str = String::from("");
        let mut size = 0;
        let mut tree_entry;

        for entry in fs::read_dir(dir).unwrap() {
            let entry = entry.unwrap();
            if (entry.file_name().to_str().unwrap().starts_with('.') && entry.file_name() != ".svcignore")
                || exclude.contains_key(&entry.path()) {
                continue;
            }
            println!("{:?}", entry.file_name());
            let metadata = entry.metadata().unwrap();
            if metadata.is_dir() {
                tree_entry = TreeEntry::new(
                    entry.path(),
                    ObjectType::ObjectTree,
                    svc_path.clone(),
                    exclude,
                );
            } else {
                tree_entry = TreeEntry::new(
                    entry.path(),
                    ObjectType::ObjectBlob,
                    svc_path.clone(),
                    exclude,
                );
                if let Err(err) =
                    TreeEntry::save_blob(entry.path(), svc_path.clone(), &tree_entry.hash)
                {
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