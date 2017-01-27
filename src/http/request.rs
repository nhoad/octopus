extern crate httparse;
extern crate url;

use std::str;

use super::headers::Headers;

#[derive(Debug)]
pub struct Request {
    pub method: String,
    pub url: url::Url,
    pub version: u8,
    pub headers: Headers,
}

/// Attempt to parse a Request object from a given buffer.
///
/// The returned Vec<u8> contains any leftover data from the buffer that was
/// not parsed as the request, i.e. you should treat it as the beginning of the
/// request body.
pub fn parse<'a>(buffer: &'a Vec<u8>, mut headers: &mut [httparse::Header<'a>], total_read: usize) -> Result<Option<(Request, Vec<u8>)>, String> {
    let mut request = httparse::Request::new(&mut headers);

    let res = match request.parse(&buffer) {
        Ok(res) => res,
        Err(e) => {
            let error = format!("{:?}", e);
            return Err(error);
        },
    };

    match res {
        httparse::Status::Complete(n) => {
            let body = buffer[n..total_read].iter().cloned().collect();

            match Request::from_raw(request) {
                Ok(request) => {
                    return Ok(Some((request, body)));
                },
                Err(e) => {
                    return Err(e);
                }
            }
        },
        httparse::Status::Partial => {
            Ok(None)
        }
    }
}

impl Into<Vec<u8>> for Request {
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

impl Request {
    pub fn from_raw(request: httparse::Request) -> Result<Request, String> {
        let headers = Headers::from_raw(request.headers).unwrap();

        let url = match url::Url::parse(&request.path.unwrap()) {
            Ok(url) => url,
            Err(url::ParseError::RelativeUrlWithoutBase) => {
                let mut absolute_url = Vec::new();

                // FIXME: from the listening port, tell if it's secure or not for
                // the correct scheme.
                let secure = false;
                if secure {
                    absolute_url.extend("https://".as_bytes());
                } else {
                    absolute_url.extend("http://".as_bytes());
                }

                match headers.get("Host") {
                    Some(host) => absolute_url.extend(host),
                    None => {
                        return Err("Host header missing".to_owned());
                    }
                }

                absolute_url.extend(request.path.unwrap().as_bytes());

                let absolute_url = str::from_utf8(&absolute_url).unwrap();

                match url::Url::parse(&absolute_url) {
                    Ok(url) => url,
                    Err(e) => {
                        return Err(format!("Could not parse {}: {}", absolute_url, e).to_owned());
                    }
                }
            },
            Err(e) => {
                return Err(format!("Could not parse {}: {}", request.path.unwrap(), e).to_owned());
            }
        };

        Ok(Request {
            headers: headers,
            url: url,
            method: String::from(request.method.unwrap()),
            version: request.version.unwrap(),
        })
    }
}

#[cfg(test)]
mod tests {
    extern crate httparse;

    #[test]
    fn test_parse_on_relative_url() {
        use super::parse;

        let buf = b"GET / HTTP/1.1\r\nHost: google.com\r\n\r\nHello".to_vec();
        let mut headers = [httparse::EMPTY_HEADER; 16];
        let total_read = buf.len();

        let (req, body) = parse(&buf, &mut headers, total_read).unwrap().unwrap();

        assert_eq!(req.headers.get("Host").unwrap(), b"google.com");
        assert!(req.headers.get("Foo").is_none());

        assert_eq!(req.method, "GET");
        assert_eq!(req.version, 1u8);

        assert_eq!(body.len(), 5);
        assert_eq!(body, b"Hello");

        assert_eq!(req.url.as_str(), "http://google.com/");
    }

    #[test]
    fn test_parse_on_absolute_url() {
        use super::parse;
        let buf = b"GET http://google.com/ HTTP/1.1\r\n\r\nHello".to_vec();
        let mut headers = [httparse::EMPTY_HEADER; 16];
        let total_read = buf.len();

        let (req, _) = parse(&buf, &mut headers, total_read).unwrap().unwrap();
        assert_eq!(req.url.as_str(), "http://google.com/");
    }

    #[test]
    fn test_parse_on_relative_url_without_host() {
        use super::parse;
        let buf = b"GET / HTTP/1.1\r\n\r\nHello".to_vec();
        let mut headers = [httparse::EMPTY_HEADER; 16];
        let total_read = buf.len();

        assert!(parse(&buf, &mut headers, total_read).is_err());
    }


    #[test]
    fn test_parse_on_nonhttp() {
        use super::parse;
        let buf = b"frozen brains tell no tales\r\n".to_vec();
        let mut headers = [httparse::EMPTY_HEADER; 16];
        let total_read = buf.len();

        assert!(parse(&buf, &mut headers, total_read).is_err());
    }
}
