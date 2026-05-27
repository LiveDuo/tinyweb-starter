
use std::process::Command;
use std::path::PathBuf;

use tiny_http::Server;

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

    let server = Server::new("0.0.0.0:8080", None).unwrap();
    println!("Listening on port {:?}", server.listening_addr.port());

    let mut tasks = Vec::<Task>::new();

    for mut request in &server {
        println!("{:?} {:?}", request.method, request.path);

        match (request.method.as_str(), request.path.as_str()) {
            (_, _) if request.path.starts_with("/api/tasks") => {

                let mut body = String::new();
                request.body().read_to_string(&mut body).unwrap();

                match request.method.as_str() {
                    "GET" => {

                        let _tasks = tasks.iter()
                            .map(|s| json::object!{ title: s.title.to_owned(), done: s.done.to_owned() }).collect::<Vec<_>>();
                        let message = json::object!{ tasks: _tasks };
                        request.data(message.dump()).with_header("content-type", "application/json").send().unwrap();
                    },
                    "POST" => {

                        let value = json::parse(body.as_str()).unwrap();
                        let title = value["title"].as_str().unwrap().to_owned();
                        let done = value["done"].as_bool().unwrap();
                        tasks.push(Task { title, done });

                        let message = json::object!{ success: true };
                        request.data(message.dump()).with_header("content-type", "application/json").send().unwrap();

                    },
                    "PUT" => {

                        let id_opt = request.path.split("/").nth(3);
                        if id_opt.is_none() {
                            request.data("Invalid parameter").send().unwrap();
                            return;
                        }

                        let id = id_opt.unwrap().parse::<usize>().unwrap();
                        let task_opt = tasks.get_mut(id);
                        if task_opt.is_none() {
                            request.data("Task error").send().unwrap();
                            return;
                        }

                        let value = json::parse(body.as_str()).unwrap();
                        let title = value["title"].as_str().unwrap().to_owned();
                        let done = value["done"].as_bool().unwrap();
                        *task_opt.unwrap() = Task { title, done };

                        let message = json::object!{ success: true };
                        request.data(message.dump()).with_header("content-type", "application/json").send().unwrap();

                    },
                    "DELETE" => {

                        let id_opt = request.path.split("/").nth(3);
                        if id_opt.is_none() {
                            request.data("Invalid parameter").send().unwrap();
                            return;
                        }

                        let id = id_opt.unwrap().parse::<usize>().unwrap();
                        let task_opt = tasks.get_mut(id);
                        if task_opt.is_none() {
                            request.data("Task error").send().unwrap();
                            return;
                        }

                        tasks.remove(id);

                        let message = json::object!{ success: true };
                        request.data(message.dump()).with_header("content-type", "application/json").send().unwrap();

                    },
                    _ => {
                        request.data("Invalid request").send().unwrap();
                    }
                }
            },
            ("GET", "/client.wasm") => {
                let mode = if cfg!(debug_assertions) { "debug" } else { "release" };
                let target_path = PathBuf::from("target").join(WASM_TRIPLET).join(mode);
                let data = std::fs::read(target_path.join("client.wasm")).unwrap();
                request.data(data).with_header("content-type", "application/wasm").send().unwrap();
            },
            ("GET", _) => {
                let data = std::fs::read_to_string(PathBuf::from("public").join("index.html")).unwrap();
                request.data(data).with_header("content-type", "text/html").send().unwrap();
            },
            _ => {
                request.data("Not found").send().unwrap();
            }
        }
    }
}
