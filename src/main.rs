use clap::{App, Arg, SubCommand};
use reqq::Reqq;

fn main() {
    println!("Hello, world!");

    let matches = App::new("reqq").version("1.0.0")
        .author("Seth Etter <mail@sethetter.com>")
        .about("You know..")

        // TODO: optional --dir option to override default of .reqq

        // .arg(Arg::with_name("env")
        //     .short("e")
        //     .long("env")
        //     .value_name("ENV")
        //     .help("Specifies the environment config file to use")
        //     .takes_value(true))

        .arg(Arg::with_name("REQUEST")
            .help("The name of the request to execute.")
            .index(1))
        .subcommand(SubCommand::with_name("list")
            .about("Lists available requests"))
        .get_matches();

    let app = Reqq::new(".reqq".to_owned()).unwrap(); // maybe make this better?

    match app.run(matches) {
        Err(_) => println!("failed!"),
        Ok(_) => {},
    };
}


