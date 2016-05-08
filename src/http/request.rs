extern crate httparse;
extern crate url;

use std::str;

use super::headers::Headers;

#[derive(Debug)]
pub struct Request<'buf> {
    pub method: &'buf str,
    pub url: url::Url,
    pub version: u8,
    pub headers: Headers<'buf>,
}

impl<'buf> Into<Vec<u8>> for Request<'buf> {
    fn into(self) -> Vec<u8> {
        let mut out = Vec::<u8>::with_capacity(65536);

        let reqline = format!("{} {} HTTP/1.{}\r\n", self.method, self.url.path(), self.version);
        out.extend(reqline.as_bytes());
        let headers: Vec<u8> = self.headers.into();
        out.extend(headers);
        out.extend(b"\r\n");
        println!("Returning {:?}", out);
        out
    }
}

impl<'buf, 'headers> Request<'buf> {
    pub fn from_raw(request: httparse::Request<'buf, 'headers>) -> Request<'buf> {
        let path = request.path.unwrap();
        let mut url = Vec::new();
        let headers = Headers::from_raw(request.headers);

        if url_is_relative(path) {
            // FIXME: from the listening port, tell if it's secure or not for
            // the correct scheme.
            let secure = false;
            if secure {
                url.extend("https://".as_bytes());
            } else {
                url.extend("http://".as_bytes());
            }

            // FIXME: handle Host header missing
            url.extend(headers.get("Host").unwrap());
        }
        url.extend(path.as_bytes());

        Request {
            headers: headers,
            url: url::Url::parse(str::from_utf8(&url).unwrap()).unwrap(),
            method: request.method.unwrap(),
            version: request.version.unwrap(),
        }
    }
}

fn url_is_relative(url: &str) -> bool {
    let colon = url.find(':');
    let slash = url.find('/');

    match slash {
        Some(slash) => {
            // we have a slash!

            match colon {
                Some(colon) => {
                    slash < colon
                },
                None => {
                    // we don't have a colon, can't be absolute
                    true
                }
            }
        },
        None => {
            // no slash, can't be relative
            false
        }
    }
}

