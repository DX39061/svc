pub fn info() {
    println!("command info")
}

pub fn init() {
    println!("command init")
}

pub fn log() {
    println!("commad log")
}

pub fn commit() {
    println!("commad commit")
}

pub fn checkout(version: &String) {
    println!("Command checkout");
    println!("{}", version)
}