
use std::process::Command;
use std::path::PathBuf;

use tiny_http::{Server, Header, Response};

struct Task { title: String, done: bool }

fn main() {

    const WASM_TRIPLET: &str = "wasm32-unknown-unknown";

    if cfg!(debug_assertions) {
        print!("Building wasm...");
        let p = Command::new("cargo").args(["build", "-p", "client", "--target", WASM_TRIPLET]).output().unwrap();
        if !p.status.success() {
            panic!("{}", String::from_utf8(p.stderr).unwrap());
        }
        println!("Done");
    }

    let server = Server::http("0.0.0.0:8080").unwrap();
    println!("Listening on port {:?}", server.server_addr().to_ip().unwrap().port());

    let mut tasks = Vec::<Task>::new();

    for mut request in server.incoming_requests() {
        println!("{:?} {:?}", request.method(), request.url());

        match (request.method().as_str(), request.url()) {
            (_, _) if request.url().starts_with("/api/tasks") => {

                let header: Header = "Content-type: application/json".parse::<Header>().unwrap();

                let mut body = String::new();
                request.as_reader().read_to_string(&mut body).unwrap();

                match request.method().as_str() {
                    "GET" => {

                        let _tasks = tasks.iter()
                            .map(|s| json::object!{ title: s.title.to_owned(), done: s.done.to_owned() }).collect::<Vec<_>>();
                        let message = json::object!{ tasks: _tasks };
                        let response = Response::from_string(message.dump());
                        request.respond(response.with_header(header)).unwrap();
                    },
                    "POST" => {

                        let value = json::parse(body.as_str()).unwrap();
                        let title = value["title"].as_str().unwrap().to_owned();
                        let done = value["done"].as_bool().unwrap();
                        tasks.push(Task { title, done });

                        let message = json::object!{ success: true };
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

                        let id = id_opt.unwrap().parse::<usize>().unwrap();
                        let task_opt = tasks.get_mut(id);
                        if task_opt.is_none() {
                            let response = Response::from_string("Task error");
                            request.respond(response).unwrap();
                            return;
                        }

                        let value = json::parse(body.as_str()).unwrap();
                        let title = value["title"].as_str().unwrap().to_owned();
                        let done = value["done"].as_bool().unwrap();
                        *task_opt.unwrap() = Task { title, done };

                        let message = json::object!{ success: true };
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

                        let id = id_opt.unwrap().parse::<usize>().unwrap();
                        let task_opt = tasks.get_mut(id);
                        if task_opt.is_none() {
                            let response = Response::from_string("Task error");
                            request.respond(response).unwrap();
                            return;
                        }

                        tasks.remove(id);

                        let message = json::object!{ success: true };
                        let response = Response::from_string(message.dump());
                        request.respond(response.with_header(header)).unwrap();

                    },
                    _ => {
                        let response = Response::from_string("Invalid request");
                        request.respond(response).unwrap();
                    }
                }
            },
            ("GET", "/client.wasm") => {
                let mode = if cfg!(debug_assertions) { "debug" } else { "release" };
                let target_path = PathBuf::from("target").join(WASM_TRIPLET).join(mode);
                let data = std::fs::read(target_path.join("client.wasm")).unwrap();
                let header = "Content-type: application/wasm".parse::<Header>().unwrap();
                let response = Response::from_data(data);
                request.respond(response.with_header(header)).unwrap();
            },
            ("GET", _) => {
                let data = std::fs::read_to_string(PathBuf::from("public").join("index.html")).unwrap();
                let header = "Content-type: text/html".parse::<Header>().unwrap();
                let response = Response::from_string(data);
                request.respond(response.with_header(header)).unwrap();
            },
            _ => {
                let response = Response::from_string("Not found");
                request.respond(response).unwrap();
            }
        }
    }
}
