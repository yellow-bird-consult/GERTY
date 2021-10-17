use std::convert::Infallible;
use std::net::SocketAddr;
use hyper::{server::Server, Body, Request, Response, StatusCode, http::Error};
use hyper::service::{make_service_fn, service_fn};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use serde_json;
use hyper::body;
use uuid::Uuid;


fn extract_string_parameter<'a>(body_data: &'a serde_json::Value, key: &'a str) -> Result<&'a str, &'a str> {
    match body_data.get(key) {
        Some(inside_value) => {
            return Ok(inside_value.as_str().unwrap())
        },
        None => {
            return Err("not found")
        }
    }
}


async fn handle(req: Request<Body>, database: Arc<Mutex<HashMap<String, Vec<String>>>>) -> Result<Response<Body>, Error> {
    let bytes = body::to_bytes(req.into_body()).await.unwrap();
    let string_body = String::from_utf8(bytes.to_vec()).expect("response was not valid utf-8");
    let value: serde_json::Value = serde_json::from_str(&string_body.as_str()).unwrap();

    let disease = extract_string_parameter(&value, "disease").unwrap().to_string();
    let mut db = database.lock().unwrap();

    match extract_string_parameter(&value, "method").unwrap() {
        "GET" => {
            match db.get(&disease) {
                Some(data_vector) => {
                    // get the value from redis

                    // work on returning JSON
                    let builder = Response::builder().status(StatusCode::OK)
                                                                               .body(Body::from("test"));
                    return builder
                },
                None => {
                    let builder = Response::builder().status(StatusCode::NOT_FOUND)
                                                                       .body(Body::from("test"));
                    return builder
                }
            }
        },
        "SET" => {
            let uuid: String;
            // TODO extract the patient data and bytes for SQLite

            match db.get(&disease) {

                Some(data_vector) => {
                    let mut placeholder_vector = data_vector.clone();
                    uuid = Uuid::new_v4().to_string();
                    placeholder_vector.push(uuid);
                    db.insert(disease, placeholder_vector);
                },
                None => {
                    uuid = Uuid::new_v4().to_string();
                    let mut data_vector = Vec::new();
                    data_vector.push(uuid);
                    db.insert(disease, data_vector);
                }
            }
            // insert the patient into Redis
            println!("{:?}", db);
            let builder = Response::builder().status(StatusCode::OK)
                                                    .body(Body::from("Patient inserted"));
            return builder
        },
        _ => {
            let builder = Response::builder().status(StatusCode::NOT_ACCEPTABLE)
                                                                       .body(Body::from("Hello World"));
            return builder
        }
    }
}

#[tokio::main]
async fn main() {
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let db: Arc<Mutex<HashMap<String, Vec<String>>>> = Arc::new(Mutex::new(HashMap::new()));

    let server = Server::bind(&addr).serve(make_service_fn(move |_conn| {
        let db = db.clone();
        async move { Ok::<_, Infallible>(service_fn(move |req| handle(req, db.clone()))) }
    }));

    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}