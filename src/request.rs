extern crate httparse;
extern crate url;

use std::str;

use ::headers::Headers;

#[derive(Debug)]
pub struct Request<'buf> {
    pub url: url::Url,
    pub method: &'buf str,
    pub version: u8,
    pub headers: Headers<'buf>,
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
            // FIXME: need to handle gluing the Host header to the URL if it's
            // relative.
            url: url::Url::parse(str::from_utf8(&url).unwrap()).unwrap(),
            method: request.method.unwrap(),
            version: request.version.unwrap(),
        }
    }
}

