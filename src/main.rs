use nf_installer::Cli;

#[tokio::main]
async fn main() {
    if let Err(e) = Cli::run().await {
        eprintln!("{e}");
        std::process::exit(1);
    }
}
