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
    #[arg(short, long, help = "Uninstall an installed Nerd Font")]
    uninstall: bool,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let result = if args.uninstall {
        run_uninstall(&args)
    } else {
        run_install(&args).await
    };

    if let Err(e) = result {
        eprintln!("{e}");
        std::process::exit(1);
    }
}

async fn run_install(args: &Args) -> Result<(), Box<dyn std::error::Error>> {
    let fonts = FontCache::get(args.refresh).await?;

    let font = match &args.font_name {
        Some(name) => fonts
            .into_iter()
            .find(|font| font.name.eq_ignore_ascii_case(name))
            .ok_or("Font not found")?,
        None => Select::new("Select a font", fonts).prompt()?,
    };

    println!("Installing {}...", font.name);
    let installed = FontInstaller::install(&font).await?;
    println!("Installed {} font files for {}", installed, font.name);

    Ok(())
}

fn run_uninstall(args: &Args) -> Result<(), Box<dyn std::error::Error>> {
    let installed = FontInstaller::installed()?;

    if installed.is_empty() {
        return Err("No installed fonts found".into());
    }

    let name = match &args.font_name {
        Some(name) => installed
            .iter()
            .find(|installed| installed.eq_ignore_ascii_case(name))
            .cloned()
            .ok_or("Font not installed")?,
        None => Select::new("Select a font to uninstall", installed).prompt()?,
    };

    println!("Uninstalling {}...", name);
    let removed = FontInstaller::uninstall(&name)?;
    println!("Removed {} font files for {}", removed, name);

    Ok(())
}
