use std::fmt::{Formatter, Result, Error, Display};
use std::mem;

// http://stackoverflow.com/questions/385132/proper-best-type-for-storing-latitude-and-longitude
// Longitudes and latitudes are not generally known to any greater precision than a 32-bit float.
#[derive(Clone, Copy, Debug, RustcDecodable, RustcEncodable)]
pub struct LatLng {
    pub lat: f32,
    pub lng: f32
}

impl LatLng {
    pub fn new() -> LatLng {
        LatLng { lat: 0.0, lng: 0.0 }
    }
}

impl Display for LatLng {
    fn fmt(&self, f: &mut Formatter) -> Result {
        try!(write!(f, "({}, {})", self.lng, self.lat));
        Ok(())
    }
}

#[derive(Debug, RustcDecodable, RustcEncodable)]
pub struct BusLine {
    pub id: i32,
    pub version: i32,
    pub route: Vec<LatLng>,
    pub short_name: String,
    pub long_name: String,
    pub operation_time: String,
    pub stations: Vec<BusStation>
}

impl Display for BusLine {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "<BUS:{} {} ({}站) id={}#{}>",self.long_name, self.operation_time, self.stations.len(), self.id, self.version);
        Ok(())
    }
}

#[derive(Clone, Debug, RustcDecodable, RustcEncodable)]
pub struct BusStation {
    pub coords: LatLng,
    pub name: String
}

impl Display for BusStation {
    fn fmt(&self, f: &mut Formatter) -> Result {
        try!(write!(f, "<STATION:{} {}>", self.name, self.coords));
        Ok(())
    }
}


impl BusStation {
    pub fn new() -> BusStation {
        BusStation { coords: LatLng::new(), name: String::new() }
    }
}

#[derive(Clone, Debug, RustcDecodable, RustcEncodable)]
pub struct RealtimeBus {
    pub id: i32,
    pub type_: i32,
    pub coords: LatLng,
    pub next_station: String,
    pub next_station_no: i32,
    pub next_station_run_time: i32,
    pub next_station_time: i32,
    pub gps_update_time: i64,
    pub station_distance: i32,
    pub station_run_time: i32,
    pub station_time: i32,
}

impl RealtimeBus {
    pub fn new() -> RealtimeBus {
        unsafe { mem::zeroed() }
    }

    pub fn describ(&self) {
        if self.next_station_run_time == -1 && self.next_station_time == -1 {
            println!("bus#{} 到站！ => {}", self.id, self.next_station)
        } else {
            println!("bus#{} 下一站 => {}", self.id, self.next_station)
        }
    }
}
