fn main() {
    if let Err(e) = gitignore::run() {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}
