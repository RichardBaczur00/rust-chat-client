use std::io::{ErrorKind, Read, Write};
use std::net::TcpListener;
use std::sync::mpsc;
use std::thread;
use std::collections::{HashMap, VecDeque};

struct Mesasge {
    source: String,
    msg: String
}

const LOCAL: &str = "127.0.0.1:6000";
const MSG_SIZE: usize = 32;
static mut CHAT_ROOMS: Vec<HashMap<String, String>> = Vec::new();
static mut CLIENT_LOOKUP: Vec<HashMap<String, usize>> = Vec::new();
static mut MESSAGE_QUEUE: Vec<Mesasge> = Vec::new();


fn sleep() {
    thread::sleep(::std::time::Duration::from_millis(100));
}

fn get_token(addr: String) -> Option<String> {
    if addr.len() <= 5 {
        return Some(addr)
    }
    let mut parts = addr.split(":");
    let _ = parts.next();
    println!("{}", addr);
    return Some(parts.next().unwrap().to_string())
}

fn main() {
    let server = TcpListener::bind(LOCAL).expect("Listener failed to bind");
    server.set_nonblocking(true).expect("failed to initialize non-blocking");
    unsafe {
        CHAT_ROOMS.push(HashMap::new());
        CLIENT_LOOKUP.push(HashMap::new());
    }

    let mut clients = vec![];
    let (tx, rx) = mpsc::channel::<String>();
    loop {
        if let Ok((mut socket, addr)) = server.accept() {
            println!("Client {} connected", addr);

            let tx = tx.clone();
            clients.push(socket.try_clone().expect("failed to clone client"));
            unsafe {
                println!("Added token {} at offset {}.", get_token(addr.to_string()).unwrap(), clients.len() - 1);
                CLIENT_LOOKUP[0].insert(get_token(addr.to_string()).unwrap(), clients.len() - 1);
            }

            thread::spawn(move || loop {
                let mut buff = vec![0; MSG_SIZE];

                match socket.read_exact(&mut buff) {
                    Ok(_) => {
                        let msg = buff.into_iter().take_while(|&x| x != 0).collect::<Vec<_>>();
                        let msg = String::from_utf8(msg).expect("Invalid utf8 message");

                        if msg.contains(":c") {
                            let mut cmd = msg.split(' ');
                            let _ = cmd.next();
                            let foreign_token = cmd.next().unwrap();
                            let local_token = get_token(addr.to_string()).unwrap();
                            
                            unsafe {
                                CHAT_ROOMS[0].insert(local_token.to_string(), foreign_token.to_string());
                                CHAT_ROOMS[0].insert(foreign_token.to_string(), local_token.to_string());
                                println!("{} connected to {}", local_token.to_string(), foreign_token.to_string());
                            }
                        }
                        unsafe {
                            /*
                            if CHAT_ROOMS[0].contains_key(&get_token(addr.to_string()).unwrap()) {
                                let f_addr = CHAT_ROOMS[0].get(&get_token(addr.to_string()).unwrap()).unwrap().to_string();
                                clients[CLIENT_LOOKUP[0][&get_token(f_addr).unwrap()]].write_all(&msg.clone().into_bytes());
                            }
                            */
                            MESSAGE_QUEUE.push(Mesasge {
                                source: get_token(addr.to_string()).unwrap(),
                                msg: msg.clone(),
                            })
                        }
                        println!("{}: {:?}", addr, msg);
                        tx.send(msg).expect("failed to send msg to rx");
                    }, 
                    Err(ref err) if err.kind() == ErrorKind::WouldBlock => (),
                    Err(_) => {
                        println!("closing connection with: {}", addr);
                        break;
                    }
                }

                sleep();
            });
        }

        
        if let Ok(msg) = rx.try_recv() {
            /*
            clients = clients.into_iter().filter_map(|mut client| {
                let mut buff = msg.clone().into_bytes();
                buff.resize(MSG_SIZE, 0);

                client.write_all(&buff).map(|_| client).ok()
            }).collect::<Vec<_>>();
            */

            unsafe {
                for item in &MESSAGE_QUEUE {
                    let mut buff = msg.clone().into_bytes();
                    buff.resize(MSG_SIZE, 0);
                    let f_addr = CHAT_ROOMS[0].get(&get_token(item.source.to_string()).unwrap()).unwrap().to_string();
                    println!("{} - {:?} -> {}", item.source, item.msg, f_addr);
                    clients[CLIENT_LOOKUP[0][&f_addr]].write(&buff);
                    clients[CLIENT_LOOKUP[0][&f_addr]].flush();
                }
            }
        }
        

        sleep();
    }
}
