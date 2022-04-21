use std::convert::Infallible;
use std::net::SocketAddr;
use hyper::{server::Server, Body, Request, Response, StatusCode, http::Error};
use hyper::service::{make_service_fn, service_fn};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use serde_json;
use hyper::body;
use uuid::Uuid;
use std::fs::File;


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


fn write_patient_data_to_file(uuid: String, patient_data: &serde_json::Value) -> () {
    let path = format!("./{}.json", uuid);
    let file = &File::create(path).unwrap();
    serde_json::to_writer(file, patient_data).unwrap();
}


fn wipe_disease(mut database: std::sync::MutexGuard<HashMap<String, Vec<String>>>, disease: String) {
    match database.get(&disease) {
        Some(data_vector) => {

            for diease_reference in data_vector {
                let path = format!("./{}.json", diease_reference);
                std::fs::remove_file(path).unwrap();
            }
            database.remove(&disease);
        },
        None => {}
    }
}


async fn handle(req: Request<Body>, database: Arc<Mutex<HashMap<String, Vec<String>>>>) -> Result<Response<Body>, Error> {
    println!("getting request");
    let bytes = body::to_bytes(req.into_body()).await.unwrap();
    let string_body = String::from_utf8(bytes.to_vec()).expect("response was not valid utf-8");
    let value: serde_json::Value = serde_json::from_str(&string_body.as_str()).unwrap();

    let mut db = database.lock().unwrap();

    match extract_string_parameter(&value, "method").unwrap() {
        "GET" => {
            let disease = extract_string_parameter(&value, "disease").unwrap().to_string();
            match db.get(&disease) {
                Some(data_vector) => {
                    if data_vector.len() == 0 {
                        let builder = Response::builder().status(StatusCode::FAILED_DEPENDENCY)
                                                                       .body(Body::from("disease is empty"));
                        return builder
                    }

                    // get the value from JSON file
                    let path = format!("./{}.json", data_vector[0]);
                    let patient: serde_json::Value = serde_json::from_reader(File::open(&path).unwrap()).unwrap();

                    std::fs::remove_file(path).unwrap();

                    // remove patient from vector
                    if data_vector.len() == 1 {
                        let placeholder_vector = Vec::new();
                        db.insert(disease, placeholder_vector);
                    } else {
                        let placeholder_vector = data_vector[1..].to_vec();
                        db.insert(disease, placeholder_vector);
                    }

                    let raw_data = serde_json::to_vec(&patient).unwrap();
                    let builder = Response::builder().status(StatusCode::OK)
                                                                               .body(Body::from(raw_data));
                    return builder
                },
                None => {
                    let builder = Response::builder().status(StatusCode::NOT_FOUND)
                                                                               .body(Body::from("disease was not found"));
                    return builder
                }
            }
        },
        "SET" => {
            let disease = extract_string_parameter(&value, "disease").unwrap().to_string();
            let uuid: String;
            // TODO extract the patient data and bytes for SQLite

            match db.get(&disease) {

                Some(data_vector) => {
                    let mut placeholder_vector = data_vector.clone();
                    uuid = Uuid::new_v4().to_string();
                    placeholder_vector.push(uuid.clone());
                    db.insert(disease, placeholder_vector);
                    write_patient_data_to_file(uuid, value.get("patient").unwrap());
                },
                None => {
                    uuid = Uuid::new_v4().to_string();
                    let mut data_vector = Vec::new();
                    data_vector.push(uuid.clone());
                    db.insert(disease, data_vector);
                    write_patient_data_to_file(uuid, value.get("patient").unwrap());
                }
            }
            let builder = Response::builder().status(StatusCode::OK)
                                                    .body(Body::from("Patient inserted"));
            return builder
        },
        "COUNT" => {
            let mut counter_hash = HashMap::new();

            for (key, value) in &*db {
                counter_hash.insert(key, value.iter().count() as i32);
            }
            let raw_data = serde_json::to_vec(&counter_hash).unwrap();
            let builder = Response::builder().status(StatusCode::OK)
                                                                               .body(Body::from(raw_data));
            return builder
        },
        "WIPE" => {
            let disease = extract_string_parameter(&value, "disease").unwrap().to_string();
            wipe_disease(db, disease);
            let builder = Response::builder().status(StatusCode::OK).body(Body::from("disease wiped"));
            return builder
        }
        _ => {
            let builder = Response::builder().status(StatusCode::NOT_ACCEPTABLE)
                                                                       .body(Body::from("Hello World"));
            return builder
        }
    }
}

#[tokio::main]
async fn main() {

    println!("starting server");
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    let db: Arc<Mutex<HashMap<String, Vec<String>>>> = Arc::new(Mutex::new(HashMap::new()));
    println!("database is now initiated");

    let server = Server::bind(&addr).serve(make_service_fn(move |_conn| {
        let db = db.clone();
        async move { Ok::<_, Infallible>(service_fn(move |req| handle(req, db.clone()))) }
    }));

    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}