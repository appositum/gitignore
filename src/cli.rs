use std::fs::{
    self,
    File,
    OpenOptions,
};
use std::io::Write;

pub fn flag_append(text: String) {
    let mut append = true;

    if fs::exists(".gitignore").unwrap() {
        if fs::read_to_string(".gitignore").unwrap().is_empty() {
            append = false;
        }
    } else {
        append = false;
    }

    let mut file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(".gitignore")
        .unwrap();

    let formatted_text = if append { format!("\n{}", text) } else { text };

    file.write_all(formatted_text.as_bytes()).unwrap();
}

pub fn flag_output(text: String, filename: String) {
    let mut file = File::create(filename).unwrap();

    file.write_all(text.as_bytes()).unwrap();
}

pub fn flag_overwrite(text: String) {
    flag_output(text, String::from(".gitignore"))
}
