mod commands;
mod config;
mod probe;
mod repo;
mod run;
mod ui;

use std::time::Duration;

use anyhow::Result;
use clap::{Parser, Subcommand, ValueEnum};

use config::Config;

#[derive(Parser)]
#[command(
    name = "stellaribc",
    version,
    about = "Orchestrator for the Stellar<->Cosmos IBC v2 bridge",
    long_about = "A caribic-style front door to the ci/flows scripts: bring the stack up, \
deploy contracts, upload the light client, create clients, and check status — without \
remembering individual script names.",
    propagate_version = true
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    #[command(about = "Install the stellaribc binary to the cargo bin dir (cargo install --path)")]
    Install,
    #[command(about = "Check prerequisites, configuration, and service health")]
    Doctor,
    #[command(about = "Show chain/service health, deployed contracts, and created clients")]
    Status,
    #[command(about = "Bring the stack up via docker compose (osmosis + api + gateway)")]
    Up(UpArgs),
    #[command(about = "Stop the stack via docker compose")]
    Down(DownArgs),
    #[command(
        alias = "f0",
        about = "Full bootstrap: images, chains, contracts, light-client wasm, keys (F0)"
    )]
    Bootstrap(BootstrapArgs),
    #[command(about = "Build (and push) the api / gateway / hermes docker images")]
    BuildImages(BuildImagesArgs),
    #[command(about = "Build + upload + deploy the Soroban contracts and wire the router")]
    DeployContracts(DeployContractsArgs),
    #[command(about = "Build + gov-upload the light-client-wasm to Cosmos, patch hermes config")]
    UploadWasm,
    #[command(about = "Import the hermes relayer keys (must equal the router admin key)")]
    ImportKeys,
    #[command(about = "Client lifecycle (create on each chain, list)")]
    Client {
        #[command(subcommand)]
        cmd: ClientCmd,
    },
    #[command(about = "Register counterparties (F1.3 / F1.4)")]
    Counterparty {
        #[command(subcommand)]
        cmd: CounterpartyCmd,
    },
}

#[derive(clap::Args)]
struct UpArgs {
    #[arg(long, help = "Start only the Cosmos chain (osmosis)")]
    cosmos: bool,
    #[arg(long, help = "Start only the Stellar-side services (api + gateway)")]
    stellar: bool,
}

#[derive(clap::Args)]
struct DownArgs {
    #[arg(long, help = "Also remove named volumes (wipes chain + key state)")]
    volumes: bool,
}

#[derive(clap::Args)]
struct BootstrapArgs {
    #[arg(long, help = "Skip building the docker images")]
    skip_images: bool,
    #[arg(long, help = "Skip the Soroban contract deploy")]
    skip_contracts: bool,
    #[arg(long, help = "Skip the light-client-wasm upload")]
    skip_wasm: bool,
    #[arg(long, help = "Skip importing the hermes relayer keys")]
    skip_keys: bool,
    #[arg(long, help = "Redeploy contracts even if IBC_CONTRACT_ID is already set")]
    force_redeploy: bool,
}

#[derive(clap::Args)]
struct BuildImagesArgs {
    #[arg(value_enum, default_value_t = ImageTarget::All, help = "Which image to build")]
    target: ImageTarget,
}

#[derive(Clone, Copy, ValueEnum)]
enum ImageTarget {
    All,
    Api,
    Gateway,
    Hermes,
}

impl ImageTarget {
    fn as_str(self) -> &'static str {
        match self {
            Self::All => "all",
            Self::Api => "api",
            Self::Gateway => "gateway",
            Self::Hermes => "hermes",
        }
    }
}

#[derive(clap::Args)]
struct DeployContractsArgs {
    #[arg(long, help = "Redeploy even if IBC_CONTRACT_ID is already set")]
    force: bool,
}

#[derive(Subcommand)]
enum ClientCmd {
    #[command(
        alias = "cosmos",
        about = "Create the Cosmos (Tendermint) client on Stellar (F1.1)"
    )]
    CreateCosmos {
        #[arg(long, help = "Use the CLI encode->invoke path instead of the relayer")]
        cli: bool,
    },
    #[command(
        alias = "stellar",
        about = "Create the Stellar (08-wasm) client on Cosmos (F1.2)"
    )]
    CreateStellar {
        #[arg(long, help = "Create a new client even if STELLAR_CLIENT_ID is already set")]
        force: bool,
    },
    #[command(about = "List clients created on the Stellar router")]
    List,
}

#[derive(Subcommand)]
enum CounterpartyCmd {
    #[command(about = "Register the Cosmos client as counterparty on Stellar (F1.3)")]
    Stellar,
    #[command(about = "Register the Stellar client as counterparty on Cosmos (F1.4)")]
    Cosmos,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let root = repo::find_root()?;
    let cfg = Config::load(&root);
    let http = reqwest::Client::builder()
        .timeout(Duration::from_secs(8))
        .build()?;

    match cli.command {
        Command::Install => commands::install(&root)?,
        Command::Doctor => commands::doctor(&root, &cfg, &http).await?,
        Command::Status => commands::status(&root, &cfg, &http).await?,
        Command::Up(args) => commands::up(&root, args.cosmos, args.stellar)?,
        Command::Down(args) => commands::down(&root, args.volumes)?,
        Command::Bootstrap(args) => commands::bootstrap(
            &root,
            args.skip_images,
            args.skip_contracts,
            args.skip_wasm,
            args.skip_keys,
            args.force_redeploy,
        )?,
        Command::BuildImages(args) => commands::build_images(&root, args.target.as_str())?,
        Command::DeployContracts(args) => commands::deploy_contracts(&root, args.force)?,
        Command::UploadWasm => commands::upload_wasm(&root)?,
        Command::ImportKeys => commands::import_keys(&root)?,
        Command::Client { cmd } => match cmd {
            ClientCmd::CreateCosmos { cli } => commands::client_create_cosmos(&root, cli)?,
            ClientCmd::CreateStellar { force } => commands::client_create_stellar(&root, force)?,
            ClientCmd::List => commands::client_list(&cfg, &http).await?,
        },
        Command::Counterparty { cmd } => match cmd {
            CounterpartyCmd::Stellar => commands::counterparty(&root, "stellar")?,
            CounterpartyCmd::Cosmos => commands::counterparty(&root, "cosmos")?,
        },
    }

    Ok(())
}
