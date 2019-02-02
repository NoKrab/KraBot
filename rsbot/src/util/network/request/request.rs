// use futures::{Future, Stream};
// use hyper::header::{ContentLength, ContentType, Headers};
// use hyper::{Client, Method, Request, Result};
// use hyper_tls::HttpsConnector;
// use serde_json;
// use serde_json::Value;
// use std::io;
// use tokio_core::reactor::Core;

// /// Makes a request to the given uri and returns the response body
// /// # Examples
// /// ```
// /// use util::network::request::request;
// /// use serde_json::Value;
// ///
// /// let result = request::get(String::from("http://httpbin.org/ip")).unwrap();
// ///  info!("{}", result);
// ///
// /// ```
// ///
// fn get(uri: String) -> Option<Value> {
//     let uri = &uri[..];
//     let mut core = Core::new().unwrap();
//     let handle = core.handle();
//     let client = Client::configure().connector(HttpsConnector::new(4, &handle).unwrap()).build(&handle);
//     info!("Get Request: {}", uri);

//     let mut req = Request::new(Method::Get, uri.parse().unwrap());

//     let request = client.request(req).and_then(|res| {
//         res.body().concat2().and_then(move |body| {
//             let v: Value = serde_json::from_slice(&body).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
//             Ok(v)
//         })
//     });

//     let result = core.run(request).unwrap();
//     Some(result)
// }

// fn post(uri: String, headers: Headers) {
//     info!("i tried");
//     let mut core = Core::new().unwrap();
//     let handle = core.handle();
//     let client = Client::configure().connector(HttpsConnector::new(4, &handle).unwrap()).build(&handle);

//     let json = r#"{"library":"hyper"}"#;
//     let uri = uri.parse().unwrap();

//     let mut req = Request::new(Method::Post, uri);
//     req.headers_mut().extend(headers.iter());

//     //    req.headers_mut().set(ContentType::json());
//     //    req.headers_mut().set(ContentLength(json.len() as u64));
//     //    req.set_body(json);

//     let request = client.request(req).and_then(|res| {
//         res.body().concat2().and_then(move |body| {
//             let v: Value = serde_json::from_slice(&body).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
//             Ok(v)
//         })
//     });

//     info!("{}", core.run(request).unwrap());
// }

// #[derive(Debug, Clone)]
// pub struct SimpleRequest {
//     uri: String,
//     method: Method,
//     headers: Option<Headers>,
//     body: Option<String>,
// }

// impl SimpleRequest {
//     pub fn new() -> SimpleRequest {
//         SimpleRequest {
//             uri: "".to_string(),
//             method: Method::Get,
//             headers: None,
//             body: None,
//         }
//     }

//     pub fn method(mut self, method: Method) -> Self {
//         self.method = method;
//         self
//     }

//     pub fn uri(mut self, uri: String) -> Self {
//         self.uri = uri;
//         self
//     }

//     pub fn headers(mut self, headers: Headers) -> Self {
//         self.headers = Some(headers);
//         self
//     }

//     pub fn get_headers(mut self) -> Option<Headers> {
//         if let Some(headers) = self.headers {
//             return Some(headers);
//         } else {
//             return None;
//         }
//     }

//     pub fn body(mut self, body: String) -> Self {
//         self.body = Some(body);
//         self
//     }

//     pub fn run(self) -> Option<Value> {
//         let mut core = Core::new().unwrap();
//         let handle = core.handle();
//         let client = Client::configure().connector(HttpsConnector::new(4, &handle).unwrap()).build(&handle);

//         let mut req = Request::new(self.method, self.uri.parse().unwrap());

//         if let Some(headers) = self.headers {
//             req.headers_mut().extend(headers.iter());
//         }

//         if let Some(body) = self.body {
//             req.headers_mut().set(ContentLength(body.len() as u64));
//             info!("Body {}", &body);
//             req.set_body(body);
//             info!("Headers: {}", req.headers_mut());
//         }

//         let request = client.request(req).and_then(|res| {
//             res.body().concat2().and_then(move |body| {
//                 let v: Value = serde_json::from_slice(&body).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
//                 Ok(v)
//             })
//         });

//         let result = core.run(request).unwrap();
//         Some(result)
//     }
// }
