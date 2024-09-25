
use tiny_http::{Server, Response};

// cargo run -p server
fn main() {

    
    let server = Server::http("0.0.0.0:4000").unwrap();
    println!("Listening on port {:?}", server.server_addr().port());

    for request in server.incoming_requests() {
        println!("{:?} {:?}", request.method(), request.url());

        let response = Response::from_string("hello world");
        request.respond(response).unwrap();
    }
}