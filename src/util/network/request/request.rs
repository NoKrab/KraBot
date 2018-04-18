use serde_json;
use serde_json::Value;
use std::io;
use futures::{Future, Stream};
use hyper::{Client, Method, Request};
use hyper::header::{ContentLength, ContentType};
use hyper_tls::HttpsConnector;
use tokio_core::reactor::Core;

/// Makes a request to the given uri and returns the response body
/// # Examples
/// ```
/// use util::network::request::request;
/// use serde_json::Value;
///
/// let result = request::get(String::from("http://httpbin.org/ip")).unwrap();
///  info!("{}", result);
///
/// ```
///
pub fn get(uri: String) -> Option<Value> {
    let uri = &uri[..];
    let mut core = Core::new().unwrap();
    let handle = core.handle();
    let client = Client::configure()
        .connector(HttpsConnector::new(4, &handle).unwrap())
        .build(&handle);
    info!("Get Request: {}", uri);

    let request = client
        .request(Request::new(Method::Get, uri.parse().unwrap()))
        .and_then(|res| {
            res.body().concat2().and_then(move |body| {
                let v: Value = serde_json::from_slice(&body).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
                Ok(v)
            })
        });

    let result = core.run(request).unwrap();
    Some(result)
}

pub fn post() {
    info!("i tried");
    let mut core = Core::new().unwrap();
    let handle = core.handle();
    let client = Client::configure()
        .connector(HttpsConnector::new(4, &handle).unwrap())
        .build(&handle);

    let json = r#"{"library":"hyper"}"#;
    let uri = "https://httpbin.org/post".parse().unwrap();

    let mut req = Request::new(Method::Post, uri);
    req.headers_mut().set(ContentType::json());
    req.headers_mut().set(ContentLength(json.len() as u64));
    req.set_body(json);

    let request = client
        .request(req)
        .and_then(|res| {
            res.body().concat2().and_then(move |body| {
                let v: Value = serde_json::from_slice(&body).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
                Ok(v)
            })
    });

    info!("{}", core.run(request).unwrap());
}




//pub fn run() {
//    let mut core = Core::new().unwrap();
//    let client = Client::new(&core.handle());
//
//    let json = r#"{"library":"hyper"}"#;
//    let uri = "http://httpbin.org/post".parse().unwrap();
//    let mut req = Request::new(Method::Post, uri);
//    req.headers_mut().set(ContentType::json());
//    req.headers_mut().set(ContentLength(json.len() as u64));
//    req.set_body(json);
//
//    let oha = String::from("http://httpbin.org/headers");
//    let ohastr = &oha[..];
//
//    let post = client.request(req).and_then(|res| {
//        debug!("POST: {}", res.status());
//
//        res.body().concat2()
//    });
//
//    let get = client.get(ohastr.parse().unwrap()).and_then(|res| {
//        debug!("GET: {}", res.status());
//
//        res.body().concat2()
//    });
//
//    let work = post.join(get);
//    let (posted, got) = core.run(work).unwrap();
//
//    let something = str::from_utf8(&posted).unwrap();
//    debug!("POST UTF8 from slice of bytes: {}", something);
//
//    let something = str::from_utf8(&got).unwrap();
//    debug!("GOT UTF8 from slice of bytes: {}", something);
//
//    let komisch: Value = serde_json::from_slice(&got).expect("Allesvorbei");
//    debug!("{:?}", komisch);
//    debug!("{}", komisch.to_string());
//    debug!("{}", komisch["headers"]["Host"]);
//
//    let uri = "http://httpbin.org/ip".parse().unwrap();
//    let work = client.get(uri).and_then(|res| {
//        debug!("Response: {}", res.status());
//
//        res.body().concat2().and_then(move |body| {
//            let v: Value = serde_json::from_slice(&body)
//                .map_err(|e| io::Error::new(io::ErrorKind::Other, e))
//                .unwrap();
//            debug!("{:?}", v);
//            debug!("current IP address is {}", v["origin"]);
//            Ok(())
//        })
//    });
//    let _ = core.run(work).unwrap();
//}







