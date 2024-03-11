use clap::*;
use cosmrs::Gas;

#[derive(Debug, Parser)]
pub struct MantisArgs {
    #[command(subcommand)]
    pub command: MantisCommands,
}

#[derive(Debug, Subcommand)]
pub enum MantisCommands {
    Solve(SolverArgs),
    Id(IdArgs),
    Simulate(SimulateArgs),
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
    #[arg(flatten)]
    pub shared: SharedArgs,
    
    /// CVM contract on Centauri. Optional, only if consider routing via cross chain CVM.
    #[arg(long)]
    pub cvm_contract: Option<String>,

    /// tokens to send to order contract as problem
    /// format: "token1amount1,token2amount2"
    #[arg(long)]
    pub pair: Option<String>,


    /// the problem to solve
    #[arg(long, default_value_t = 10)]
    pub simulate_period_seconds: u16,
}

#[derive(clap::Parser, Debug)]
pub struct SolverArgs {
    #[arg(flatten)]
    pub shared: SharedArgs,

    /// the problem to solve
    #[arg(long, default_value_t = 10)]
    pub solve_period_seconds: u16,
}

impl MantisArgs {
    pub fn parsed() -> Self {
        use clap::Parser;

        Self::parse()
    }
}
