use std::io::Write;

use ventus_proto::{VentusRequest, VentusResponse};

fn main() {
    // Read the request from stdin
    let request: VentusRequest = rmp_serde::from_read(std::io::stdin()).unwrap();

    // Create a response
    let response = VentusResponse {
        status_code: 200,
        headers: vec![],
        // headers: HashMap::from([("Content-Type".to_string(), "text/plain".to_string())]),
        body: request.body,
    };

    // Write the response to stdout
    let response = rmp_serde::to_vec(&response).unwrap();
    std::io::stdout().write_all(&response).unwrap();
}
