use cope_config::config::Config;
use cope_config::config::MacAdress;
use cope_config::config::NodeID;
use cope_config::file_generator::generate;
use serde::Deserialize;
use std::env;
use std::fs;

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
        .map(|(node, adr)| (NodeID::from_string(node), MacAdress::from_string(adr)))
        .collect();
    let relay = NodeID::from_string(&toml_config.relay);
    let black_list = toml_config
        .black_list
        .iter()
        .map(|(node, list)| {
            (
                NodeID::from_string(node),
                list.iter().map(|i| NodeID::from_string(i)).collect(),
            )
        })
        .collect();

    let config = Config::new(nodes, relay, black_list);
    let dest_path = "src/config.rs";
    generate(&config, &dest_path.to_string());
}
