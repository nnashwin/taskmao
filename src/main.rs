use clap::clap_app;

fn main() {
    let matches = clap_app!(myapp =>
        (version: "1.0")
        (author: "Tyler B. <tyler@tylerboright.com>")
        (about: "The brain gains power through noticing.  Notice how you spend your time.")
        (@arg NAME: +required "Sets the name of a task to execute")
        (@arg CATEGORY: -c --category +takes_value "Sets the category of the task")
    ).get_matches();

    if let Some(n) = matches.value_of("NAME") {
        println!("Values for name: {}", n);
    }

    if let Some(c) = matches.value_of("CATEGORY") {
        println!("Values for name: {}", c);
    }

}
