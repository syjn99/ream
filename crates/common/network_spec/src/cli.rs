use std::{fs, sync::Arc};

use crate::networks::{BeaconNetworkSpec, DEV, HOLESKY, HOODI, MAINNET, SEPOLIA};

pub fn beacon_network_parser(network_string: &str) -> Result<Arc<BeaconNetworkSpec>, String> {
    match network_string {
        "mainnet" => Ok(MAINNET.clone()),
        "holesky" => Ok(HOLESKY.clone()),
        "sepolia" => Ok(SEPOLIA.clone()),
        "hoodi" => Ok(HOODI.clone()),
        "dev" => Ok(DEV.clone()),
        _ => {
            let contents = fs::read_to_string(network_string)
                .map_err(|err| format!("Failed to read file: {err}"))?;
            Ok(Arc::new(serde_yaml::from_str(&contents).map_err(
                |err| format!("Failed to parse YAML from: {err}"),
            )?))
        }
    }
}
