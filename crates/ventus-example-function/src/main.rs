use std::io::Write;

use ventus_proto::{VentusRequest, VentusResponse};

fn main() {
    let runtime = tokio::runtime::Builder::new_current_thread();

    // Read the request from stdin
    let request: VentusRequest = rmp_serde::from_read(std::io::stdin()).unwrap();

    // Create a response
    let response = VentusResponse {
        status_code: 200,
        headers: vec![],
        body: request.body,
    };

    // Write the response to stdout
    let response = rmp_serde::to_vec(&response).unwrap();
    std::io::stdout().write_all(&response).unwrap();
}
