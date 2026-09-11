use std::{
    fs::{
        create_dir_all,
        read_dir,
        read_to_string,
        write,
    },
    path::PathBuf,
};

pub fn gitignore_cache_dir() -> PathBuf {
    return dirs::cache_dir().unwrap().join("gitignore.rs");
}

pub fn cache_list(template_list: &Vec<String>) {
    for template in template_list {
        let cache_dir = gitignore_cache_dir().join(template);
        let _ = create_dir_all(cache_dir);
    }
}

pub fn cache_template(template: &String, content: &String) {
    let cache_dir = gitignore_cache_dir().join(template);
    let _ = create_dir_all(&cache_dir);
    let _ = write(cache_dir.join(format!("{template}.gitignore")), content);
}

pub fn get_cached_list() -> Vec<String> {
    let mut list = vec![];

    if let Ok(paths) = read_dir(gitignore_cache_dir()) {
        for path in paths {
            list.push(path.unwrap().file_name().into_string().unwrap());
        }

        list.sort();
    }

    list
}

pub fn get_cached_template(template: &String) -> Option<String> {
    let filename = gitignore_cache_dir()
        .join(template)
        .join(format!("{template}.gitignore"));

    if let Ok(contents) = read_to_string(filename) {
        Some(contents)
    } else {
        None
    }
}
