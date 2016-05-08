extern crate httparse;

use super::headers::Headers;

#[derive(Debug)]
pub struct Reply<'buf> {
    pub version: u8,
    pub code: u16,
    pub reason: &'buf str,
    pub headers: Headers<'buf>,
}

impl<'buf, 'headers> Reply<'buf> {
    pub fn from_raw(response: httparse::Response<'buf, 'headers>) -> Reply<'buf> {
        let headers = Headers::from_raw(response.headers);

        Reply {
            version: response.version.unwrap(),
            code: response.code.unwrap(),
            reason: response.reason.unwrap(),
            headers: headers,
        }
    }
}
