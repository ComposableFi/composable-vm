use clap::*;
use cosmrs::Gas;

#[derive(Debug, Parser)]
pub struct MantisArgs {
    #[command(subcommand)]
    pub command: MantisCommands,
}

#[derive(Debug, Subcommand)]
pub enum MantisCommands {
    /// solves orders
    Solve(SolverArgs),
    Id(IdArgs),
    /// spams chain with test orders
    Simulate(SimulateArgs),
    Glt(GltArgs),
}

#[derive(Debug, Parser)]
pub struct GltArgs {
    #[command(subcommand)]
    pub command: GltCommands,
}

#[derive(Debug, Subcommand)]
pub enum GltCommands {
    // given offchain configuration, validates is
    Validate,
    // given offchain configuration and existing chains, plans apply
    // outputs offline transaction to chains provided
    Plan,
    /// adds specific things to offchain config
    Add,
}

#[derive(Debug, Parser)]
pub struct IdArgs {
    #[command(subcommand)]
    pub command: IdCommands,
}

#[derive(Debug, Subcommand)]
pub enum IdCommands {
    Asset(AssetArgs),
}

#[derive(Debug, Parser)]
pub struct AssetArgs {
    #[command(subcommand)]
    pub command: AssetCommands,
}

#[derive(Debug, Subcommand)]
pub enum AssetCommands {
    Gen { network_id: u32, asset_id: u64 },
}

#[derive(clap::Parser, Debug)]
pub struct SharedArgs {
    #[arg(long)]
    pub main_chain_id: String,

    /// the node hosting order contract
    #[arg(long)]
    pub rpc_centauri: String,

    #[arg(long)]
    pub grpc_centauri: String,

    /// Order contract on Centauri
    #[arg(long)]
    pub order_contract: String,
    /// wallet to use.
    /// For now BIP39 normalized English mnemonic empty passphrase with Kepler default derivation supported
    #[arg(long)]
    pub wallet: String,

    #[arg(long, default_value_t = 1_000_000_000)]
    pub gas: Gas,
}

#[derive(clap::Parser, Debug)]
pub struct SimulateArgs {
    #[command(flatten)]
    pub shared: SharedArgs,

    /// CVM contract on Centauri. Optional, only if consider routing via cross chain CVM.
    #[arg(long)]
    pub cvm_contract: Option<String>,

    /// tokens to send to order contract as problem
    /// format: "--coins="token1amount1,token2amount2" --coins="token2amount2,token3amount3"
    #[arg(long, value_delimiter = ' ', num_args = 1..)]
    pub coins: Vec<String>,
}

#[derive(clap::Parser, Debug)]
pub struct SolverArgs {
    #[command(flatten)]
    pub shared: SharedArgs,

    /// CVM
    #[arg(long)]
    pub cvm_contract: String,

    /// http url to call with parameters to obtain route
    pub solution_provider: String,
}

impl MantisArgs {
    pub fn parsed() -> Self {
        Self::parse()
    }
}
