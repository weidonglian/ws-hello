use std::fs;
use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

type Counter = std::sync::atomic::AtomicUsize;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    let counter = Arc::new(Counter::new(0));
    for stream in listener.incoming() {
        let stream = stream.unwrap();
        let counter = Arc::clone(&counter);
        std::thread::spawn(move || {
            handle_connection(stream, counter);
        });
    }
    println!("Hello, world!");
}

fn handle_connection(mut stream: TcpStream, counter: Arc<Counter>) {
    let mut buffer = [0; 2048];
    let buffer_size = stream.read(&mut buffer).unwrap();
    let request = String::from_utf8_lossy(&buffer[..buffer_size]);
    println!("Request: {}", request);
    let (status_line, filename) = if request.starts_with("GET / HTTP") {
        ("HTTP/1.1 200 OK\r\n\r\n", "hello.html")
    } else if request.starts_with("GET /sleep HTTP") {
        thread::sleep(Duration::from_secs(6));
        ("HTTP/1.1 200 OK\r\n\r\n", "hello.html")
    } else {
        ("HTTP/1.1 404 NOT FOUND\r\n\r\n", "404.html")
    };

    let contents = fs::read_to_string(filename).unwrap();
    let contents = contents.replace(
        "${counter}",
        format!("{}", counter.load(Ordering::SeqCst)).as_str(),
    );
    let response = format!("{}{}", status_line, contents);

    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
    counter.fetch_add(1, Ordering::SeqCst);
}
