use std::fs::{File, OpenOptions};
use std::io::Write;

pub fn flag_append(text: String) {
    let mut file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(".gitignore")
        .unwrap();

    file.write_all(format!("{}\n", text).as_bytes()).unwrap();
}

pub fn flag_output(text: String, filename: &str) {
    let mut file = File::create(filename).unwrap();

    file.write_all(format!("{}\n", text).as_bytes()).unwrap();
}

pub fn flag_overwrite(text: String) {
    flag_output(text, ".gitignore")
}
