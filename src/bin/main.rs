use rust_web_server::ThreadPool;
use std::fs;
use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::time::Duration;

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};

    #[test]
    fn end2end_test() {
        // An iterator that infinitely accepts connections
        let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
        // create the pool
        let pool = ThreadPool::new(3).unwrap();
        // create a client socket
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 7878);
        // Create a bunch of dummy requests, one is long (the sleep one)
        let n_requests = 6;
        let mut vec_requests = Vec::with_capacity(n_requests);
        vec_requests.push(String::from("GET /sleep HTTP/1.1\r\n\r\n"));
        vec_requests.push(String::from("GET /sleep HTTP/1.1\r\n\r\n"));
        vec_requests.push(String::from("GET /sleep HTTP/1.1\r\n\r\n"));
        vec_requests.push(String::from("GET / HTTP/1.1\r\n\r\n"));
        vec_requests.push(String::from("GET / HTTP/1.1\r\n\r\n"));
        vec_requests.push(String::from("GET / HTTP/1.1\r\n\r\n"));

        // we wanna send a job to the pool for each request
        for a_request in vec_requests {
            // client connects to tcp socket
            // mut b/c it handles its own state like a responsible adult
            // TODO it's probably not necessary to create a new one for each request...
            let mut stream = TcpStream::connect(&addr).unwrap();

            // just panic if you can't write, something is wrong..
            assert!(
                // client writes a request to the socket
                !stream.write(a_request.as_bytes()).is_err(),
                "Panic! Err when writing query to tcp stream in Test"
            );
            // diss iz da key: the listener (who's been listening the whole time) MUST accept the
            // incoming data/requests AND it owns the socket clients
            match listener.accept() {
                // if no issue with connection, a client/socket (TcpStream) is returned AND the data that
                // was written into it IS available to read, so the latter won't be blocking
                // so whatever happens in the closure and in handle_connection, reading will be OK
                // Note: a new stream AND a new _socket are created at each loop iteration, so no pb
                // of borrowing a moved value.
                Ok((_socket, addr)) => {
                    println!("new client: {:?}", addr);
                    pool.execute(|| {
                        handle_connection(_socket);
                    });
                }
                Err(e) => println!("couldn't get client: {:?}", e),
            }
        }
    }
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    let pool = ThreadPool::new(3).unwrap();

    // limit to 20 requests
    for stream in listener.incoming().take(20) {
        let stream = stream.unwrap();

        pool.execute(|| {
            handle_connection(stream);
        });
    }
}

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 512];

    stream.read(&mut buffer).unwrap();

    let get = b"GET / HTTP/1.1\r\n";
    let sleep = b"GET /sleep HTTP/1.1\r\n";

    let (status_line, filename) = if buffer.starts_with(get) {
        ("HTTP/1.1 200 OK\r\n\r\n", "html/hello.html")
    } else if buffer.starts_with(sleep) {
        thread::sleep(Duration::from_secs(11));
        ("HTTP/1.1 200 OK\r\n\r\n", "html/hello.html")
    } else {
        ("HTTP/1.1 404 NOT FOUND\r\n\r\n", "html/404.html")
    };
    let contents = fs::read_to_string(filename).unwrap();
    let response = format!("{}{}", status_line, contents);

    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}
