use clap::Parser;
use inquire::Select;

use nf_installer::cache::FontCache;
use nf_installer::installer::FontInstaller;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, help = "Nerd Font name")]
    font_name: Option<String>,
    #[arg(short, long, help = "Ignore cache and reload")]
    refresh: bool,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let fonts = match FontCache::get(args.refresh).await {
        Ok(fonts) => fonts,
        Err(e) => {
            eprintln!("Could not load the font list: {e}");
            std::process::exit(1);
        }
    };

    let font = match &args.font_name {
        Some(name) => fonts
            .into_iter()
            .find(|font| font.name.eq_ignore_ascii_case(name)),
        None => Select::new("Select a font", fonts).prompt().ok(),
    };

    let Some(font) = font else {
        eprintln!("Font not found");
        std::process::exit(1);
    };

    println!("Installing {}...", font.name);

    match FontInstaller::install(&font).await {
        Ok(count) => println!("Installed {} font files for {}", count, font.name),
        Err(e) => {
            eprintln!("Installation failed: {e}");
            std::process::exit(1);
        }
    }
}
