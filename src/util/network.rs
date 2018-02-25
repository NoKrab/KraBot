//extern crate futures;
//extern crate hyper;
//extern crate tokio_core;
//
//use std::io::{self, Write};
//use self::futures::{Future, Stream};
//use self::futures::future;
//
//use self::hyper::{Method, Error};
//use self::hyper::client::{Client, Request};
//use self::tokio_core::reactor::Core;
//
//
//pub fn get_host_ip() -> Future {
//    let mut core = Core::new()?;
//    let client = Client::new(&core.handle());
//
//    let uri = "http://httpbin.org/ip".parse()?;
//    let work = client.get(uri).and_then(|res| {
//        println!("Response: {}", res.status());
//        res.body().for_each(|chunk| {
//            let s = chunk.to_string().unwrap();
//
//        });
//        return future::ok(res.body());
//    });
//    core.run(work)?;
//
//
//}