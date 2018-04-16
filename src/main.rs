#![feature(ip_constructors)]

use std::net::{TcpListener, TcpStream, Ipv4Addr};
use std::io::{BufRead, BufReader, Write, BufWriter};
use std::sync::mpsc;

extern crate crossbeam;
extern crate ascii_utils;

use ascii_utils::table as ascii;

mod operation; // Look for operation.rs

use operation::Operation;

fn main()
{
	println!("Starting server");

    let control_soc = TcpListener::bind((Ipv4Addr::unspecified(), 1313)).expect("Failed to bind to address");

    for client_stream in control_soc.incoming()
    {
        match client_stream
        {
            Ok(client) => // type: TcpListener
            {
                println!("New client peer: {:?}", client.peer_addr());
                crossbeam::scope(|scope| // Threads spawned in this scope will be destroyed at the end of said scope
                {
                    // Let client_recv send information to client_send
                    let (tx, rx): (mpsc::Sender<Operation>, mpsc::Receiver<Operation>) = mpsc::channel();

                    let recv = scope.spawn(||
                    {
                        client_recv(&client, tx);
                    });

                    let send = scope.spawn(||
                    {
                        client_send(&client, rx); // effectively a slave to client_recv thread
                    });

                    recv.join();
                    // recv sent Exit operation, so send should be joining soon™
                    send.join();
                });
            }
            Err(_) =>
            {
                println!("We got nothing?");
            }
        }
    }
}

fn client_recv(client: &TcpStream, tx: mpsc::Sender<Operation>)
{
    println!("Thread for receiving data from client: {:?}", client.peer_addr());
    let reader = BufReader::new(client);
    for bytes in reader.split(b'\0') // Delimited by null
    {
        match bytes
        {
            Ok(bytes) =>
            {
                if bytes.len() == 0
                {
                    println!("Received empty transmission");
                    continue;
                }
                let op = Operation::from(bytes[0], bytes.get(1).cloned()); // Cloned because Rust is picky about u8 vs &u8
                if let Err(err) = op { println!("{:?}", err); continue; } // FIXME: this is bleh
                let op = op.expect("This shouldn't be possible");
                println!("Got op {:?}", op);
                if let Operation::Exit = op { tx.send(op).expect("Failed to send operation to slave thread"); return; }
                tx.send(op).expect("Failed to send operation to slave thread");

                println!("<\t{}", String::from_utf8(bytes).expect("Invalid UTF-8"));
            },
            Err(err) => println!("Error splitting string: {:?}", err)
        }
    }
}

fn client_send(client: &TcpStream, rx: mpsc::Receiver<Operation>)
{
    println!("Thread for sending data to client: {:?}", client.peer_addr());
    let mut writer = BufWriter::new(client);
    loop
    {
        let op = rx.try_recv();
        match op
        {
            Ok(op) =>
            {
                match op
                {
                    Operation::Exit => { return; },
                    Operation::List =>
                    {
                        let cur_dir = std::env::current_dir().expect("Unable to get current directory");
                        let entries = std::fs::read_dir(cur_dir).expect("Error iterating over directory entires");
                        let mut result = String::new();
                        for item in entries
                        {
                            match item
                            {
                                Ok(item) =>
                                {
                                    result.push_str(&item.file_name().into_string().unwrap());
                                    //result.push('\x1F'); // 0x1F: US, Unit Separator
                                    result.push(ascii::US.into());
                                }
                                Err(err) => println!("Error iterating over directory entries: {}", err)
                            }
                        }
                        let res = write!(&mut writer, "file_listing_here\x00{}\n", result);
                        if let Err(_) = res { return; }
                        let res = writer.flush();
                        if let Err(_) = res { return; }
                    },
                    Operation::Get(selection) =>
                    {
                        let res = write!(&mut writer, "get_{}\x00\n", selection);
                        if let Err(_) = res { return; }
                        let res = writer.flush();
                        if let Err(_) = res { return; }
                    }
                }
            },
            Err(_) => ()
        }
    }
}
