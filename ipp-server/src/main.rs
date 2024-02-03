use std::convert::Infallible;
use std::net::SocketAddr;
use std::fs::File;

use http_body_util::{BodyExt, Full};
use hyper::server::conn::http1;
use hyper::service::Service;
use hyper::{Request, Response};
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;
use bytes::{Buf, BufMut, Bytes, BytesMut};

use std::process::Command;


use std::io::Write;
use uuid::Uuid;

use std::time::{SystemTime, UNIX_EPOCH};

use tokio::sync::mpsc;
use tokio::sync::oneshot;


#[repr(u8)]
#[derive(Copy, Clone, Debug)]
enum DelimiterTag {
    OperationAttributes = 0x01,
    JobAttributes = 0x02,
    EndOfAttributes = 0x03,
    PrinterAttributes = 0x04,
    UnsupportedAttributes = 0x05,
}

#[derive(Debug)] 
#[derive(PartialEq)]
#[repr(u8)]
enum ValueTag {
    Integer = 0x21,
    Boolean = 0x22,
    Enum = 0x23,
    OctetString = 0x30,
    DateTime = 0x31,
    Resolution = 0x32,
    RangeOfInteger = 0x33,
    BegCollection = 0x34,
    TextWithLanguage = 0x35,
    NameWithLanguage = 0x36,
    EndCollection = 0x37,
    NameWithoutLanguage = 0x42,
    Keyword = 0x44,
    Uri = 0x45,
    UriSchema = 0x46,
    Charset = 0x47,
    NaturalLanguage = 0x48,
    MimeMediaType = 0x49,
}


impl DelimiterTag {
    fn parse_tag(tag: u8) -> Option<Self> {
        match tag {
            0x01 => Some(DelimiterTag::OperationAttributes),
            0x02 => Some(DelimiterTag::JobAttributes),
            0x03 => Some(DelimiterTag::EndOfAttributes),
            0x04 => Some(DelimiterTag::PrinterAttributes),
            0x05 => Some(DelimiterTag::UnsupportedAttributes),
            _ => None,
        }
    }
}

impl ValueTag {
    fn parse_tag(tag: u8) -> Option<Self> {
        match tag {
            0x21 => Some(ValueTag::Integer),
            0x22 => Some(ValueTag::Boolean),
            0x23 => Some(ValueTag::Enum),
            0x30 => Some(ValueTag::OctetString),
            0x31 => Some(ValueTag::DateTime),
            0x32 => Some(ValueTag::Resolution),
            0x33 => Some(ValueTag::RangeOfInteger),
            0x34 => Some(ValueTag::BegCollection),
            0x35 => Some(ValueTag::TextWithLanguage),
            0x36 => Some(ValueTag::NameWithLanguage),
            0x37 => Some(ValueTag::EndCollection),
            0x42 => Some(ValueTag::NameWithoutLanguage),
            0x44 => Some(ValueTag::Keyword),
            0x45 => Some(ValueTag::Uri),
            0x46 => Some(ValueTag::UriSchema),
            0x47 => Some(ValueTag::Charset),
            0x48 => Some(ValueTag::NaturalLanguage),
            0x49 => Some(ValueTag::MimeMediaType),
            other => { println!("Encountered unknown tag {}", other); None },
        }
    }
}

#[derive(Debug)]
enum AttributeValue {
    Keyword(String),
    KeywordList(Vec<String>),
    Other(ValueTag, Bytes),
    OtherList(ValueTag, Vec<Bytes>)
}


impl AttributeValue {
    fn parse(tag: ValueTag, b: &[u8]) -> Self {
        match tag {
            ValueTag::Keyword => {
                AttributeValue::Keyword(String::from(String::from_utf8_lossy(b)))
            },
            other => {
                AttributeValue::Other(other, Bytes::copy_from_slice(b))
            }
        }
    }

    fn parse_next(self, b: &[u8]) -> Self {
        match self {
            AttributeValue::Keyword(s) => AttributeValue::KeywordList(Vec::from([s, String::from(String::from_utf8_lossy(b))])),
            AttributeValue::KeywordList(mut s) => { 
                s.push(String::from(String::from_utf8_lossy(b)));
                AttributeValue::KeywordList(s)
            },
            AttributeValue::Other(tag, s) => AttributeValue::OtherList(tag, Vec::from([s, Bytes::copy_from_slice(b)])),
            AttributeValue::OtherList(tag, mut s) => {
                s.push(Bytes::copy_from_slice(b));
                AttributeValue::OtherList(tag, s)
            }
        }
    }
}

#[derive(Debug)]
struct Attribute {
    kind: DelimiterTag,
    name: String,
    value: AttributeValue,
}

#[derive(Debug)]
struct IppIncomingMessage {
    version_major: u8,
    version_minor: u8,
    operation_id: u16,
    request_id: u32,
    attributes: Vec<Attribute>
}

fn parse(b: &mut impl Buf) -> Option<IppIncomingMessage> {
    let version_major = b.get_u8();
    let version_minor = b.get_u8();
    let operation_id = b.get_u16();
    let request_id = b.get_u32();
    println!("Version: {}.{} Operation: {}", version_major, version_minor, operation_id);
    
    let attributes = parse_attributes(b);

    let data_size = b.remaining();
    println!("Data size: {}", data_size);
    
    if data_size > 0 {
        let job_name = attributes.iter().find(|a| a.name == "job-name").map(|a| match &a.value {
            AttributeValue::Other(ValueTag::NameWithoutLanguage, b) => {
                String::from(String::from_utf8_lossy(&b))
            },
            _ => String::from("PRINTED_UNKNOWN_NAME")
        }).unwrap_or(String::from("PRINTED_UNKNOWN_NAME"));
        store_pdf(b.copy_to_bytes(data_size), job_name.as_str());
    }
    
    Some(IppIncomingMessage {
        version_major,
        version_minor,
        request_id,
        operation_id,
        attributes
    })
}


fn parse_attributes(b: &mut impl Buf) -> Vec<Attribute> {
    let mut delimiter = DelimiterTag::EndOfAttributes;
    let mut result = Vec::new();
    let mut current_attribute: Option<Attribute> = None;
    loop {
        let tag = b.get_u8();
        match DelimiterTag::parse_tag(tag) {
            Some(DelimiterTag::EndOfAttributes) => {
                break
            },
            Some(d) => {
                delimiter = d;
                continue;
            },
            None => {}
        }
        
        let tag = ValueTag::parse_tag(tag);
        let name_length = b.get_u16();
        let name = b.copy_to_bytes(name_length as usize);
        let value_length = b.get_u16();
        let value = b.copy_to_bytes(value_length as usize);

        if name_length == 0 {
            let mut new_attributes = current_attribute.unwrap();
            new_attributes = Attribute {
                value: new_attributes.value.parse_next(&value),
                ..new_attributes
            };
            current_attribute = Some(new_attributes);
        } else {
            let attribute = Attribute {
                kind: delimiter,
                name: String::from(String::from_utf8_lossy(&name)),
                value: AttributeValue::parse(tag.unwrap(), &value)
            };
            if let Some(attr) = current_attribute { result.push(attr) };
            current_attribute = Some(attribute);
        }
    }
    if let Some(attr) = current_attribute { result.push(attr) };
    result
}

fn send_attribute(tpe: ValueTag, name: &str, value: &str, buf: &mut BytesMut) {
    buf.put_u8(tpe as u8);

    let name = name.as_bytes();
    let value = value.as_bytes();

    let name_length = name.len() as u16;
    let value_length = value.len() as u16;

    buf.put_u16(name_length);
    buf.put(name);
    buf.put_u16(value_length);
    buf.put(value);
}

fn send_document_type(buf: &mut BytesMut) {
    buf.put_u8(DelimiterTag::OperationAttributes as u8);
    
    send_attribute(ValueTag::Charset, "attributes-charset", "utf-8", buf);
    send_attribute(ValueTag::NaturalLanguage, "attributes-natural-language", "en-us", buf);

    buf.put_u8(DelimiterTag::PrinterAttributes as u8);    
    send_attribute(ValueTag::MimeMediaType, "document-format-supported", "application/pdf", buf);
    send_attribute(ValueTag::MimeMediaType, "", "application/postscript", buf);

    send_attribute(ValueTag::Keyword, "ipp-versions-supported", "1.1", buf);
    
    send_attribute(ValueTag::MimeMediaType, "document-format-default", "application/pdf", buf);
    send_attribute(ValueTag::Keyword, "compression-supported", "none", buf);
    send_attribute(ValueTag::Boolean, "printer-is-accepting-jobs", "\x01", buf);
    send_attribute(ValueTag::Enum, "printer-state", "\x00\x00\x00\x03", buf);
    send_attribute(ValueTag::Keyword, "printer-state-reasons", "none", buf);
    
    send_attribute(ValueTag::Uri, "printer-uri-supported", "ipp://192.168.0.10/remarkable-printer", buf);
    send_attribute(ValueTag::NameWithoutLanguage, "printer-name", "remarkable-test-printer", buf);
    send_attribute(ValueTag::Keyword, "uri-authentication-supported", "none", buf);
    send_attribute(ValueTag::Keyword, "uri-security-supported", "none", buf);
    
    send_attribute(ValueTag::Enum, "operations-supported", "\x00\x00\x00\x02", buf);
    send_attribute(ValueTag::Enum, "", "\x00\x00\x00\x04", buf);
    send_attribute(ValueTag::Enum, "", "\x00\x00\x00\x0a", buf);
    send_attribute(ValueTag::Enum, "", "\x00\x00\x00\x0b", buf);
    
}

fn metadata_template(visible_name: &str) -> String {
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH).unwrap();
    let milliseconds = since_the_epoch.as_millis();

    format!("{{
        \"deleted\": false,
        \"lastModified\": \"{milliseconds}\",
        \"lastOpened\": \"0\",
        \"lastOpenedPage\": 0,
        \"metadatamodified\": true,
        \"modified\": true,
        \"parent\": \"\",
        \"pinned\": false,
        \"synced\": false,
        \"type\": \"DocumentType\",
        \"version\": 0,
        \"visibleName\": \"{visible_name}\"
    }}")
}

const CONTENT_TEMPLATE: &str = "{
    \"fileType\": \"pdf\"  
}";

fn store_pdf(bytes: Bytes, job_name: &str) {
    let base_path = "/home/root/.local/share/remarkable/xochitl/";
    let uuid = Uuid::new_v4();
    let uuid = uuid.as_hyphenated();
    let path = format!("{base_path}{uuid}.pdf");
    let mut file = File::create(path).unwrap();
    file.write_all(&bytes).unwrap();

    let metadata = metadata_template(job_name);
    let path = format!("{base_path}{uuid}.metadata");
    let mut file = File::create(path).unwrap();
    file.write_all(metadata.as_bytes()).unwrap();

    let path = format!("{base_path}{uuid}.content");
    let mut file = File::create(path).unwrap();
    file.write_all(CONTENT_TEMPLATE.as_bytes()).unwrap();

    Command::new("systemctl")
        .args(["restart", "xochitl"])
        .output().unwrap();
}

async fn print_job(buf: &mut BytesMut, state: JobHandler) {
    let job_id = state.add_job().await;
    
    buf.put_u8(DelimiterTag::OperationAttributes as u8);
    
    send_attribute(ValueTag::Charset, "attributes-charset", "utf-8", buf);
    send_attribute(ValueTag::NaturalLanguage, "attributes-natural-language", "en-us", buf);

    write_job(job_id, buf);
}

fn validate_job(buf: &mut BytesMut) {
    buf.put_u8(DelimiterTag::OperationAttributes as u8);
    
    send_attribute(ValueTag::Charset, "attributes-charset", "utf-8", buf);
    send_attribute(ValueTag::NaturalLanguage, "attributes-natural-language", "en-us", buf);
}



async fn send_jobs(buf: &mut BytesMut, state: JobHandler) {
    let jobs = state.fetch_jobs().await;

    buf.put_u8(DelimiterTag::OperationAttributes as u8);
    

    println!("Jobs: {:?}", &jobs);

    for job in jobs {
        write_job(job, buf);
    }
}

async fn send_job(buf: &mut BytesMut, job_id: u32, state: JobHandler) {
    let jobs = state.fetch_jobs().await;

    buf.put_u8(DelimiterTag::OperationAttributes as u8);
    

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

    buf.put_u8(DelimiterTag::JobAttributes as u8);
    send_attribute(ValueTag::Charset, "attributes-charset", "utf-8", buf);
    send_attribute(ValueTag::NaturalLanguage, "attributes-natural-language", "en-us", buf);
    send_attribute(ValueTag::Integer, "job-id", job_id.as_str(), buf);
    
    send_attribute(ValueTag::Uri, "job-uri", job_uri.as_str(), buf);
    

    send_attribute(ValueTag::Enum, "job-state", "\x00\x00\x00\x09", buf);
    send_attribute(ValueTag::Keyword, "job-state-reasons", "job-completed-successfully", buf);
    send_attribute(ValueTag::NameWithoutLanguage, "job-name", "com.google.android.apps.photos.Image", buf);
    send_attribute(ValueTag::Uri, "printer-uri", "ipp://192.168.0.10/remarkable-printer", buf);                
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

    buf.put_u8(DelimiterTag::EndOfAttributes as u8);

    buf.freeze()
}


#[derive(Clone)]
struct IppHandler {
    job_handler: JobHandler
}

async fn reply(state: JobHandler, request: Request<hyper::body::Incoming>) -> Result<Response<Full<Bytes>>, Infallible> {
    println!("{} {}", request.method(), request.uri().path());
    
    let mut a = request.collect().await.unwrap().aggregate();

    let message = parse(&mut a).unwrap();

    let job_id = message.attributes.iter().find(|a| a.name == "job-id").map(|a| match &a.value {
        AttributeValue::Other(ValueTag::Integer, b) => {
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
