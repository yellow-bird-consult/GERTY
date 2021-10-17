use std::convert::Infallible;
use std::net::SocketAddr;
use hyper::{server::Server, Body, Request, Response};
use hyper::service::{make_service_fn, service_fn};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use serde_json;
use hyper::body;


async fn handle(req: Request<Body>, database: Arc<Mutex<HashMap<&str, i32>>>) -> Result<Response<Body>, Infallible> {
    let bytes = body::to_bytes(req.into_body()).await.unwrap();
    let string_body = String::from_utf8(bytes.to_vec()).expect("response was not valid utf-8");
    let value: serde_json::Value = serde_json::from_str(&string_body.as_str()).unwrap();
    println!("{:?}", value);

    let mut db = database.lock().unwrap();

    match db.get("count") {
        Some(data) => {
            let new_count = data + 1;
            db.insert("count", new_count);
        }
        None => {
            db.insert("count", 1);
        }
    }

    println!("{:?}", db);
    Ok(Response::new(Body::from("Hello World")))
}

#[tokio::main]
async fn main() {
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let db: Arc<Mutex<HashMap<&str, i32>>> = Arc::new(Mutex::new(HashMap::new()));

    let server = Server::bind(&addr).serve(make_service_fn(move |_conn| {
        let db = db.clone();
        async move { Ok::<_, Infallible>(service_fn(move |req| handle(req, db.clone()))) }
    }));

    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}