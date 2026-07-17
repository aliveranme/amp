use clap::Parser;

use amp_core::AppConfig;

#[derive(Parser)]
#[command(name = "amp-code", version, about = "BYOK LLM proxy CLI")]
struct Cli {
    /// Start the API server
    #[arg(long)]
    server: bool,

    /// Proxy host
    #[arg(long, default_value = "127.0.0.1")]
    host: String,

    /// Proxy port
    #[arg(long, default_value = "8080")]
    port: u16,

    /// Database path
    #[arg(long, default_value = "amp-code.db")]
    db: String,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    if cli.server {
        tracing::info!("Starting amp-code server on {}:{}", cli.host, cli.port);
        let mut config = AppConfig::load();
        config.host = cli.host;
        config.port = cli.port;
        config.db_path = cli.db;
        amp_server::serve(config).await;
    } else {
        println!("amp-code BYOK CLI");
        println!("Run with --server to start the proxy server");
    }
}
