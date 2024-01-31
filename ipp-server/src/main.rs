use std::convert::Infallible;
use std::net::SocketAddr;

use http_body_util::{BodyExt, Full};
use hyper::body::Bytes;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response};
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;





fn parseMessage(b: Bytes) -> Result<(), box <  std::error::Error> > {

}



async fn hello(request: Request<hyper::body::Incoming>) -> Result<Response<Full<Bytes>>, Infallible> {

    println!("{} {}", request.method(), request.uri().path());

    // Make hexdump from the binary body data
    let mut hexdump = String::new();
    hexdump.push_str("  Hex ");
    match request.collect().await {
        Ok(bytes) => {
            let b = bytes.to_bytes();
            b.data;
            
            // .iter().for_each(|b| {
            //     hexdump.push_str(&format!("{:02x} ", b));
            // });
            println!("{}", hexdump)
        }
        Err(e) => {
            println!("Error: {}", e);
        }
    }

    Ok(Response::new(Full::new(Bytes::from("Hello, Worl2d!"))))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("Hello, world!");
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    // We create a TcpListener and bind it to 127.0.0.1:3000
    let listener = TcpListener::bind(addr).await?;

    // We start a loop to continuously accept incoming connections
    loop {
        let (stream, _) = listener.accept().await?;

        // Use an adapter to access something implementing `tokio::io` traits as if they implement
        // `hyper::rt` IO traits.
        let io = TokioIo::new(stream);

        // Spawn a tokio task to serve multiple connections concurrently
        tokio::task::spawn(async move {
            // Finally, we bind the incoming connection to our `hello` service
            if let Err(err) = http1::Builder::new()
                // `service_fn` converts our function in a `Service`
                .serve_connection(io, service_fn(hello))
                .await
            {
                println!("Error serving connection: {:?}", err);
            }
        });
    }
}