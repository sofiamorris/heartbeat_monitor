mod connection;
use connection::stream_read;

use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::io::Write;
use std::thread;
use std::time::{Duration, Instant};
use threadpool::ThreadPool;
use std::env;

fn listener(host: &str) -> std::io::Result<()>{

    let listener = TcpListener::bind(host).unwrap();
    println!("Bind successful");
    
    let pool = ThreadPool::new(2); //change to any thread limit

    for stream in listener.incoming() {
        println!("Request received on {:?}, processing...", stream);
        match stream {
            Ok(stream) => {
                let shared_stream = Arc::new(Mutex::new(stream));
                let _ = pool.execute(move || {
                    println!("Passing TCP connection to handler...");
                    let _ = handle_connection(& shared_stream);
                    println!("Connection handled");
                });
            }
            Err(e) => {
                eprintln!("Connection failed: {}", e);
            }
        }
        println!("moving to next incoming stream");
    }

    Ok(())
}

fn handle_connection(stream: & Arc<Mutex<TcpStream>>) -> std::io::Result<()>{
    println!("Starting heartbeat handler");

    let mut timer = Instant::now();
    let failure_duration = Duration::from_secs(10); //change to any failure limit

    loop{

        if timer.elapsed() > failure_duration{
            println!("Connection has died");
            return Ok(());
        }

        let loc_stream: &mut TcpStream = &mut *stream.lock().unwrap();
        loc_stream.set_read_timeout(Some(failure_duration))?;

        let received = match stream_read(loc_stream) {
            Ok(message) => message,
            Err(err) => {return Err(err);}
        };

        timer = Instant::now();

        let ack = "Received message";
        let _ = loc_stream.write(ack.as_bytes());

        println!("{}", received);
    }

    Ok(())
}


fn main() {

    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <name>", args[0]);
        std::process::exit(1);
    }

    if let Err(e) = listener(&args[1]) {
        eprintln!("Listener error: {}", e);
    }
}