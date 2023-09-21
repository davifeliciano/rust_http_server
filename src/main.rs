use std::{
    fs,
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream},
    path::Path,
    thread,
    time::Duration,
};

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        println!("Connection established!");
        handle_connection(stream);
    }
}

fn build_response<P: AsRef<Path>>(path: P, status_code: u16) -> String {
    let first_line = format!("HTTP/1.1 {status_code} OK");
    let body = fs::read_to_string(path).unwrap();
    let length = body.len();

    format!("{first_line}\r\nContent-Length: {length}\r\n\r\n{body}")
}

fn handle_connection(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&mut stream);
    let request_first_line = buf_reader.lines().next().unwrap().unwrap();

    let (filename, status_code) = match &request_first_line.split(' ').collect::<Vec<_>>()[..] {
        ["GET", "/", "HTTP/1.1"] => ("200.html", 200),
        ["GET", "/sleep", "HTTP/1.1"] => {
            thread::sleep(Duration::from_secs(5));
            ("200.html", 200)
        }
        ["GET", _, "HTTP/1.1"] => ("404.html", 404),
        _ => ("400.html", 400),
    };

    let response = build_response(filename, status_code);
    stream.write_all(response.as_bytes()).unwrap();
}
