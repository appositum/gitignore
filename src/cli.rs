use std::collections::HashMap;
use std::fs::{
    self,
    File,
    OpenOptions,
};
use std::io::Write;
use std::ops::Rem;

use ansi_term::Colour::Red;

pub fn flag_list(input: Vec<String>) {
    // space between columns will be
    // the size of the longest string
    let longest: usize = input
        .iter()
        .max_by(|s1, s2| s1.len().cmp(&s2.len()))
        .unwrap()
        .len();

    // terminal width measured in character count
    let (terminal_width, _) = term_size::dimensions().unwrap();
    let number_of_columns = terminal_width / (longest + 1);

    let mut list = input.clone();

    // add empty strings to make sure we can
    // split into exactly <number_of_columns> size chunks
    while list.len().rem(number_of_columns) != 0 {
        list.push(String::new());
    }

    let chunks = list.chunks(number_of_columns);

    for chunk in chunks {
        for c in chunk {
            print!("{:<w$}", c, w = longest + 1);
        }
        print!("\n");
    }
}

pub fn flag_search(search: String, templates: HashMap<String, String>) {
    let search_lowercase = search.to_lowercase();

    for (k, v) in templates {
        if k.contains(&search_lowercase) {
            let matched: Vec<_> = k.match_indices(&search_lowercase).collect();
            let (index_start, _) = matched[0]; // only need the first substring match
            let index_end = index_start + search_lowercase.len();
            let matched_substr = &v[index_start..index_end];

            /*
            $ gitignore -s ara
            Laravel

            str_start      = Lar
            matched_substr = ara
            str_end        = vel
            */
            let (str_start, _rest) = v.split_at(index_start);
            let (_rest, str_end) = v.split_at(index_end);
            println!("{str_start}{}{str_end}", Red.paint(matched_substr));
        }
    }
}

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
