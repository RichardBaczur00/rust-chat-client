use std::io::{self, ErrorKind, Read, Write};
use std::net::TcpStream;
use std::sync::mpsc::{self, TryRecvError};
use std::thread;
use std::time::Duration;


const LOCAL: &str = "127.0.0.1:6000";
const BUFF_MAX_SIZE: usize = 128;


fn sleep() {
    thread::sleep(Duration::from_millis(100));
}



fn main() {
    let mut client = TcpStream::connect(LOCAL).expect("Stream failed to connect.");
    client.set_nonblocking(true).expect("Failed to set non-blocking behaviour.");

    let (tx, rx) = mpsc::channel::<String>();

    std::thread::spawn(move || loop {
        let mut buff: Vec<u8> = vec![0, BUFF_MAX_SIZE as u8];
        match client.read_exact(&mut buff) {
            Ok(_) => {
                let msg = buff.into_iter().take_while(|&x| x != 0).collect::<Vec<_>>();
                if msg.len() != 0 {
                    println!("message recv {:?}", msg);
                }
            }
            Err(ref err) if err.kind() == ErrorKind::WouldBlock => (),
            Err(_) => {
                println!("Connection with server was lost!");
                break;
            }
        }

        match rx.try_recv() {
            Ok(msg) => {
                let mut buff = msg.clone().into_bytes();
                buff.resize(BUFF_MAX_SIZE, 0);
                client.write_all(&buff).expect("Writting to socket failed!");
                println!("Message sent {:?}", msg);
            }
            Err(TryRecvError::Empty) => (),
            Err(TryRecvError::Disconnected) => break
        }

        sleep();
    });

    println!("Write a Message: ");
    loop {
        let mut buff = String::new();
        io::stdin().read_line(&mut buff).expect("Reading from stdin failed");
        let msg = buff.trim().to_string();
        if msg != "" {
            if msg == ":q" || tx.send(msg).is_err() {break; }
        }
    }
    println!("Bye!");
}
