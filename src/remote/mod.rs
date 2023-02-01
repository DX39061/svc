pub fn pull() {
    println!("Command pull")
}

pub fn push() {
    println!("Command push")
}

pub fn set_remote(url: &String) {
    println!("Command set-remote");
    println!("{}", url)
}