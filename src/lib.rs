pub mod core;
pub mod utils;

use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};
use inquire::Select;

use crate::core::installer::FontInstaller;
use crate::core::sources::FontSource;
use crate::core::sources::nerd_fonts::NerdFonts;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Install a Nerd Font
    Nerd(NerdArgs),
    /// Install a Google Font
    Google(GoogleArgs),
    /// Install fonts from a local archive or font file
    Local(LocalArgs),
    /// List the installed fonts
    List,
    /// Uninstall an installed font
    Uninstall(UninstallArgs),
}

#[derive(Args, Debug)]
struct NerdArgs {
    #[arg(short, long, help = "Nerd Font name")]
    font_name: Option<String>,
    #[arg(short, long, help = "Ignore cache and reload")]
    refresh: bool,
}

#[derive(Args, Debug)]
struct GoogleArgs {
    #[arg(short, long, help = "Font family name")]
    font_name: Option<String>,
    #[arg(short, long, help = "Ignore cache and reload")]
    refresh: bool,
    #[arg(
        short,
        long,
        help = "Write web assets into a project instead of installing"
    )]
    web: bool,
    #[arg(short, long, help = "Output directory for web assets")]
    out: Option<PathBuf>,
    #[arg(
        long,
        value_delimiter = ',',
        help = "Weights to fetch, comma separated"
    )]
    weights: Option<Vec<String>>,
}

#[derive(Args, Debug)]
struct LocalArgs {
    #[arg(help = "Path to an archive or font file")]
    path: PathBuf,
}

#[derive(Args, Debug)]
struct UninstallArgs {
    #[arg(short, long, help = "Font name")]
    font_name: Option<String>,
}

impl Cli {
    /// Parses the arguments and runs the selected command.
    pub async fn run() -> Result<(), Box<dyn std::error::Error>> {
        let cli = Self::parse();

        let command = match cli.command {
            Some(command) => command,
            None => Self::pick_command()?,
        };

        match command {
            Command::Nerd(args) => Self::nerd(&args).await,
            Command::Google(args) => Self::google(&args).await,
            Command::Local(args) => Self::local(&args),
            Command::List => Self::list(),
            Command::Uninstall(args) => Self::uninstall(&args),
        }
    }

    /// Asks what to do when the tool is started without a subcommand.
    fn pick_command() -> Result<Command, Box<dyn std::error::Error>> {
        let choice = Select::new(
            "What do you want to do?",
            vec!["Nerd Font", "Google Font", "Uninstall"],
        )
        .prompt()?;

        let command = match choice {
            "Nerd Font" => Command::Nerd(NerdArgs {
                font_name: None,
                refresh: false,
            }),
            "Google Font" => Command::Google(GoogleArgs {
                font_name: None,
                refresh: false,
                web: false,
                out: None,
                weights: None,
            }),
            _ => Command::Uninstall(UninstallArgs { font_name: None }),
        };

        Ok(command)
    }

    /// Picks a Nerd Font from the available list and installs it.
    async fn nerd(args: &NerdArgs) -> Result<(), Box<dyn std::error::Error>> {
        let source = NerdFonts;
        let fonts = source.list(args.refresh).await?;

        let font = match &args.font_name {
            Some(name) => fonts
                .into_iter()
                .find(|font| font.name.eq_ignore_ascii_case(name))
                .ok_or("Font not found")?,
            None => Select::new("Select a font", fonts).prompt()?,
        };

        println!("Installing {}...", font.name);
        let archive = source.fetch(&font).await?;
        let installed = FontInstaller::install(&font.name, &archive)?;
        println!("Installed {} font files for {}", installed, font.name);

        Ok(())
    }

    /// Fetches a Google Font, either into the system or into a project directory.
    async fn google(_args: &GoogleArgs) -> Result<(), Box<dyn std::error::Error>> {
        Err("Google Fonts are not implemented yet".into())
    }

    /// Installs fonts from a local archive or font file.
    fn local(_args: &LocalArgs) -> Result<(), Box<dyn std::error::Error>> {
        Err("Local installation is not implemented yet".into())
    }

    /// Prints the installed fonts.
    fn list() -> Result<(), Box<dyn std::error::Error>> {
        let installed = FontInstaller::installed()?;

        if installed.is_empty() {
            println!("No installed fonts found");
            return Ok(());
        }

        for name in installed {
            println!("{name}");
        }

        Ok(())
    }

    /// Picks a font from the installed ones and removes it.
    fn uninstall(args: &UninstallArgs) -> Result<(), Box<dyn std::error::Error>> {
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
}
