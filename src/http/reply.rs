extern crate httparse;

use super::headers::Headers;

#[derive(Debug)]
pub struct Reply {
    pub version: u8,
    pub code: u16,
    pub reason: String,
    pub headers: Headers,
}

impl Reply {
    pub fn from_raw(response: httparse::Response) -> Reply {
        let headers = Headers::from_raw(response.headers).unwrap();

        Reply {
            version: response.version.unwrap(),
            code: response.code.unwrap(),
            reason: String::from(response.reason.unwrap()),
            headers: headers,
        }
    }
}
