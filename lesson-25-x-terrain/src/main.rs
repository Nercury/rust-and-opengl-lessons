extern crate env_logger;
extern crate failure;

mod debug;

fn main() {
    let mut builder = env_logger::Builder::new();
    builder.filter(None, log::LevelFilter::Trace);
    builder.default_format_module_path(true);
    builder.default_format_level(true);
    if ::std::env::var("RUST_LOG").is_ok() {
        builder.parse(&::std::env::var("RUST_LOG").unwrap());
    }
    builder.init();

    if let Err(e) = run() {
        println!("{}", debug::failure_to_string(e));
    }
}

fn run() -> Result<(), failure::Error> {
    println!("hello");
    Ok(())
}