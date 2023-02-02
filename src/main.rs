use std::process::exit;

mod server;
mod args;

fn main() {
    let hosts = args::extract_hosts();
    let headers = args::extract_headers();
    let host = args::get("-host", "127.0.0.1".to_string()) as String;
    let port = args::get("-port", 3232) as u16;

    match hosts {
        Ok(_) => {},
        Err(err) => { println!("{}", err); exit(1); }
    }

    let server: server::WebServer = server::WebServer::new(
        host,
        port,
        Some(server::RunnerOptions {
            replace_hosts: hosts.unwrap(),
            replace_headers: headers.unwrap(),
        }),
    );

    server.run().unwrap();
}