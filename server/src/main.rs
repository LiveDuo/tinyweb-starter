
use tiny_http::{Server, Header, Response};

// cargo run -p server
fn main() {

    
    let server = Server::http("0.0.0.0:4000").unwrap();
    println!("Listening on port {:?}", server.server_addr().port());

    for request in server.incoming_requests() {
        println!("{:?} {:?}", request.method(), request.url());

        match (request.method().as_str(), request.url()) {
            ("GET", "/") => {
                let response = Response::from_string("<b>hello world</b>");
                let response = response.with_header("Content-type: text/html".parse::<Header>().unwrap());
                request.respond(response).unwrap();
            },
            ("GET", "/ping") => {
                let response = Response::from_string("pong");
                request.respond(response).unwrap();
            },
            _ => {
                let response = Response::from_string("Not found");
                request.respond(response).unwrap();
            }
        }
    }
}