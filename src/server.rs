/*
1. Connections that can't be processed are dropped with a 400 status - e.g the host can't be found in the -u option 
2. Given the format -u a:b, `a` is a seed found in the original path and `b` the domain to replace it with
*/

use std::{collections::HashMap, fmt::format, str::FromStr};

use actix_web::{
  App,
  HttpServer,
  HttpResponse,
  web,
  dev::Server, HttpRequest,
};

#[derive(Clone)]
pub struct WebServer {
  pub address: String,
  pub port: u16,
  runner: RunnerOptions,
}

#[derive(Clone)]
pub struct RunnerOptions {
  pub replace_hosts: HashMap<String, String>,
  pub replace_headers: reqwest::header::HeaderMap,
}

impl WebServer {
  pub fn new(address: String, port: u16, runner: Option<RunnerOptions>) -> WebServer {
      return WebServer {
        address,
        port,
        runner: match runner {
          Some(runner) => runner,
          None => RunnerOptions {
            replace_hosts: HashMap::new(),
            replace_headers: reqwest::header::HeaderMap::new(),
          },
        },
      };
  }
  
  #[actix_web::main]
  pub async fn run(&self) -> std::io::Result<()> {
    //let mut cfg: ServiceConfig;
    //configure(&self, &mut cfg);
    let http_server = configure(&(self.runner), self.address.clone(), self.port);

    println!("Running at http://{}:{}", self.address, self.port);
    
    return http_server.await;
  }
}

fn prepare_request(url: &mut String, req: &HttpRequest, reqwest_headers: &mut reqwest::header::HeaderMap, options: &web::Data<RunnerOptions>) {
  // Will set the final URL being proxied and headers we want to forward and change
  let path = req.uri();
  let mut domain: String = String::from("");
  
  for (key, value) in options.replace_hosts.iter() {
    if path.to_string().contains(key) {
      *url = format!("http://{}{}", value, path);
      domain = value.clone();
      break;
    }
  }

  if url == "" {
    return;
  }

  // looks necessary. actix headers are different from reqwest
  for (k, v) in (*req).headers().iter() {
    (*reqwest_headers).insert(k, (*v).clone());
  }

  for (key, v) in options.replace_headers.iter() {
    (*reqwest_headers).insert(key, (*v).clone());
  }

  // remove the host header. By default, its value is the domain of the URL being proxied (rust's server)
  (*reqwest_headers).remove("host");
  (*reqwest_headers).insert("host", domain.parse().unwrap());
}

async fn forwarder(options: web::Data<RunnerOptions>, req: HttpRequest) -> HttpResponse {
  let mut url = String::from("");
  let mut reqwest_headers = reqwest::header::HeaderMap::new();

  prepare_request(&mut url, &req, &mut reqwest_headers, &options);

  if url == "" {
    return HttpResponse::BadRequest().body("Based on the path, can't find host on RunnerOptions.replace_hosts");
  }
  
  let mut response_builder = HttpResponse::Ok();
  let response = reqwest::Client::builder().default_headers(reqwest_headers).build().unwrap().get(url).send().await.unwrap(); // default reference: reqwest::get(url).await.unwrap();
  let response_headers = response.headers().clone();
  let response_length = response.content_length();
  let body = response.bytes_stream();
  
  for (key, value) in response_headers.iter() {
      response_builder.insert_header((key.to_string(), value.to_str().unwrap()));
  }

  if !response_length.is_none() {
    response_builder.no_chunking(response_length.unwrap());
  }

  return response_builder.streaming(body);
}

fn configure(options: &RunnerOptions, address: String, port: u16) -> Server {
  let options = options.clone();
  let server: Server = HttpServer::new(move || {
    App::new()
      .app_data(web::Data::new(options.clone()))
      .route("/{path:.*}", web::get().to(forwarder))
  })
  .bind(format!("{}:{}", address, port))
  .unwrap()
  .run();
  
  return server;
}