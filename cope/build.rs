use cope_config::config::MacAddress;
use cope_config::config::TmpConfig;
use cope_config::file_generator::generate;
use cope_config::node_id::NodeID;
use serde::Deserialize;
use std::env;
use std::fs;
use std::str::FromStr;

#[derive(Deserialize)]
struct TOMLConfig {
    nodes: Vec<(String, String)>,
    relay: String,
    black_list: Vec<(String, Vec<String>)>,
}

fn main() {
    let cfg_path = match env::var("CONFIG_PATH") {
        Ok(path) => path,
        Err(_) => {
            println!("No CONFIG_PATH found pasing default_cfg.toml");
            "default_cfg.toml".to_string()
        }
    };
    let cfg_content = match fs::read_to_string(cfg_path) {
        Ok(file) => file,
        Err(error) => panic!("Error opening file: {:?}", error),
    };
    let toml_config: TOMLConfig = match toml::from_str(&cfg_content) {
        Ok(config) => config,
        Err(error) => panic!("Error parsing file: {:?}", error),
    };
    let nodes = toml_config
        .nodes
        .iter()
        .map(|(node, adr)| {
            (
                NodeID::from_str(node)
                    .unwrap_or_else(|e| panic!("Node ID {} is invalid: {}.", node, e)),
                MacAddress::from_str(adr)
                    .unwrap_or_else(|e| panic!("MAC Adress {} is invalid: {}.", adr, e)),
            )
        })
        .collect();
    let relay = NodeID::from_str(&toml_config.relay)
        .unwrap_or_else(|e| panic!("Node ID {} is invalid: {}.", toml_config.relay, e));
    let black_list = toml_config
        .black_list
        .iter()
        .map(|(node, list)| {
            (
                NodeID::from_str(node)
                    .unwrap_or_else(|e| panic!("Node ID {} is invalid: {}.", node, e)),
                list.iter()
                    .map(|i| {
                        NodeID::from_str(i)
                            .unwrap_or_else(|e| panic!("Node ID {} is invalid: {}.", i, e))
                    })
                    .collect(),
            )
        })
        .collect();

    let config = TmpConfig::new(nodes, relay, black_list);
    let dest_path = "src/config.rs";
    generate(&config, &dest_path.to_string());
}
