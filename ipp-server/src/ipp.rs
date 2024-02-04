use bytes::{Buf, BufMut, Bytes, BytesMut};
use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::FromPrimitive;  
use core::arch::asm;  


#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, FromPrimitive, ToPrimitive)]
pub enum DelimiterTag {
    OperationAttributes = 0x01,
    JobAttributes = 0x02,
    EndOfAttributes = 0x03,
    PrinterAttributes = 0x04,
    UnsupportedAttributes = 0x05,
}

#[derive(Debug, PartialEq, FromPrimitive)]
#[repr(u8)]
pub enum ValueTag {
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
        DelimiterTag::from_u8(tag)
    }
}

impl ValueTag {
    fn parse_tag(tag: u8) -> Option<Self> {
        let result = ValueTag::from_u8(tag);
        if let None = result { println!("Encountered unknown tag {}", tag); };
        result
    }
}

#[derive(Debug)]
pub enum AttributeValue {
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
pub struct IncomingMessage {
    pub version_major: u8,
    pub version_minor: u8,
    pub operation_id: u16,
    pub request_id: u32,
    attributes: Vec<Attribute>
}

impl IncomingMessage {
    pub fn get_attribute(&self, name: &str) -> Option<&AttributeValue> {
        for attr in &self.attributes {
            if attr.name == name {
                return Some(&attr.value);
            }
        }
        None
    }

    pub fn get_delimited_attribute(&self, kind: DelimiterTag, name: &str) -> Option<&AttributeValue> {
        for attr in &self.attributes {
            if attr.kind == kind && attr.name == name {
                return Some(&attr.value);
            }
        }
        None
    }
}

pub fn parse(b: &mut impl Buf) -> Option<IncomingMessage> {
    let version_major = b.get_u8();
    let version_minor = b.get_u8();
    let operation_id = b.get_u16();
    let request_id = b.get_u32();
    
    let attributes = parse_attributes(b);
    
    Some(IncomingMessage {
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

pub fn send_attribute(tpe: ValueTag, name: &str, value: &str, buf: &mut BytesMut) {
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
