use std::io;
use std::io::{Result, Error};
use std::str::FromStr;
use hyper::Client;
use hyper::client::IntoUrl;
use hyper::client::response::Response;
use hyper::header::Headers;
use cipher::Cipher;
use xml::reader::EventReader;
use xml::reader::events::XmlEvent;
use xml::reader::events::XmlEvent::{StartElement, EndElement, Characters};
use bus::{BusLine, BusStation, LatLng, RealtimeBus};
use itertools::Itertools;

pub struct BeijingBusApi {
    headers: Headers,
}


static CHECK_UPDATE_URL: &'static str = "http://mc.aibang.com/aiguang/bjgj.c?m=checkUpdate&version=1";
static BUSLINE_INFO_URL_PREFIX: &'static str = "http://mc.aibang.com/aiguang/bjgj.c?m=update&id=";

impl BeijingBusApi {
    pub fn new() -> BeijingBusApi {
        let mut h = Headers::new();
        let uid = "233333333333333333333333333333333333333";
        let headers = vec![("SOURCE", "1"), ("PKG_SOURCE", "1"), ("OS", "android"), ("ROM", "4.2.1"),
                           ("RESOLUTION", "1280*720"), ("MANUFACTURER", "2013022"), ("MODEL", "2013022"),
                           ("UA", "2013022,17,4.2.1,HBJ2.0,Unknown,1280*720"), ("IMSI", "233333333333333"),
                           ("IMEI", "233333333333333"), ("UID", uid.clone()), ("CID", uid.clone()),
                           ("PRODUCT", "nextbus"), ("PLATFORM", "android"), ("VERSION", "1.0.5"),
                           ("FIRST_VERSION", "2"), ("PRODUCTID", "5"), ("VERSIONID", "2"),
                           ("CUSTOM", "aibang"), ("User-Agent", "Beijing Realtime Bus Crawler. v0.0.1.")];
        for &(k, v) in headers.iter() {
            h.set_raw(k, vec![v.as_bytes().to_vec()])
        }
        BeijingBusApi { headers: h }
    }

    fn api_open<U: IntoUrl>(&self, url: U) -> Result<Response> {
        let mut client = Client::new();
        let res = client.get(url).headers(self.headers.clone()).send().unwrap();
        Ok(res)
    }

    pub fn get_realtime_busline_info(&self, id: i32, no: usize) -> Result<Vec<RealtimeBus>> {
        let mut url = "http://bjgj.aibang.com:8899/bus.php".into_url().unwrap();
        url.set_query_from_pairs(vec![
            ("city", "北京"), //"%E5%8C%97%E4%BA%AC"),
            ("id", &id.to_string()),
            ("no", &no.to_string()),
            ("type", "2"),
            ("encrypt", "1"),
            ("versionid", "2")].iter().map(|&pair| pair));
        let mut resp = self.api_open(url).unwrap();
        let mut er = EventReader::new(resp);

        let mut last_text: String = String::new();
        let mut cipher = Cipher::new();
        let mut current_tag: String = String::new();
        let mut buses = Vec::new();
        let mut current_bus = RealtimeBus::new();
        let mut error_code = 200;

        for event in er.events() {
            // println!("line => {:?}", event);
            match event {
                StartElement { name: name, ..} => {
                    current_tag = name.local_name.clone();
                    if current_tag == "bus" {
                        current_bus = RealtimeBus::new();
                    }

                }
                EndElement { name: name } => {
                    match name.local_name.as_ref() {
                        "gt"     => {
                            cipher.set_aibang_key(&last_text);
                            current_bus.gps_update_time = i64::from_str(&last_text).unwrap();
                        }
                        "id"     => current_bus.id = i32::from_str(&last_text).unwrap(),
                        "ns"     => current_bus.next_station = last_text.clone(),
                        "nsn"    => current_bus.next_station_no = i32::from_str(&last_text).unwrap(),
                        "nsrt"   => current_bus.next_station_run_time = i32::from_str(&last_text).unwrap(),
                        "nst"    => current_bus.next_station_time = i64::from_str(&last_text).unwrap(),
                        "sd"     => current_bus.station_distance = i32::from_str(&last_text).unwrap(),
                        "srt"    => current_bus.station_run_time = i32::from_str(&last_text).unwrap(),
                        "st"     => current_bus.station_time = i32::from_str(&last_text).unwrap(),
                        "x"      => current_bus.coords.lng = f32::from_str(&last_text).unwrap(),
                        "y"      => current_bus.coords.lat = f32::from_str(&last_text).unwrap(),
                        "ut"     => (), // println!("ut => {}", &last_text),
                        "bus"    => buses.push(current_bus.clone()),
                        "status" => error_code = i32::from_str(&last_text).unwrap(),
                        "message" => {
                            if error_code != 200 {
                                return Err(Error::new(io::ErrorKind::NotFound,
                                                      format!("code={}: {}", error_code, last_text.clone())))
                            }
                        }
                        _        => ()
                    }
                }
                Characters(text) => {
                    match current_tag.as_ref() {
                        "ns" | "nsn" | "sd" | "srt" | "st" | "x" | "y" => {
                            last_text = cipher.decrypt_str(&text).unwrap();
                        }
                        _ => last_text = text.into(),
                    }
                }
                _ => ()
            }
        }
        Ok(buses)

    }
    pub fn get_busline_info(&self, id: i32) -> Result<BusLine> {
        let url: String = format!("{}{}", BUSLINE_INFO_URL_PREFIX, id);
        let mut resp = self.api_open(&url).unwrap();
        let mut er = EventReader::new(resp);

        let mut line = BusLine { id: id, version: 0, route: Vec::new(),
                                 short_name: String::new(), long_name: String::new(),
                                 operation_time: String::new(), stations: Vec::new() };

        let mut current_tag: String = String::new();
        let mut last_text: String = String::new();
        let mut cipher = Cipher::new();
        let mut current_station = BusStation::new();

        for event in er.events() {
            //println!("line => {:?}", event);
            match event {
                StartElement { name: name, ..} => {
                    current_tag = name.local_name.clone()
                }
                EndElement { name: name } => {
                    match name.local_name.as_ref() {
                        "lineid"   => cipher.set_aibang_key(&last_text),
                        "coord"    => {
                            let raw = cipher.decrypt_str(&last_text).unwrap();
                            let mut nit = raw.split(',').map(
                                |s| f32::from_str(s).unwrap()).into_rc();
                            for (lng, lat) in nit.clone().zip(nit.clone()) {
                                //let pos = LatLng { lat: f32::}
                                // println!("loc -> (lat = {}, lng = {})", lat, lng);
                                let loc = LatLng { lat: lat, lng: lng };
                                line.route.push(loc);
                            }
                        }
                        "no"       => (), //println!("decrypt => {}", cipher.decrypt_str(&last_text).unwrap()),
                        "version"  => line.version = i32::from_str(&last_text).unwrap(),
                        // typo in original
                        "shotname" => line.short_name = cipher.decrypt_str(&last_text).unwrap(),
                        "linename" => line.long_name = cipher.decrypt_str(&last_text).unwrap(),
                        "time"     => line.operation_time = last_text.clone(),
                        // <station> sub structure
                        "name"     => current_station.name = cipher.decrypt_str(&last_text).unwrap(),
                        "lat"      => current_station.coords.lat = f32::from_str(&cipher.decrypt_str(&last_text).unwrap()).unwrap(),
                        "lon"      => current_station.coords.lng = f32::from_str(&cipher.decrypt_str(&last_text).unwrap()).unwrap(),
                        "station"  => {
                            line.stations.push(current_station.clone());
                            current_station = BusStation::new();
                        }
                        _          => ()
                    }
                }
                Characters(text) => {
                    last_text = text.into();
                }
                _ => ()
            }

        }
        Ok(line)
    }

    pub fn update_id_versions(&self) -> Result<Vec<(i32, i32)>> {
        let mut resp = self.api_open(CHECK_UPDATE_URL).unwrap();

        let mut er = EventReader::new(resp);
        let mut line_id_versions = Vec::<(i32, i32)>::new();
        let mut current_id: i32 = 0;
        let mut current_status = 0;
        let mut current_version = 0;

        let mut current_tag: String = String::new();

        for event in er.events() {
            match event {
                StartElement { name: name, ..} => {
                    current_tag = name.local_name.clone();
                }
                EndElement { name: name } => {
                    if name.local_name == "line" {
                        if current_status == 0 {
                            line_id_versions.push((current_id, current_version));
                        }
                    }
                }
                Characters(text) => {
                    match current_tag.as_ref() {
                        "id" =>
                            current_id = i32::from_str(text.as_ref()).unwrap(),
                        "status" => {
                            current_status = i32::from_str(text.as_ref())
                            .ok().expect(format!("Error {}:{}", file!(), line!()).as_ref());
                        }
                        "version" => {
                            current_version = i32::from_str(text.as_ref()).unwrap();
                        }
                        _ => ()
                    }
                }
                _ => ()
            }
        }

        Ok(line_id_versions)
    }

}
