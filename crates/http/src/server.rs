use std::io::{self, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use crate::pool::ThreadPool;
use crate::request::{self, ParseError};
use crate::response::{Response, StatusCode};
use crate::router::Router;

const READ_TIMEOUT: Duration = Duration::from_secs(5);

pub fn serve(router: Router, port: u16) -> io::Result<()> {
    let listener = TcpListener::bind(("0.0.0.0", port))?;
    let workers = thread::available_parallelism().map_or(4, |n| n.get()) * 4;
    let pool = ThreadPool::new(workers);
    let router = Arc::new(router);

    println!("forge: listening on http://localhost:{port} ({workers} workers)");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let router = Arc::clone(&router);
                pool.execute(move || {
                    if let Err(e) = handle_connection(stream, &router) {
                        eprintln!("forge: connection error: {e}");
                    }
                });
            }
            Err(e) => eprintln!("forge: failed to accept connection: {e}"),
        }
    }
    Ok(())
}

fn handle_connection(stream: TcpStream, router: &Router) -> io::Result<()> {
    stream.set_read_timeout(Some(READ_TIMEOUT))?;
    let mut reader = BufReader::new(stream.try_clone()?);

    let response = match request::parse(&mut reader) {
        Ok(req) => {
            catch_unwind(AssertUnwindSafe(|| router.handle(req))).unwrap_or_else(|_| {
                Response::error(StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
            })
        }
        Err(ParseError::Io(e)) => return Err(e),
        Err(ParseError::Malformed(msg)) => Response::error(StatusCode::BAD_REQUEST, msg),
        Err(ParseError::TooLarge) => {
            Response::error(StatusCode::PAYLOAD_TOO_LARGE, "Request too large")
        }
        Err(ParseError::Unsupported(msg)) => Response::error(StatusCode::NOT_IMPLEMENTED, msg),
    };

    let mut stream = stream;
    response.write_to(&mut stream)?;
    stream.flush()
}
