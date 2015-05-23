extern crate hyper;
extern crate crypto;
extern crate rustc_serialize;
extern crate xml;
extern crate itertools;

mod cipher;
mod bus;
mod api;

use std::str::FromStr;
use rustc_serialize::json;

use bus::{BusLine, BusStation, LatLng};
use itertools::Itertools;
use api::BeijingBusApi;

use std::io::prelude::*;
use std::fs::File;
use std::io::Result;

fn main1() {
    // Create a client.
    let api = BeijingBusApi::new();
    let line_id_versions = api.update_id_versions().unwrap();

    println!("got ids => {:?}", line_id_versions);

    let mut lines = Vec::new();
    // let line = api.get_busline_info(369).unwrap();
    // println!("line => {}", line);

    // let ret = json::encode(&line);
    // println!("got str => {}", ret.unwrap());


    // for station in line.stations.iter() {
    //     println!("{}", station);
    // }
    //     println!("line => {}", line);
    for &(id, _) in line_id_versions.iter() {
        let line = api.get_busline_info(id).unwrap();
        println!("line => {}", line);
        lines.push(line)
    }

    let ret = json::encode(&lines);
    // let ret = json::as_pretty_json(&lines);
    let mut f = File::create("data.json").unwrap();
    write!(f, "{}", ret.unwrap());
}



fn run() -> Result<()> {
    let mut f = File::open("data.json").unwrap();
    let mut buf = String::new();
    try!(f.read_to_string(&mut buf));
    let lines: Vec<BusLine> = json::decode(&buf).unwrap();

    for line in lines.iter() {
        //println!("line -> {}", line);

    }
    let api = BeijingBusApi::new();

    let line_id = 795;

    //let ret = api.get_busline_info(87).unwrap();
    let ret = api.get_busline_info(795).unwrap();
    println!("line -> {}", ret);
    // let ret = api.get_realtime_busline_info(369);
    let ret = api.get_realtime_busline_info(795, 2000).unwrap();
    println!("------{:?}", ret);
    ret.iter().map(|b| b.describ()).count();


    Ok(())
}


fn main() {
    // main1();
    match run() {
        _ => ()
    }
}

#[test]
fn it_works() {
}
