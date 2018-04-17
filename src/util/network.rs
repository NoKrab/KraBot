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

use serde_json;
//use serde_json::Error;
use futures::{Future, Stream};
use hyper::header::{ContentLength, ContentType};
use hyper::Client;
use hyper::{Method, Request};
use serde_json::Value;
use std::io;
use std::str;
use tokio_core::reactor::Core;

pub fn run() {
    let mut core = Core::new().unwrap();
    let client = Client::new(&core.handle());

    let json = r#"{"library":"hyper"}"#;
    let uri = "http://httpbin.org/post".parse().unwrap();
    let mut req = Request::new(Method::Post, uri);
    req.headers_mut().set(ContentType::json());
    req.headers_mut().set(ContentLength(json.len() as u64));
    req.set_body(json);

    let oha = String::from("http://httpbin.org/headers");
    let ohastr = &oha[..];

    let post = client.request(req).and_then(|res| {
        debug!("POST: {}", res.status());

        res.body().concat2()
    });

    let get = client.get(ohastr.parse().unwrap()).and_then(|res| {
        debug!("GET: {}", res.status());

        res.body().concat2()
    });

    let work = post.join(get);
    let (posted, got) = core.run(work).unwrap();

    let something = str::from_utf8(&posted).unwrap();
    debug!("POST UTF8 from slice of bytes: {}", something);

    let something = str::from_utf8(&got).unwrap();
    debug!("GOT UTF8 from slice of bytes: {}", something);

    let komisch: Value = serde_json::from_slice(&got).expect("Allesvorbei");
    debug!("{:?}", komisch);
    debug!("{}", komisch.to_string());
    debug!("{}", komisch["headers"]["Host"]);

    let uri = "http://httpbin.org/ip".parse().unwrap();
    let work = client.get(uri).and_then(|res| {
        debug!("Response: {}", res.status());

        res.body().concat2().and_then(move |body| {
            let v: Value = serde_json::from_slice(&body).map_err(|e| io::Error::new(io::ErrorKind::Other, e)).unwrap();
            debug!("{:?}", v);
            debug!("current IP address is {}", v["origin"]);
            Ok(())
        })
    });
    let _ = core.run(work).unwrap();
}
