use std::{
    env,
    error::Error,
    fs,
    io::{prelude::*, BufReader},
    net::{Ipv4Addr, SocketAddrV4, TcpListener, TcpStream},
    num::NonZeroUsize,
    path::Path,
    thread,
    time::Duration,
};

use http_server::ThreadPool;

fn main() -> Result<(), Box<dyn Error>> {
    let port: u16 = env::var("PORT").unwrap_or("7878".to_string()).parse()?;
    let socket = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), port);
    let listener = TcpListener::bind(socket)?;
    let thread_count = thread::available_parallelism()
        .unwrap_or(NonZeroUsize::new(1).unwrap())
        .get();

    let pool = ThreadPool::build(thread_count)?;

    println!("Spawned pool with {thread_count} threads");

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        pool.execute(|| {
            handle_connection(stream);
        });
    }

    Ok(())
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
