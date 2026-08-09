use clap::Parser;
use inquire::{error::InquireError, Select};

use nf_installer::font_scraper::FontScraper;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, help = "Nerd Font name")]
    font_name: Option<String>
}
#[tokio::main]
async fn main() {
    let args = Args::parse();

    if let Some(font_name) = args.font_name {
        println!("Font name: {}", font_name);
    } else {
        let fonts = FontScraper::get_font_names().await.unwrap();
        let options: Vec<&str> = fonts.iter().map(|font| font.name.as_str()).collect();
        let ans: Result<&str, InquireError> = Select::new("Select a font", options)
            .prompt();
        match ans {
            Ok(choice) => println!("You selected {} Nerd Font!", choice),
            Err(_) => println!("There was an error, please try again"),
        }
    }
}
