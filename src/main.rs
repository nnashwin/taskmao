use clap::clap_app;
use std::fs;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
extern crate dirs;

fn read(file: &str) -> Result<String, std::io::Error> {
    let mut file = File::open(file)?;

    let mut data = String::new();
    file.read_to_string(&mut data)?;
    Ok(data)
}

fn main() -> std::io::Result<()> {
    let matches = clap_app!(myapp =>
                            (version: "1.0")
                            (author: "Tyler B. <tyler@tylerboright.com>")
                            (about: "The brain gains power through noticing.  Notice how you spend your time.")
                            (@arg NAME: +required "Sets the name of a task to execute")
                            (@arg CATEGORY: -c --category +takes_value "Sets the category of the task")
                           ).get_matches();

    // create directory and files if they don't exist
    let mut path: PathBuf = match dirs::home_dir() {
        Some(path) => PathBuf::from(path),
        None => PathBuf::from(""),
    };

    path.push(".taskmaster");

    fs::create_dir_all(path.as_path())?;

    println!("{}", "path created");

    path.push("entries.json");

    let f = match read(path.to_str().unwrap_or_default()) {
        Ok(file) => file,
        Err(_err) => String::from(""),
    };
    println!("{}", f);

    if let Some(n) = matches.value_of("NAME") {
        println!("Values for name: {}", n);
    }

    if let Some(c) = matches.value_of("CATEGORY") {
        println!("Values for category: {}", c);
    }

    // write file
    match File::create(path.to_str().unwrap_or_default()) {
        Err(why) => panic!("couldn't create entries file at path {:?}: {}", path.to_str(), why),
        Ok(file) => file,
    };

    Ok(())
}
