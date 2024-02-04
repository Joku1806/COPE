// The file generator generates a file containing a const instance of Config
// User has to provide a instance of TmpConfig that can be written to file
// User can specify the location of the output file

use std::fmt::format;
use crate::config::TmpConfig;
use crate::types::mac_address::MacAddress;
use crate::types::node_id::NodeID;
use crate::types::traffic_generator_type::TrafficGeneratorType;
use std::fs;
use std::io::Write;

pub fn generate(config: &TmpConfig, path: &String) {
    let mut file = fs::File::create(path).unwrap();
    let node_count = config.nodes().len();
    let simulator_packet_loss = config.simulator_packet_loss;
    // TODO: check for correcness of input
    // TODO: remove all the unwraps
    writeln!(
        file,
        "// This file is auto generated by a build.rs file and cope_config"
    )
    .unwrap();
    writeln!(file, "use cope_config::config::*;").unwrap();
    writeln!(file, "use cope_config::types::node_id::NodeID;").unwrap();
    writeln!(file, "use cope_config::types::mac_address::MacAddress;").unwrap();
    writeln!(
        file,
        "use cope_config::types::traffic_generator_type::TrafficGeneratorType;"
    )
    .unwrap();
    writeln!(file, "use std::time::Duration;\n").unwrap();
    writeln!(file, "pub const CONFIG: Config<{}> = Config{{", node_count).unwrap();
    writeln!(file, "    simulator_packet_loss: {:.3},", config.simulator_packet_loss).unwrap();
    write_nodes(&mut file, config);
    write_relay(&mut file, config);
    write_whitelist(&mut file, config, node_count, "rx_whitelist");
    write_whitelist(&mut file, config, node_count, "tx_whitelist");
    write_traffic_generators(&mut file, config);
    writeln!(file, "}};").unwrap();
}

fn write_nodes(file: &mut fs::File, config: &TmpConfig) {
    let mut nodes = String::new();
    nodes.push_str("[\n");
    for (n, m) in config.nodes() {
        let node = node_id_to_string(n);
        let mac = mac_adr_to_string(m);
        nodes.push_str(&format!("        ({}, {}),\n", node, mac));
    }
    nodes.push_str("    ]");
    writeln!(file, "    nodes: {},", nodes).unwrap();
}

fn write_relay(file: &mut fs::File, config: &TmpConfig) {
    let relay = node_id_to_string(&config.relay());
    writeln!(file, "    relay: {},", relay).unwrap();
}

fn write_whitelist(file: &mut fs::File, config: &TmpConfig, node_count: usize, key: &str) {
    let source = match key {
        "rx_whitelist" => config.rx_whitelist(),
        "tx_whitelist" => config.tx_whitelist(),
        _ => panic!("Invalid key {}", key),
    };

    let mut s = String::new();
    s.push_str("[\n");
    for (node, _) in config.nodes() {
        let node_id = node_id_to_string(node);
        let list = match source.iter().find(|(n, _)| *n == *node).map(|(_, l)| l) {
            Some(res) => res,
            None => panic!("Did not find Node {} in {}", node_id, key),
        };
        let bl_atr = node_list_to_string(list, node_count);
        s.push_str(&format!("        ({}, {}),\n", node_id, bl_atr));
    }
    s.push_str("    ]");
    writeln!(file, "    {}: {},", key, s).unwrap();
}

fn write_traffic_generators(file: &mut fs::File, config: &TmpConfig) {
    let mut tgs = String::new();
    tgs.push_str("[\n");
    for (n, t) in config.traffic_generators() {
        println!("{}, {}", n, t);
        let node = node_id_to_string(n);
        let tgt = tgt_to_string(t);
        tgs.push_str(&format!("        ({}, {}),\n", node, tgt));
    }
    tgs.push_str("    ]");
    writeln!(file, "    traffic_generators: {},", tgs).unwrap();
}

fn node_id_to_string(node_id: &NodeID) -> String {
    return format!("NodeID::new('{}')", node_id.to_string());
}

fn mac_adr_to_string(mac_adr: &MacAddress) -> String {
    let bytes = mac_adr.as_bytes();
    return format!(
        "MacAddress::new({}, {}, {}, {}, {}, {})",
        bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5]
    );
}

fn tgt_to_string(tgt: &TrafficGeneratorType) -> String {
    let serialized = match tgt {
        TrafficGeneratorType::None => "None".into(),
        TrafficGeneratorType::Greedy => "Greedy".into(),
        // NOTE: Is it a problem if we lose precision here?
        TrafficGeneratorType::Poisson(m) => format!("Poisson({:#})", m),
        TrafficGeneratorType::Random(m) => format!("Random({:#})", m),
        TrafficGeneratorType::Periodic(p) => format!(
            "Periodic(Duration::new({}, {}))",
            p.as_secs(),
            p.subsec_nanos()
        ),
    };

    return format!("TrafficGeneratorType::{}", serialized);
}

fn node_list_to_string(list: &Vec<NodeID>, node_count: usize) -> String {
    let mut str = String::new();
    str.push_str("[\n            ");

    for n in list {
        let node = node_id_to_string(n);
        str.push_str(&format!("Some({}),\n            ", node));
    }

    assert_eq!(true, node_count >= list.len());
    for _ in 0..(node_count - list.len()) {
        str.push_str("None, ");
    }
    str.push_str("\n        ]");
    str
}
