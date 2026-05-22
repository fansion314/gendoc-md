fn main() {
    if let Err(error) = gendoc_md::run() {
        eprintln!("{error:#}");
        std::process::exit(1);
    }
}
