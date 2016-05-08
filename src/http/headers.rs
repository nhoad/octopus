extern crate httparse;

use std::str;
use std::collections::{HashMap, LinkedList};
use std::collections::hash_map::Entry;

#[derive(Debug)]
pub struct Headers {
    // FIXME: case insensitivity
    data: HashMap<String, LinkedList<Vec<u8>>>,
}

impl Headers {
    pub fn new() -> Headers {
        Headers {
            data: HashMap::new(),
        }
    }

    pub fn from_raw(raw: &[httparse::Header]) -> Headers {
        let mut headers = Headers::new();

        for header in raw {
            headers.insert(header.name, header.value.iter().cloned().collect());
        }

        // TODO: while headers are still mutable, iterate through and complain
        // about weird situations, e.g. no Host header, or two Content-Length headers.

        headers
    }

    pub fn content_length(&self) -> Option<usize> {
        match self.get("Content-Length") {
            Some(value) => {
                Some(str::from_utf8(&value).unwrap().parse().unwrap())
            },
            None => None
        }
    }

    pub fn get(&self, name: &str) -> Option<Vec<u8>> {
        match self.data.get(name) {
            Some(values) => {
                Some(values.front().unwrap().clone())
            },
            None => None
        }
    }

    pub fn insert(&mut self, name: &str, value: Vec<u8>) {
        let mut item = match self.data.entry(String::from(name)) {
            Entry::Occupied(entry) => {
                entry.into_mut()
            },
            Entry::Vacant(entry) => {
                entry.insert(LinkedList::new())
            },
        };

        item.push_back(value);
    }

}

impl Into<Vec<u8>> for Headers {
    fn into(self) -> Vec<u8> {
        let mut out = Vec::<u8>::with_capacity(65536);

        for (name, values) in &self.data {
            for value in values {
                out.extend(name.as_bytes());
                out.extend(b": ");
                out.extend(value);
                out.extend(b"\r\n");
            }
        }
        out
    }
}

#[test]
fn test_headers() {
    let mut headers = Headers::new();

    let value: Vec<u8> = "google.com".as_bytes().iter().cloned().collect();

    headers.insert("Host", value);

    let value: Vec<u8> = "google.com".as_bytes().iter().cloned().collect();

    assert_eq!(headers.get("Host"), Some(value));
    assert_eq!(headers.get("Most"), None);
}


