//! transforms CVM program into analyzed format
//!
//! Cases:
//! 1. Centauri -> transfer -> Cosmos Hub => converted to usual IBC transfer
//! 2. Centauri -> transfer -> Cosmos Hub -> transfer -> Osmosis => PFM enabled transfer
//! 3. Centauri -> transfer -> Cosmos Hub(local CVM op) -> transfer -> Osmosis => unroutable
//! 4. Centauri -> transfer -> Cosmos Hub -> transfer -> Osmosis (swap) => PFM enabled transfer with CVM wasm hook
//! 5. Centauri -> transfer -> Cosmos Hub -> transfer -> Osmosis (swap) -> transfer -> Neutron => PFM enabled transfer with CVM wasm hook and after usual transfer
//! 6. Centauri -> transfer -> Cosmos Hub -> transfer -> Osmosis (swap) -> transfer -> Neutron(swap) => PFM enabled transfer with CVM wasm hook and after one more CVM hop
