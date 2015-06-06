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

use std::io::Read;

use hyper::Client;
use hyper::header::Connection;
use hyper::header::ConnectionOption;
use crypto::md5::Md5;
use crypto::digest::Digest;


pub use self::bus::*;

pub mod api;

mod cipher;
mod bus;

#[test]
fn it_works() {
}
