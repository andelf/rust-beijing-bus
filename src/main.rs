extern crate hyper;
extern crate crypto;
extern crate rustc_serialize;
extern crate xml;
extern crate itertools;
#[macro_use(to_sql_checked, accepts)]
extern crate postgres;
extern crate postgis;

mod cipher;
mod bus;
mod api;


use std::str::FromStr;
use std::error::Error;
use rustc_serialize::json;

use bus::{BusLine, BusStation, LatLng};
use itertools::Itertools;
use api::BeijingBusApi;

use std::io::prelude::*;
use std::fs::File;
use std::io::Result;
use postgres::{Connection, SslMode};
use postgis::{Point, LineString};

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


static create_bus_station_table: &'static str = "
CREATE TABLE bus_station (
    id              serial PRIMARY KEY,
    coords          geography(POINT, 4326),
    name            text,
    line_id         integer,
    no              integer
)
";

/// r##"  "##;


fn run() -> Result<()> {
    let mut f = File::open("data.json").unwrap();
    let mut buf = String::new();
    try!(f.read_to_string(&mut buf));
    let lines: Vec<BusLine> = json::decode(&buf).unwrap();

    for line in lines.iter() {
        //println!("line -> {}", line);

    }
    let api = BeijingBusApi::new();

    let line_id = 369;

    let conn = Connection::connect("postgresql://mmgis@192.168.1.38/beijingbus", &SslMode::None)
        .unwrap();
/*
    let ret = conn.execute("CREATE TABLE busline (
                    id              integer PRIMARY KEY,
                    version         integer,
                    short_name      varchar(10),
                    long_name       text,
                    operation_time  varchar(20),
                    route           geography(LINESTRING,4326)
                  )", &[]);
    println!("create table => {:?}", ret);
     */
    // let ret = conn.execute(create_bus_station_table, &[]);
    // println!("create table => {:?}", ret);
    //let ret = api.get_busline_info(87).unwrap();
    let line = api.get_busline_info(line_id).unwrap();
    println!("line -> {}", line);

    let ret = conn.execute("INSERT INTO busline (id, version, short_name, long_name, operation_time, route) VALUES ($1, $2, $3, $4, $5, $6)",
                           &[&34, &22, &"116", &"116(城铁柳芳站-龙潭公园)", &"5:00-24:00",
                             &{
                                 let mut route = LineString::new();
                                 route.points.push(Point::new(123.0, 49.0));
                                 route.points.push(Point::new(133.0, 59.0));
                                 route.points.push(Point::new(153.0, 80.0));
                                 route
                             }]);
    // let ret = conn.execute("INSERT INTO busline (id, version, short_name, long_name, operation_time) VALUES ($1, $2, $3, $4, $5)",
    //                        &[&87, &21, &"116", &"116(城铁柳芳站-龙潭公园)", &"5:00-24:00",]);


    println!("insert  => {:?}", ret);

    // let ret = conn.execute("INSERT INTO bus_station (coords, name, line_id, no) VALUES ($1, $2, $3, $4)",
    //                        &[
    //                            &{
    //                                let mut pt = Point::new();
    //                                pt
    //                            },
    //                            &"通州小街桥东", &233, &23]);


    // println!("insert  => {:?}", ret);

    let stmt = conn.prepare("SELECT * FROM busline").unwrap();
    //let stmt = conn.prepare("SELECT * FROM bus_station").unwrap();
    for row in stmt.query(&[]).unwrap() {
        // println!("row => {:?}", row);
        // println!(">>>>>> {:?}", row.get_bytes("route"));
        println!(">>>>>> {}", row.get::<_, LineString<Point>>("route"));

        //println!(">>>>>> {:?}", row.get::<_, postgis::Point>("coords"));
    }

    // let ret = api.get_realtime_busline_info(369);
    let ret = try!(api.get_realtime_busline_info(line_id, 2000));
    println!("------{:?}", ret);
    ret.iter().map(|b| b.describ()).count();


    Ok(())
}


fn main() {
    // main1();
    match run() {
        Err(e) => {
            println!("error: {}", e.description());
        }
        _ => ()
    }
}

#[test]
fn it_works() {
}
