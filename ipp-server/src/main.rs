use std::convert::Infallible;
//use std::io::Write;
//use tokio::io::AsyncWriteExt;
use std::net::SocketAddr;
use std::fs::File;

use http_body_util::{BodyExt, Full};
use hyper::body::{Buf, Bytes};
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response};
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;

use std::io::Write;



#[derive(Debug)]
struct IppMessage {
    operation_id: u16,
    request_id: u32,
}



trait Binary {
    fn from_buf(buf: &mut impl Buf) -> Option<Self> where Self: Sized;
}

enum DelimiterTag {
    OperationAttributes = 0x01,
    JobAttributes = 0x02,
    EndOfAttributes = 0x03,
    PrinterAttributes = 0x04,
    UnsupportedAttributes = 0x05,
}

#[derive(Debug)] 
#[derive(PartialEq)]
enum ValueTag {
    JobAttributes = 0x02,
    EndOfAttributes = 0x03,
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

impl Binary for DelimiterTag {
    fn from_buf(buf: &mut impl Buf) -> Option<Self> {
        let tag = buf.get_u8();
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

impl Binary for ValueTag {
    fn from_buf(buf: &mut impl Buf) -> Option<Self> {
        let tag = buf.get_u8();
        match tag {
            0x02 => Some(ValueTag::JobAttributes),
            0x03 => Some(ValueTag::EndOfAttributes),
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

fn parse(b: &mut impl Buf) -> Option<IppMessage> {
    let version_major = b.get_u8();
    let version_minor = b.get_u8();
    println!("Version: {}.{}", version_major, version_minor);
    if version_major != 2 {
        return None;
    }
    let operation_id = b.get_u16();
    let request_id = b.get_u32();
    
    let _ipp_message = parseAttributeGroups(b);

    let data_size = b.remaining();
    println!("Data size: {}", data_size);

    println!("OperationId: {}", operation_id);
    println!("RequestId: {}", request_id);
    
    if data_size > 0 {
        let mut file = File::create("foo.txt").unwrap();
        file.write(&b.copy_to_bytes(data_size)).unwrap();
    }
    
    Some(IppMessage {
        operation_id,
        request_id,
    })
}

fn parseAttributeGroups(b: &mut impl Buf) {
    let tag: Option<DelimiterTag> = DelimiterTag::from_buf(b);
    match tag {
        Some(DelimiterTag::EndOfAttributes) => {
            println!("EndOfAttributes");
        },

        Some(DelimiterTag::OperationAttributes) => {
            println!("OperationAttributes");
            parseAttributes(b);
        },
        Some(DelimiterTag::JobAttributes) => {
            println!("JobAttributes");
            parseAttributes(b);
        },
        Some(DelimiterTag::PrinterAttributes) => {
            println!("PrinterAttributes");
            parseAttributes(b);
        },
        Some(DelimiterTag::UnsupportedAttributes) => {
            println!("UnsupportedAttributes");
            parseAttributes(b);
        },
        None => {
            println!("Unknown tag");
            parseAttributes(b);
        },
    }
}

fn parseAttributes(b: &mut impl Buf) {
    loop {
        let tag = ValueTag::from_buf(b);
        if tag == Some(ValueTag::EndOfAttributes) {
            break;
        }
        if tag == Some(ValueTag::JobAttributes) {
            println!("JobAttributes");
            continue;
        }
        let name_length = b.get_u16();
        let name = b.copy_to_bytes(name_length as usize);
        let value_length = b.get_u16();
        let value = b.copy_to_bytes(value_length as usize);
        println!("Tag: {:?}, Name: {}, Value: {}", tag, String::from_utf8_lossy(&name), String::from_utf8_lossy(&value));
    }
}

fn sendAttribute(tpe: ValueTag, name: &str, value: &str, buf: &mut Vec<u8>) {
    buf.push(tpe as u8);

    let name = name.as_bytes();
    let value = value.as_bytes();

    let name_length = name.len() as u16;
    let value_length = value.len() as u16;

    buf.extend_from_slice(&name_length.to_be_bytes());
    buf.extend_from_slice(name);
    buf.extend_from_slice(&value_length.to_be_bytes());
    buf.extend_from_slice(value);
}

fn send_document_type(buf: &mut Vec<u8>) {
    //buf.push(DelimiterTag::OperationAttributes as u8);

    //sendAttribute(ValueTag::Charset, "attributes-charset", "utf-8", buf);
    // sendAttribute(ValueTag::NaturalLanguage, "attributes-natural-language", "en-us", buf);
    
    buf.push(DelimiterTag::PrinterAttributes as u8);
    sendAttribute(ValueTag::MimeMediaType, "document-format-supported", "application/pdf", buf);
    sendAttribute(ValueTag::MimeMediaType, "document-format-default", "application/pdf", buf);
    // sendAttribute(ValueTag::Keyword, "compression-supported", "none", buf);
    // sendAttribute(ValueTag::Boolean, "printer-is-accepting-jobs", "\x01", buf);
    // sendAttribute(ValueTag::Enum, "printer-state", "\x00\x00\x00\x03", buf);
    // sendAttribute(ValueTag::Keyword, "printer-state-reasons", "none", buf);
    
    sendAttribute(ValueTag::Uri, "printer-uri-supported", "ipp://192.168.0.10/remarkable-printer", buf);
    sendAttribute(ValueTag::NameWithoutLanguage, "printer-name", "remarkable-test-printer", buf);
    sendAttribute(ValueTag::Keyword, "uri-authentication-supported", "none", buf);
    sendAttribute(ValueTag::Keyword, "uri-security-supported", "none", buf);
    
    // sendAttribute(ValueTag::Enum, "operations-supported", "\x00\x00\x00\x02", buf);
    // sendAttribute(ValueTag::Enum, "", "\x00\x00\x00\x04", buf);
    
    
}

fn response(status_code: u16, request_id: u32) -> Vec<u8> {
    let mut buf: Vec<u8> = Vec::new();

    buf.push(0x02);
    buf.push(0x00);
    buf.extend_from_slice(&status_code.to_be_bytes());
    buf.extend_from_slice(&request_id.to_be_bytes());

    buf.push(DelimiterTag::OperationAttributes as u8);

    send_document_type(&mut buf);

    buf.push(DelimiterTag::EndOfAttributes as u8);

    buf
}


async fn hello(request: Request<hyper::body::Incoming>) -> Result<Response<Full<Bytes>>, Infallible> {

    println!("{} {}", request.method(), request.uri().path());

    let mut a = request.collect().await.unwrap().aggregate();

    let message = parse(&mut a).unwrap();

    println!("{:#?}", message);

    let r = response(0, message.request_id);
    println!("Success");
    Ok(Response::new(Full::new(Bytes::from(r))))

}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("Hello, world!");
    let addr = SocketAddr::from(([0, 0, 0, 0], 631));

    // We create a TcpListener and bind it to 127.0.0.1:631
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