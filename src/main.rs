use std::net::SocketAddr;

use http_body_util::{combinators::BoxBody, BodyExt};
use hyper::{body::Bytes, server::conn::http1, service::service_fn, Method, Request, Response};
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;

mod test_main;
mod test_aes;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
  println!("Hello, world!");
  let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
  let listener = TcpListener::bind(addr).await?;
  loop {
    let (stream, _) = listener.accept().await?;
    let io = TokioIo::new(stream);
    tokio::task::spawn(async move{
      if let Err(err) = http1::Builder::new()
        .serve_connection(io, service_fn(flow1000_server)).await {
          println!("Error serving connection:{:?}", err);
        }

    });
  }
}

fn full<T: Into<Bytes>>(chunk: T) -> BoxBody<Bytes, hyper::Error> {
  http_body_util::Full::new(chunk.into())
      .map_err(|never| match never {})
      .boxed()
}



async fn flow1000_server(req: Request<hyper::body::Incoming>) 
  -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {
  match (req.method(), req.uri().path()) {
    (&Method::GET, "/") => Ok(Response::new(full("hello"))),
    _ => Ok(Response::new(full("hello"))),
      
  }
}