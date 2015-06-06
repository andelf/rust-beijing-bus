#![feature(scoped, collections, test)]
#![feature(custom_derive, plugin)]
#![plugin(tojson_macros)]

extern crate hyper;
extern crate crypto;
extern crate rustc_serialize;
extern crate xml;
extern crate itertools;
#[macro_use(to_sql_checked, accepts)]
extern crate postgres;
extern crate postgis;
extern crate time;
extern crate r2d2;
extern crate r2d2_postgres;
extern crate rand;
#[cfg(test)]
extern crate test;

mod cipher;
mod bus;
mod api;

use std::str::FromStr;
use std::error::Error;
use std::iter::FromIterator;
use std::thread;
use std::io;
use std::io::prelude::*;
use std::fs::File;
use std::io::Result;
use std::sync::Arc;
use std::sync::mpsc::channel;

use rustc_serialize::json;
use bus::{BusLine, BusStation, LatLng};
use itertools::Itertools;
use api::BeijingBusApi;

use postgres::{Connection, SslMode};
use postgis::{NoSRID, Point, LineString, WGS84};
use postgis::mars;

use r2d2_postgres::PostgresConnectionManager;

use rand::Rand;
use rand::distributions::{IndependentSample, Range};

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
    //let line_id = 983;

    //let conn = Connection::connect("postgresql://mmgis@192.168.1.38/beijingbus", &SslMode::None).unwrap();



//     let ret = conn.execute("CREATE TABLE realtime_bus (

//   )

// ")
    // let ret = conn.execute(create_bus_station_table, &[]);
    // println!("create table => {:?}", ret);
    //let ret = api.get_busline_info(87).unwrap();
    let line = api.get_busline_info(line_id).unwrap();
    println!("line -> {}", line);

    // let ret = conn.execute("INSERT INTO busline (id, version, short_name, long_name, operation_time, route) VALUES ($1, $2, $3, $4, $5, $6)",
    //                        &[&332, &22, &"116", &"116(城铁柳芳站-龙潭公园)", &"5:00-24:00",
    //                          &LineString::from_iter(vec![
    //                              Point::<WGS84>::new(123.0, 49.0), Point::new(133.0, 59.0), Point::new(153.0, 80.0)])]);
    // // let ret = conn.execute("INSERT INTO busline (id, version, short_name, long_name, operation_time) VALUES ($1, $2, $3, $4, $5)",
    // //                        &[&87, &21, &"116", &"116(城铁柳芳站-龙潭公园)", &"5:00-24:00",]);


    // println!("insert  => {:?}", ret);

    // let ret = conn.execute("INSERT INTO bus_station (coords, name, line_id, no) VALUES ($1, $2, $3, $4)",
    //                        &[
    //                            &{
    //                                let mut pt = Point::new();
    //                                pt
    //                            },
    //                            &"通州小街桥东", &233, &23]);


    // println!("insert  => {:?}", ret);

    // let stmt = conn.prepare("SELECT * FROM busline where id = 107").unwrap();
    // //let stmt = conn.prepare("SELECT * FROM bus_station").unwrap();
    // for row in stmt.query(&[]).unwrap() {
    //     // println!("row => {:?}", row);
    //     // println!(">>>>>> {:?}", row.get_bytes("route"));
    //     println!(">>>>>> {}", row.get::<_, LineString<Point>>("route"));

    //     //println!(">>>>>> {:?}", row.get::<_, postgis::Point>("coords"));
    // }

    // let ret = api.get_realtime_busline_info(369);
    let ret = try!(api.get_realtime_busline_info(line_id, 2000));
    println!("------{:?}", ret);
    ret.iter().map(|b| {
        b.describ();
        print!("mars => {} ", b.coords);
        println!("earth => {:?}", mars::to_wgs84(b.coords.lng as f64, b.coords.lat as f64));
        println!("nsrt {} nst {} ---- {}", b.next_station_run_time, b.next_station_time, b.next_station_time as i64 - b.gps_update_time as i64);
    }).count();


    Ok(())
}

fn insert_db() -> Result<()> {
    let mut f = File::open("data.json").unwrap();
    let mut buf = String::new();
    try!(f.read_to_string(&mut buf));
    let lines: Vec<BusLine> = json::decode(&buf).unwrap();

    //let connection_str = "postgresql://wangshuyu@127.0.0.1/beijingbus";
    let connection_str = "postgresql://mmgis@192.168.1.38/beijingbus";

    let conn = Connection::connect(connection_str, &SslMode::None).unwrap();

    let ret = conn.execute("CREATE TABLE IF NOT EXISTS busline (
                    id              integer PRIMARY KEY,
                    version         integer,
                    short_name      varchar(10),
                    long_name       text,
                    operation_time  varchar(30),
                    route           geography(LINESTRING,4326)
                  )", &[]);
    let ret = conn.execute("CREATE TABLE IF NOT EXISTS bus_station (
                    id              serial PRIMARY KEY,
                    line_id         integer,
                    no              integer,
                    name            text,
                    coords          geography(POINT,4326)
                  )", &[]);
    let ret = conn.execute("CREATE TABLE IF NOT EXISTS bus_realtime (
                    id              serial PRIMARY KEY,
                    line_id         integer,
                    bus_id          integer,
                    coords          geography(POINT,4326),
                    gps_time        timestamptz,
                    next_station_no integer,
                    next_station_time timestamptz,
                    recorded_at     timestamptz NOT NULL DEFAULT CURRENT_TIMESTAMP,
                    UNIQUE (line_id, bus_id, gps_time)
                  )", &[]);
    println!("create table => {:?}", ret);


    let (tx, rx) = channel();


    // avoid calling drop
    let mut threads = Vec::new();
    let manager = PostgresConnectionManager::new(connection_str, SslMode::None).unwrap();
    let error_handler = Box::new(r2d2::LoggingErrorHandler);
    let config = r2d2::Config::builder().pool_size(20).connection_timeout(time::Duration::seconds(20)).build();
    let pool = Arc::new(r2d2::Pool::new(config, manager, error_handler).unwrap());

    for line in lines.iter() {
        if line.long_name.matches("运通").count() == 0 {
            continue;
        }

        // if line.id != 873 {
        //     continue
        // }
        let api = BeijingBusApi::new();
        let line_id = line.id;
        let tx = tx.clone();

        let t = thread::Builder::new().name(format!("LINE-{:04}", line_id)).scoped(move || {
            println!("start new thread! named = {}", thread::current().name().unwrap());
            let mut retries = 0;
            loop {
                let start_time_ns = time::precise_time_ns();

                let ret = match api.get_realtime_busline_info(line_id, 2000) {
                    Ok(ret) => {
                        retries = 0;
                        ret
                    },
                    Err(_)  => {
                        retries += 1;
                        print!("E");
                        thread::sleep_ms(4000);
                        if retries > 5 {
                            return;
                        } else {
                            continue;
                        }
                    }
                };
                ret.iter().map(|bus| {
                    let coords = Point::from_gcj02(bus.coords.lng as f64, bus.coords.lat as f64);
                    let nst: Option<time::Timespec> = if bus.next_station_time == -1 {
                        None
                    } else {
                        Some(time::Timespec::new(bus.next_station_time, 0))
                    };

                    tx.send((line_id, bus.id, time::Timespec::new(bus.gps_update_time, 0), coords, bus.next_station_no, nst));

                }).count();
                if (time::precise_time_ns() - start_time_ns) / 1000000000 < 8 {
                    thread::sleep_ms(8000);
                }
            }
        });
        threads.push(t.unwrap());
    }

    let conn = pool.get().unwrap();

    loop {
        let (line_id, bus_id, gst, coords, ns_no, nst) = rx.recv().unwrap();
        let stmt = conn.prepare("select bus_id from bus_realtime where line_id = $1 and bus_id = $2 and gps_time = $3").unwrap();
        let rows = stmt.query(&[&line_id, &bus_id, &gst]).unwrap();
        if rows.iter().count() == 1 {
            //println!("{}/#{} skip!", line.short_name, bus.id);
            print!(".");
            continue
        }
        let ret = conn.execute("INSERT INTO bus_realtime (line_id, bus_id, gps_time, coords, next_station_no, next_station_time)
                                VALUES ($1, $2, $3, $4, $5, $6)",
                               &[&line_id, &bus_id, &gst, &coords, &ns_no, &nst]);
        print!("!");
        io::stdout().flush();
    }

    Ok(())
}


#[bench]
fn test_gcj02_bad_values(b: &mut test::Bencher) {
    let mut rng = rand::thread_rng();
    let between_x = Range::new(72.0f32, 137.8);
    let between_y = Range::new(0.8293f32, 55.8271);

    let (x, y) = (between_x.ind_sample(&mut rng), between_y.ind_sample(&mut rng));
    b.iter(|| {

        let (rx, ry) = mars::to_wgs84(x as f64, y as f64);
    });
}



fn main() {
    //main1();

    //test_gcj02_bad_values();

    //println!("===> {:?}", mars::to_wgs84(72.006454, 41.425484));

    //match run() {
    match insert_db() {
        Err(e) => {
            println!("error: {}", e.description());
        }
        _ => ()
    }
}
