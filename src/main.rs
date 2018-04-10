#![feature(ip_constructors)]

use std::net::{TcpListener, TcpStream, Ipv4Addr};
use std::io::{BufRead, BufReader, Write, BufWriter};
use std::sync::mpsc;

extern crate crossbeam;

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
                    let (tx, rx): (mpsc::Sender<bool>, mpsc::Receiver<bool>) = mpsc::channel();
                    let recv = scope.spawn(||
                    {
                        client_recv(&client, tx);
                    });

                    let send = scope.spawn(||
                    {
                        client_send(&client, rx);
                    });

                    recv.join();
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

fn client_recv(client: &TcpStream, tx: mpsc::Sender<bool>)
{
    println!("Thread for receiving data from client: {:?}", client.peer_addr());
    let reader = BufReader::new(client);
    for line in reader.lines()
    {
        match line
        {
            Ok(line) =>
            {
                println!("<\t{}", line);
            }
            Err(err) => println!("No data from client {:?}: {:?}", client.peer_addr(), err)
        }
    }
    println!("We're out of lines from client");
    tx.send(true).unwrap();
}

fn client_send(client: &TcpStream, rx: mpsc::Receiver<bool>)
{
    println!("Thread for sending data to client: {:?}", client.peer_addr());
    let mut writer = BufWriter::new(client);
    let stdin = BufReader::new(io::stdin());
    for line in stdin.lines()
    {
        if rx.try_recv() == Ok(true)
        {
            return;
        }
        match line
        {
            Ok(line) => // String
            {
                let res = write!(&mut writer, "{}\n", line);
                match res
                {
                    Err(err) => 
                    {
                        return;
                    },
                    _ => ()
                }
                let res = writer.flush();
                match res
                {
                    Err(err) => 
                    {
                        eprintln!("!Exiting send, err {}", err);
                        return;
                    },
                    _ => eprintln!("!Success flushing")
                }
            }
            Err(err) => println!("No data from stdin: {:?}", err)
        }
        if rx.try_recv() == Ok(true)
        {
            return;
        }
    }
    println!("We're out of lines from stdin");
}
