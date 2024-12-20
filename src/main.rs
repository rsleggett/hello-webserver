use std::{
    env, fs, io::{prelude::*, BufReader }, net::{ TcpListener, TcpStream }, thread, time::Duration
};
use rayon::ThreadPoolBuilder;
use hello_webserver::ThreadPool;

fn main() {
    let use_rayon = env::var("USE_RAYON").is_ok();
    let listener: TcpListener = TcpListener::bind("127.0.0.1:7878").unwrap();
    match use_rayon {
        true => run_rayon_server(listener),
        false => run_custom_server(listener)
    };
}

fn run_custom_server(listener: TcpListener) {
    let pool = ThreadPool::build(4).unwrap_or_else(|_e| { panic!("PoolCreationError"); });
    for stream in listener.incoming() {
        let stream = stream.unwrap();
        
        pool.execute(|| {
            handle_connection(stream);
        });
    }
}

fn run_rayon_server(listener: TcpListener) {
    let pool = ThreadPoolBuilder::new().num_threads(4).build().unwrap();
    for stream in listener.incoming() {
        let stream = stream.unwrap();
        
        pool.spawn(|| {
            handle_connection(stream);
        });
    }
}

fn handle_connection(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&mut stream);
    let mut http_request = buf_reader.lines();
    let request_line = http_request.next().unwrap().unwrap();

    let (status_line, filename) = match &request_line[..] {
        "GET / HTTP/1.1" => ("HTTP/1.1 200 OK", "hello.html"),
        "GET /sleep HTTP/1.1" => {
            thread::sleep(Duration::from_secs(10));
            ("HTTP/1.1 200 OK", "hello.html")
        },
        _ => ("HTTP/1.1 404 Not Found", "404.html")
    };

    let contents = fs::read_to_string(filename).unwrap();
    let length = contents.len();
    let response = format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");
    stream.write_all(response.as_bytes()).unwrap();
}
