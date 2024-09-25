
use std::process::Command;
use std::path::PathBuf;

use tiny_http::{Server, Header, Response};

// cargo run -p server
fn main() {

    const WASM_TRIPLET: &str = "wasm32-unknown-unknown";
    
    print!("Building wasm...");
    let p = Command::new("cargo").args(["build", "-p", "client", "--target", WASM_TRIPLET]).output().unwrap();
    assert!(p.status.success());
    println!("Done");

    let server = Server::http("0.0.0.0:8000").unwrap();
    println!("Listening on port {:?}", server.server_addr().port());

    for request in server.incoming_requests() {
        println!("{:?} {:?}", request.method(), request.url());

        match (request.method().as_str(), request.url()) {
            ("GET", "/") => {
                let data = std::fs::read_to_string(PathBuf::from("public").join("index.html")).unwrap();
                let header = "Content-type: text/html".parse::<Header>().unwrap();
                let response = Response::from_string(data);
                request.respond(response.with_header(header)).unwrap();
            },
            ("GET", "/client.wasm") => {
                let target_path = PathBuf::from("target").join(WASM_TRIPLET).join("release");
                let data = std::fs::read(target_path.join("client.wasm")).unwrap();
                let header = "Content-type: application/wasm".parse::<Header>().unwrap();
                let response = Response::from_data(data);
                request.respond(response.with_header(header)).unwrap();
            },
            ("GET", "/api/ping") => {
                let header: Header = "Content-type: application/json".parse::<Header>().unwrap();
                let message = json::object!{ pong: true };
                let response = Response::from_string(message.dump());
                request.respond(response.with_header(header)).unwrap();
            },
            _ => {
                let response = Response::from_string("Not found");
                request.respond(response).unwrap();
            }
        }
    }
}