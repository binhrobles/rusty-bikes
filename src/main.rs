use std::{
    fmt,
    fs,
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream},
    thread,
    time::Duration,
    collections::HashMap,
};
// use rusty_router::ThreadPool;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    println!("Listening on http://localhost:7878");
    // let pool = ThreadPool::new(4);

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        handle_connection(stream);
    }
}

struct Response {
    status_code: u16,
    content: Option<String>,
    cors: bool,
}

impl fmt::Display for Response {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "HTTP/1.1 {} {}", self.status_code, match self.status_code {
            200 => "OK",
            204 => "No Content",
            400 => "Not Found",
            _ => "Unknown",
        })?;

        if self.cors {
            write!(f, "\r\nAccess-Control-Allow-Headers: Access-Control-Allow-Headers, Origin,Accept, X-Requested-With, Authorization, Content-Type, Access-Control-Request-Method, Access-Control-Request-Headers")?;
            write!(f, "\r\nAccess-Control-Allow-Methods: OPTIONS,GET,POST")?;
            write!(f, "\r\nAccess-Control-Allow-Origin: *")?;
            write!(f, "\r\nAccess-Control-Max-Age: 1728000")?;
        }

        if let Some(c) = &self.content {
            write!(f, "\r\nContent-Length: {}\r\n\r\n{c}", c.len())?;
        }

        Ok(())
    }
}

fn handle_options_request() -> String {
    Response { status_code: 204, content: None, cors: true }.to_string()
}

fn handle_default(status_code: u16, filename: &str, cors: bool) -> String {
    let contents = fs::read_to_string(filename).unwrap();
    Response { status_code, content: Some(contents), cors }.to_string()
}

fn handle_connection(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&mut stream);
    let request_line = buf_reader.lines().next().unwrap().unwrap();
    println!("Request: {:#?}", request_line);

    let response = match &request_line[..] {
        "OPTIONS /v2/directions/cycling-regular/geojson HTTP/1.1" => handle_options_request(),
        "POST /v2/directions/cycling-regular/geojson HTTP/1.1" => handle_default(200, "staticGeoJsonResponse.geojson", true),
        _ => handle_default(404, "404.html", false),
    };

    stream.write_all(response.as_bytes()).unwrap();
}
