#[macro_use]
extern crate serde;
extern crate reqwest;

use getopts::Options;
use reqwest::Error;

const API_URL: &str = "https://api.nibl.co.uk/nibl";

fn print_usage(program: &str, opts: Options) {
    let msg = format!(
        "Usage: {} -b BOT -p PACKAGE1[,PACKAGE2,...] [options]",
        program
    );
    print!("{}", opts.usage(&msg));
}

fn main() -> Result<(), Error> {
    let args: Vec<String> = std::env::args().collect();
    let program = args[0].clone();
    let mut opts = Options::new();
    opts.reqopt("b", "query", "Query to run", "QUERY")
        .optopt("e", "episode", "Episode number", "NUMBER")
        .optflag("h", "help", "print this help menu");

    // Unfortunately, cannot use getopts to check for a single optional flag
    // https://github.com/rust-lang-nursery/getopts/issues/46
    if args.contains(&"-h".to_string()) || args.contains(&"--help".to_string()) {
        print_usage(&program, opts);
        std::process::exit(0);
    }

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(error) => {
            eprintln!("{}.", error);
            eprintln!("{}", opts.short_usage(&program));
            std::process::exit(1);
        }
    };

    let query = matches.opt_str("q").expect("Query must be specified.");
    let episode: Option<String> = matches.opt_str("q");

    let mut response = reqwest::get(&format!("{}/bots", API_URL))?;
    let bot_list: BotList = response.json()?;
    if bot_list.status != "OK" {
        panic!("Could not fetch bot list: {}", bot_list.message);
    }
    let bots = bot_list.content;
    let mut search_url = format!("{}/search?query={}", API_URL, query);
    if episode.is_some() {
        search_url += &format!("&episode={}", episode.unwrap());
    }
    response = reqwest::get(&search_url)?;
    let search_result: SearchResult = response.json()?;
    if search_result.status != "OK" {
        panic!("Could not search package: {}", search_result.message);
    }

    let packages = search_result.content;
    let first_package = packages.first().unwrap();
    let bot: &Bot = bots
        .iter()
        .find(|bot| bot.id == first_package.bot_id)
        .unwrap();

    println!("Package: {} - {}", bot.name, first_package.number);

    Ok(())
}

#[derive(Deserialize)]
struct BotList {
    status: String,
    message: String,
    content: Vec<Bot>,
}

#[derive(Deserialize)]
struct Bot {
    id: i64,
    name: String,
}

#[derive(Deserialize)]
struct SearchResult {
    status: String,
    message: String,
    content: Vec<Package>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Package {
    bot_id: i64,
    number: i32,
}
