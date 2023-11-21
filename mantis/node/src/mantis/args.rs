#[derive(clap::Parser, Debug)]
pub struct MantisArgs {
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

impl MantisArgs {
    pub fn parsed() -> Self {
        use clap::Parser;

        Self::parse()
    }
}
