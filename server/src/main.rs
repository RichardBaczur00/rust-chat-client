use std::io::{ErrorKind, Read, Write};
use std::net::TcpListener;
use std::sync::mpsc;
use std::thread;


const LOCAL: &str = "127.0.0.1:6000";
const BUFF_MAX_SIZE: usize = 128;


fn sleep() {
    thread::sleep(std::time::Duration::from_millis(100));
}


fn main() {
    let server = TcpListener::bind(LOCAL).expect("Listener failed to bind!");
    server.set_nonblocking(true).expect("failed to initialize non-blocking begaviour");

    let mut clients = vec![];
    let (tx, rx) = mpsc::channel::<String>();
    loop {
        if let Ok((mut socket, addr)) = server.accept() {
            println!("Client {} connected", addr);

            let tx = tx.clone();
            clients.push(socket.try_clone().expect("Failed to clone client!"));

            std::thread::spawn(move || loop {
                let mut buff: Vec<u8> = vec![0, BUFF_MAX_SIZE as u8];

                match socket.read_exact(&mut buff) {
                    Ok(_) => {
                        let msg = buff.into_iter().take_while(|&x| x != 0).collect::<Vec<_>>();
                        let msg = String::from_utf8(msg).expect("Invalid utf-8 message!");

                        println!("{}: {:?}", addr, msg);
                        tx.send(msg).expect("Failed to send msg to rx");
                    }
                    Err(ref err) if err.kind() == ErrorKind::WouldBlock => (),
                    Err(_) => {
                        println!("Closing connection with: {}", addr);
                        break;
                    }
                }

                sleep();
            });
        }

        if let Ok(msg) = rx.try_recv() {
            clients = clients.into_iter().filter_map(|mut client| {
                let mut buff = msg.clone().into_bytes();
                buff.resize(BUFF_MAX_SIZE, 0);

                client.write_all(&buff).map(|_| client).ok()
            }).collect::<Vec<_>>();
        }
    }
}
