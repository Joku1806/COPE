use cope_config::config::TmpConfig;
use cope_config::file_generator::generate;
use cope_config::types::mac_address::MacAddress;
use cope_config::types::node_id::NodeID;
use cope_config::types::traffic_generator_type::TrafficGeneratorType;
use parse_duration;
use serde::Deserialize;
use std::env;
use std::fs;
use std::str::FromStr;
use std::time::Duration;

#[derive(Deserialize)]
struct TOMLConfig {
    nodes: Vec<(String, String)>,
    relay: String,
    rx_whitelist: Vec<(String, Vec<String>)>,
    tx_whitelist: Vec<(String, Vec<String>)>,
    traffic_generators: Vec<(String, String)>,
    simulator_packet_loss: f64,
    round_trip_time: String,
    packet_pool_size: usize,
    control_packet_duration: String,
    max_retrans_amount: u8,
    use_coding: bool,
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

    let rx_whitelist = toml_config
        .rx_whitelist
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

    let tx_whitelist = toml_config
        .tx_whitelist
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

    let traffic_generators = toml_config
        .traffic_generators
        .iter()
        .map(|(node, tgt)| {
            (
                NodeID::from_str(node)
                    .unwrap_or_else(|e| panic!("Node ID {} is invalid: {}.", node, e)),
                TrafficGeneratorType::from_str(tgt).unwrap_or_else(|e| {
                    panic!("Traffic Generator Type {} is invalid: {}.", tgt, e)
                }),
            )
        })
        .collect();
    let simulator_packet_loss = toml_config.simulator_packet_loss;
    let round_trip_time = parse_duration::parse(&toml_config.round_trip_time).unwrap();
    let packet_pool_size = toml_config.packet_pool_size;
    let control_packet_duration =
        parse_duration::parse(&toml_config.control_packet_duration).unwrap();
    let max_retrans_amount = toml_config.max_retrans_amount;
    let use_coding = toml_config.use_coding;

    let config = TmpConfig::new(
        nodes,
        relay,
        rx_whitelist,
        tx_whitelist,
        traffic_generators,
        simulator_packet_loss,
        round_trip_time,
        packet_pool_size,
        control_packet_duration,
        max_retrans_amount,
        use_coding,
    );
    let dest_path = "src/config.rs";
    generate(&config, &dest_path.to_string());
}
