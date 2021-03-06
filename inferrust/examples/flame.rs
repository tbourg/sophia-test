use inferrust::inferray::*;
use inferrust::rules::*;

use time::precise_time_ns;

use std::fs;

fn main() {
    let profile = &mut RuleProfile::RDFSPlus();
    let path = "inferrust/res/bsbm/10k.nt";
    let f = std::io::BufReader::new(std::fs::File::open(path).unwrap());
    let t0 = precise_time_ns();
    let ts = sophia::parser::nt::parse_bufread(f);
    let mut i_graph = InfGraph::from(ts);
    println!("graph size: {}", i_graph.size());
    let t1 = precise_time_ns();
    i_graph.process(profile);
    let t2 = precise_time_ns();
    let load_time = (t1 - t0) as f64 / 1e9;
    let process_time = (t2 - t1) as f64 / 1e9;
    println!(
        "profile: {}, graph size: {}, ltime: {}, ptime: {}",
        profile.name(),
        i_graph.size(),
        load_time,
        process_time,
    );
}
