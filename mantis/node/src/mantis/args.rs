use clap::*;

#[derive(Debug, Parser)]
pub struct MantisArgs {
    #[command(subcommand)]
    command: MantisCommands,
}

#[derive(Debug, Subcommand)]
pub enum MantisCommands {
    Solve(SolverArgs),
    Id(IdArgs),
}

#[derive(Debug, Parser)]
pub struct IdArgs {
    #[command(subcommand)]
    command: IdCommands,
}

#[derive(Debug, Subcommand)]
pub enum IdCommands {
    Asset(AssetArgs),
}

#[derive(Debug, Parser)]
pub struct AssetArgs {
    #[command(subcommand)]
    command: AssetCommands,
}

#[derive(Debug, Subcommand)]
pub enum AssetCommands {
    Get { network_id: u32, asset_id: u64 },
}

#[derive(clap::Parser, Debug)]
pub struct SolverArgs {
    /// the node hosting order contract
    #[arg(long)]
    pub rpc_centauri: String,

    #[arg(long)]
    pub grpc_centauri: String,

    /// the node with pools
    #[arg(long)]
    pub osmosis: String,

    /// the node with pools
    #[arg(long)]
    pub neutron: String,

    /// CVM contract on Centauri
    #[arg(long)]
    pub cvm_contract: String,

    /// Order contract on Centauri
    #[arg(long)]
    pub order_contract: String,

    /// tokens to send to order contract as problem
    /// format: "token1amount1,token2amount2"
    #[arg(long)]
    pub simulate: Option<String>,

    /// wallet to use
    #[arg(long)]
    pub wallet: String,
}

impl SolverArgs {
    pub fn parsed() -> Self {
        use clap::Parser;

        Self::parse()
    }
}
