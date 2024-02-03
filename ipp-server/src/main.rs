use std::convert::Infallible;
use std::net::SocketAddr;

use http_body_util::{BodyExt, Full};
use hyper::server::conn::http1;
use hyper::service::Service;
use hyper::{Request, Response};
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;
use bytes::{Buf, BufMut, Bytes, BytesMut};


use tokio::sync::mpsc;
use tokio::sync::oneshot;

mod ipp;
mod store_pdf;



fn send_document_type(buf: &mut BytesMut) {
    buf.put_u8(ipp::DelimiterTag::OperationAttributes as u8);
    
    ipp::send_attribute(ipp::ValueTag::Charset, "attributes-charset", "utf-8", buf);
    ipp::send_attribute(ipp::ValueTag::NaturalLanguage, "attributes-natural-language", "en-us", buf);

    buf.put_u8(ipp::DelimiterTag::PrinterAttributes as u8);    
    ipp::send_attribute(ipp::ValueTag::MimeMediaType, "document-format-supported", "application/pdf", buf);
    ipp::send_attribute(ipp::ValueTag::MimeMediaType, "", "application/postscript", buf);

    ipp::send_attribute(ipp::ValueTag::Keyword, "ipp-versions-supported", "1.1", buf);
    
    ipp::send_attribute(ipp::ValueTag::MimeMediaType, "document-format-default", "application/pdf", buf);
    ipp::send_attribute(ipp::ValueTag::Keyword, "compression-supported", "none", buf);
    ipp::send_attribute(ipp::ValueTag::Boolean, "printer-is-accepting-jobs", "\x01", buf);
    ipp::send_attribute(ipp::ValueTag::Enum, "printer-state", "\x00\x00\x00\x03", buf);
    ipp::send_attribute(ipp::ValueTag::Keyword, "printer-state-reasons", "none", buf);
    
    ipp::send_attribute(ipp::ValueTag::Uri, "printer-uri-supported", "ipp://192.168.0.10/remarkable-printer", buf);
    ipp::send_attribute(ipp::ValueTag::NameWithoutLanguage, "printer-name", "remarkable-test-printer", buf);
    ipp::send_attribute(ipp::ValueTag::Keyword, "uri-authentication-supported", "none", buf);
    ipp::send_attribute(ipp::ValueTag::Keyword, "uri-security-supported", "none", buf);
    
    ipp::send_attribute(ipp::ValueTag::Enum, "operations-supported", "\x00\x00\x00\x02", buf);
    ipp::send_attribute(ipp::ValueTag::Enum, "", "\x00\x00\x00\x04", buf);
    ipp::send_attribute(ipp::ValueTag::Enum, "", "\x00\x00\x00\x0a", buf);
    ipp::send_attribute(ipp::ValueTag::Enum, "", "\x00\x00\x00\x0b", buf);
    
}

async fn print_job(buf: &mut BytesMut, state: JobHandler) {
    let job_id = state.add_job().await;
    
    buf.put_u8(ipp::DelimiterTag::OperationAttributes as u8);
    
    ipp::send_attribute(ipp::ValueTag::Charset, "attributes-charset", "utf-8", buf);
    ipp::send_attribute(ipp::ValueTag::NaturalLanguage, "attributes-natural-language", "en-us", buf);

    write_job(job_id, buf);
}

fn validate_job(buf: &mut BytesMut) {
    buf.put_u8(ipp::DelimiterTag::OperationAttributes as u8);
    
    ipp::send_attribute(ipp::ValueTag::Charset, "attributes-charset", "utf-8", buf);
    ipp::send_attribute(ipp::ValueTag::NaturalLanguage, "attributes-natural-language", "en-us", buf);
}



async fn send_jobs(buf: &mut BytesMut, state: JobHandler) {
    let jobs = state.fetch_jobs().await;

    buf.put_u8(ipp::DelimiterTag::OperationAttributes as u8);
    

    println!("Jobs: {:?}", &jobs);

    for job in jobs {
        write_job(job, buf);
    }
}

async fn send_job(buf: &mut BytesMut, job_id: u32, state: JobHandler) {
    let jobs = state.fetch_jobs().await;

    buf.put_u8(ipp::DelimiterTag::OperationAttributes as u8);
    

    println!("Jobs: {:?}", &jobs);

    for job in jobs {
        if job == job_id {
            write_job(job, buf);
        }
    }
}

fn write_job(job_id: u32, buf: &mut BytesMut) {
    let job_bytes: [u8; 4] = job_id.to_be_bytes();
    let job_id = String::from(String::from_utf8_lossy(&job_bytes)); // TODO: This is wrong for octets higher than 128
    let mut job_uri = "ipp://192.168.0.10/remarkable-printer/jobs/".to_string();
    job_uri.push_str(job_id.to_string().as_str());

    buf.put_u8(ipp::DelimiterTag::JobAttributes as u8);
    ipp::send_attribute(ipp::ValueTag::Charset, "attributes-charset", "utf-8", buf);
    ipp::send_attribute(ipp::ValueTag::NaturalLanguage, "attributes-natural-language", "en-us", buf);
    ipp::send_attribute(ipp::ValueTag::Integer, "job-id", job_id.as_str(), buf);
    
    ipp::send_attribute(ipp::ValueTag::Uri, "job-uri", job_uri.as_str(), buf);
    

    ipp::send_attribute(ipp::ValueTag::Enum, "job-state", "\x00\x00\x00\x09", buf);
    ipp::send_attribute(ipp::ValueTag::Keyword, "job-state-reasons", "job-completed-successfully", buf);
    ipp::send_attribute(ipp::ValueTag::NameWithoutLanguage, "job-name", "com.google.android.apps.photos.Image", buf);
    ipp::send_attribute(ipp::ValueTag::Uri, "printer-uri", "ipp://192.168.0.10/remarkable-printer", buf);                
}

async fn response(status_code: u16, request_id: u32, request_type: u16, job_id: u32, state: JobHandler) -> Bytes {
    let mut buf: BytesMut = BytesMut::new();

    buf.put_u8(0x01);
    buf.put_u8(0x01);
    buf.put_u16(status_code);
    buf.put_u32(request_id);

    if request_type == 10 {
        send_jobs(&mut buf, state).await;
    } else if request_type == 2 {
        print_job(&mut buf, state).await;
    } else if request_type == 11 {
        send_document_type(&mut buf);
    } else if request_type == 4 {
        validate_job(&mut buf);
    } else if request_type == 9 {
        send_job(&mut buf, job_id, state).await;
    } else {
        panic!("Unknown request type: {}", request_type)
    }

    buf.put_u8(ipp::DelimiterTag::EndOfAttributes as u8);

    buf.freeze()
}


#[derive(Clone)]
struct IppHandler {
    job_handler: JobHandler
}

async fn reply(state: JobHandler, request: Request<hyper::body::Incoming>) -> Result<Response<Full<Bytes>>, Infallible> {
    println!("{} {}", request.method(), request.uri().path());
    
    let mut a = request.collect().await.unwrap().aggregate();

    let message = ipp::parse(&mut a).unwrap();

    let data_size = a.remaining();
    println!("Data size: {}", data_size);
    
    if data_size > 0 {
        let job_name = message.get_attribute("job-name").map(|a| match a {
            ipp::AttributeValue::Other(ipp::ValueTag::NameWithoutLanguage, b) => {
                String::from(String::from_utf8_lossy(&b))
            },
            _ => String::from("PRINTED_UNKNOWN_NAME")
        }).unwrap_or(String::from("PRINTED_UNKNOWN_NAME"));
        store_pdf::store_pdf(a.copy_to_bytes(data_size), job_name.as_str());
    }
    
    let job_id = message.get_attribute("job-id").map(|a| match a {
        ipp::AttributeValue::Other(ipp::ValueTag::Integer, b) => {
            b.clone().get_u32()
        },
        _ => 0
    }).unwrap_or(0);

    println!("{:#?} {}", message, job_id);

    let r = response(0, message.request_id, message.operation_id, job_id, state).await;
    println!("Success: {:?}", r.to_vec());
    Ok(Response::new(Full::new(r)))
}


impl Service<Request<hyper::body::Incoming>> for IppHandler {
    type Response = Response<Full<Bytes>>;
    type Error = Infallible;
    type Future = std::pin::Pin<Box<dyn std::future::Future<Output = Result<Self::Response, Self::Error>> + Send>>; // wtf

    fn call(&self, request: Request<hyper::body::Incoming>) -> Self::Future {
        let job_handler = self.job_handler.clone();
        Box::pin(async { reply(job_handler, request).await })
    }
}

#[derive(Clone)]
struct JobHandler {
    add_job: mpsc::Sender<oneshot::Sender<u32>>,
    fetch_jobs: mpsc::Sender<oneshot::Sender<Vec<u32>>>
}

impl JobHandler {
    async fn add_job(&self) -> u32 {
        let (sender, receiver) = oneshot::channel();
        self.add_job.send(sender).await.unwrap();
        receiver.await.unwrap()
    }
    async fn fetch_jobs(&self) -> Vec<u32> {
        let (sender, receiver) = oneshot::channel();
        self.fetch_jobs.send(sender).await.unwrap();
        receiver.await.unwrap()
    }
}


async fn handle_jobs(mut add_job_rx: mpsc::Receiver<oneshot::Sender<u32>>, mut fetch_jobs_rx: mpsc::Receiver<oneshot::Sender<Vec<u32>>>) {
    let mut jobs = Vec::new();
 
    loop {
        tokio::select! {
            Some(add_job) = add_job_rx.recv() => {
                let job_id = jobs.len() as u32 + 17;
                jobs.push(job_id);
                add_job.send(job_id).unwrap();
            }
            Some(fetch_jobs) = fetch_jobs_rx.recv() => {
                fetch_jobs.send(jobs.clone()).unwrap();
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("IPP Server");
    let addr = SocketAddr::from(([0, 0, 0, 0], 631));

    // We create a TcpListener and bind it to 127.0.0.1:631
    let listener = TcpListener::bind(addr).await?;

    let (add_job_tx, add_job_rx) = mpsc::channel(1);
    let (fetch_jobs_tx, fetch_jobs_rx) = mpsc::channel(1);

    let ipp_handler = IppHandler {
        job_handler: JobHandler {
            add_job: add_job_tx,
            fetch_jobs: fetch_jobs_tx
        }
    };

    tokio::task::spawn(handle_jobs(add_job_rx, fetch_jobs_rx));

    // We start a loop to continuously accept incoming connections
    loop {
        let (stream, _) = listener.accept().await?;

        // Use an adapter to access something implementing `tokio::io` traits as if they implement
        // `hyper::rt` IO traits.
        let io = TokioIo::new(stream);

        let ipp_handler = ipp_handler.clone();

        // Spawn a tokio task to serve multiple connections concurrently
        tokio::task::spawn(async move {
            // Finally, we bind the incoming connection to our `hello` service
            if let Err(err) = http1::Builder::new()
                // `service_fn` converts our function in a `Service`
                .serve_connection(io, ipp_handler)
                .await
            {
                println!("Error serving connection: {:?}", err);
            }
        });
    }
}
