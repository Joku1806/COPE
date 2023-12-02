// The file generator generates a file containing a const instance of Config
// User has to provide a instance of TmpConfig that can be written to file
// User can specify the location of the output file

use crate::config::{MacAddress, TmpConfig};
use crate::node_id::NodeID;
use std::fs;
use std::io::Write;

pub fn generate(config: &TmpConfig, path: &String) {
    let mut file = fs::File::create(path).unwrap();
    let node_count = config.nodes().len();
    // TODO: check for correcness of input
    // TODO: remove all the unwraps
    writeln!(
        file,
        "// This file is auto generated by a build.rs file and cope_config"
    )
    .unwrap();
    writeln!(file, "use cope_config::config::*;").unwrap();
    writeln!(file, "use cope_config::node_id::NodeID;\n").unwrap();
    writeln!(file, "pub const CONFIG: Config<{}> = Config{{", node_count).unwrap();
    write_nodes(&mut file, config);
    write_relay(&mut file, config);
    write_whitelist(&mut file, config, node_count, "rx_whitelist");
    write_whitelist(&mut file, config, node_count, "tx_whitelist");
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
