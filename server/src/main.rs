
use tiny_http::{Server, Header, Response};

// cargo run -p server
fn main() {

    
    let server = Server::http("0.0.0.0:4000").unwrap();
    println!("Listening on port {:?}", server.server_addr().port());

    for request in server.incoming_requests() {
        println!("{:?} {:?}", request.method(), request.url());

        match (request.method().as_str(), request.url()) {
            ("GET", "/") => {
                // TODO read from file
                let header = "Content-type: text/html".parse::<Header>().unwrap();
                let response = Response::from_string("<b>hello world</b>");
                request.respond(response.with_header(header)).unwrap();
            },
            ("GET", "/assets/client.wasm") => {
                // TODO read from file
                let header = "Content-type: text/html".parse::<Header>().unwrap();
                let response = Response::from_string("<b>hello world</b>");
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