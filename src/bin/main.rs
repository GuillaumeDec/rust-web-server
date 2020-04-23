use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::fs;
use std::thread;
use rust_web_server::ThreadPool;
use std::time::Duration;
use std::net::Shutdown;

/// http;
//use http::client::RequestWriter;
//use http::method::Get;
#[cfg(test)]
mod tests {
    use super::*;
    use http::Response;
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};

    ///
    /// let socket = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
    ///
    #[test]
    fn full_test() {
//        let _listener = TcpListener::bind("127.0.0.1:7878").unwrap();
        {
            let pool = ThreadPool::new(4).unwrap();
            // must be mut b/c we'll write stuff into it
            // must be an Option such that we can borrow later using take(), cf. chap 20
            // https://doc.rust-lang.org/book/ch20-03-graceful-shutdown-and-cleanup.html
            let mut a_stream: Option<TcpStream>;
            let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 7878);
//        a_stream = Some(TcpStream::connect_timeout(&addr, Duration::from_millis(1000)).unwrap());
            println!("gonna connect!!!!!!!!!");
            a_stream = Some(TcpStream::connect(&addr).unwrap());
            println!("just connected!!!!!!!!!");
            let mut vec_requests = Vec::with_capacity(2);

//            let request1 = RequestWriter::new(Get, FromStr::from_str("bla").unwrap());
            let foo = String::from("GET /sleep HTTP/1.1\r\n\r\n");
            let foo2 = String::from("GET / HTTP/1.1\r\n\r\n");
            let foo3 = String::from("GET / HTTP/1.1\r\n\r\n");

//            let foo = Response::builder().body("GET /sleep HTTP/1.1\r\n".to_string()).unwrap();
//        let foo2 = Response::builder().body("GET / HTTP/1.1\r\n".to_string()).unwrap();
//            let foo3 = Response::builder().body("GET / HTTP/1.1\r\n".to_string()).unwrap();
            vec_requests.push(foo);
            vec_requests.push(foo2);
            vec_requests.push(foo3);

            for a_request in vec_requests {
                println!("a_request! {:?}", a_request);
                a_stream = Some(TcpStream::connect(&addr).unwrap());
                if let Some(mut a_tcp_stream) = a_stream.take() {
//                let bytes_written = a_tcp_stream.write(a_request.as_bytes());

//                println!("has written {:?} to tcp stream: {:?}", bytes_written, a_request);
                    let mut bufferrr = [0; 25];
//                a_tcp_stream.flush();
//                println!("peek {:?}", a_tcp_stream.peek(&mut bufferrr));
//                a_tcp_stream.shutdown(Shutdown::Read);

                    let mut request_data = String::new();
                    request_data.push_str("GET / HTTP/1.0");
                    request_data.push_str("\r\n");
                    request_data.push_str("Host: 127.0.0.1:7878");
                    request_data.push_str("\r\n");
                    request_data.push_str("Connection: close"); // <== Here!
                    request_data.push_str("\r\n");
                    request_data.push_str("\r\n");

                    println!("request_data = {:?}", request_data);

                    let request = a_tcp_stream.write_all(request_data.as_bytes());
                    println!("request = {:?}", request);


                    pool.execute(|| {
                        handle_connection(a_tcp_stream);
                    });
//                a_tcp_stream.flush();
                }
            }
            println!("Outside of pool scope -- about to call Drop trait?");
        }
        println!("Outside of pool scope -- should have called Drop trait already");
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
    println!("ABT TO HANDLE CONNECTION!! ");
    let mut buffer = [0; 512];
    println!("ABT TO HANDLE CONNECTION2!! ");
//    stream.set_read_timeout(Some(Duration::from_millis(400))).unwrap();
//    stream.flush();
//    stream.re
//    stream.read_to_string().unwrap();

//    let mut buf = String::new();
//    let result = stream.read_to_string(&mut buf);
//    println!("result = yay", );
//    println!("buf = {}", buf);

    stream.read(&mut buffer).unwrap();
    println!("ABT TO HANDLE CONNECTION3!! ");

    let get = b"GET / HTTP/1.1\r\n";
    let sleep = b"GET /sleep HTTP/1.1\r\n";

    println!("WHAT WE GOT IN STREAM:  {:?}", buffer.to_ascii_lowercase());

    let (status_line, filename) = if buffer.starts_with(get) {
        println!("WHAT WE GOT IN STREAM: NOSLEEP ");
        ("HTTP/1.1 200 OK\r\n\r\n", "html/hello.html")
    } else if buffer.starts_with(sleep) {
        println!("WHAT WE GOT IN STREAM: SLEEP ");
        thread::sleep(Duration::from_secs(4));
        println!("WHAT WE GOT IN STREAM: DONE SLEEPING ");
        ("HTTP/1.1 200 OK\r\n\r\n", "html/hello.html")
    } else {
        println!("WHAT WE GOT IN STREAM: 404 ");
        ("HTTP/1.1 404 NOT FOUND\r\n\r\n", "html/404.html")
    };
    let contents = fs::read_to_string(filename).unwrap();
    let response = format!("{}{}", status_line, contents);
    println!("CONTENTS, RESP: {:?} {:?} ", contents, response);

    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
    println!("FLUSHED! for buffer {:?}", buffer.to_ascii_lowercase());
}
