
use std::io::{Read, Write};
use std::{fs::OpenOptions, process::Command};
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

    let mut file = OpenOptions::new().create(true).read(true).write(true).open("/tmp/data").unwrap();

    for mut request in server.incoming_requests() {
        println!("{:?} {:?}", request.method(), request.url());

        match (request.method().as_str(), request.url()) {
            ("GET", "/") => {
                let data = std::fs::read_to_string(PathBuf::from("public").join("index.html")).unwrap();
                let header = "Content-type: text/html".parse::<Header>().unwrap();
                let response = Response::from_string(data);
                request.respond(response.with_header(header)).unwrap();
            },
            ("GET", "/client.wasm") => {
                let target_path = PathBuf::from("target").join(WASM_TRIPLET).join("debug");
                let data = std::fs::read(target_path.join("client.wasm")).unwrap();
                let header = "Content-type: application/wasm".parse::<Header>().unwrap();
                let response = Response::from_data(data);
                request.respond(response.with_header(header)).unwrap();
            },
            (_, _) if request.url().starts_with("/api/todo") => {

                let header: Header = "Content-type: application/json".parse::<Header>().unwrap();

                let mut body = String::new();
                request.as_reader().read_to_string(&mut body).unwrap();
                
                match request.method().as_str() {
                    "GET" => {

                        let id_opt = request.url().split("/").nth(3);
                        if id_opt.is_none() {
                            let response = Response::from_string("Invalid parameter");
                            request.respond(response).unwrap();
                            return;
                        }

                        let mut file_data = String::new();
                        file.read_to_string(&mut file_data).unwrap();

                        let id = id_opt.unwrap().parse::<usize>().unwrap();
                        let line_opt = file_data.split("\n").nth(id);
                        if line_opt.is_none() {
                            let response = Response::from_string("Line error");
                            request.respond(response).unwrap();
                            return;
                        }
                        
                        let message = json::object!{ data: line_opt.unwrap() };
                        let response = Response::from_string(message.dump());
                        request.respond(response.with_header(header)).unwrap();
                    },
                    "POST" => {

                        let mut file_data = String::new();
                        file.read_to_string(&mut file_data).unwrap();

                        let mut lines = file_data.split("\n").collect::<Vec<_>>();
                        lines.push("data");
                        std::fs::write("/tmp/data", lines.join("\n")).unwrap();
                        
                        let message = json::object!{ sucess: true };
                        let response = Response::from_string(message.dump());
                        request.respond(response.with_header(header)).unwrap();

                    },
                    "PUT" => {
                        
                        let id_opt = request.url().split("/").nth(3);
                        if id_opt.is_none() {
                            let response = Response::from_string("Invalid parameter");
                            request.respond(response).unwrap();
                            return;
                        }

                        let mut file_data = String::new();
                        file.read_to_string(&mut file_data).unwrap();

                        let id = id_opt.unwrap().parse::<usize>().unwrap();
                        let mut lines = file_data.split("\n").collect::<Vec<_>>();
                        if id > lines.len() {
                            let response = Response::from_string("File error");
                            request.respond(response).unwrap();
                            return;
                        }

                        lines[id] = body.as_str();
                        file.write(lines.join("\n").as_bytes()).unwrap();
                        
                        let message = json::object!{ sucess: true };
                        let response = Response::from_string(message.dump());
                        request.respond(response.with_header(header)).unwrap();

                    },
                    "DELETE" => {

                        let id_opt = request.url().split("/").nth(3);
                        if id_opt.is_none() {
                            let response = Response::from_string("Invalid parameter");
                            request.respond(response).unwrap();
                            return;
                        }

                        let mut file_data = String::new();
                        file.read_to_string(&mut file_data).unwrap();

                        let id = id_opt.unwrap().parse::<usize>().unwrap();
                        let mut lines = file_data.split("\n").collect::<Vec<_>>();
                        lines.remove(id);
                        file.write(lines.join("\n").as_bytes()).unwrap();
                        
                        let message = json::object!{ sucess: true };
                        let response = Response::from_string(message.dump());
                        request.respond(response.with_header(header)).unwrap();

                    },
                    _ => {
                        let response = Response::from_string("Invalid request");
                        request.respond(response).unwrap();
                    }
                }
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