#![feature(ip_constructors)]

use std::net::{TcpListener, TcpStream, Ipv4Addr};
use std::io;
use std::io::{BufRead, BufReader, Write, BufWriter};

extern crate crossbeam;

fn main()
{
	println!("Starting server");

    let control_soc = TcpListener::bind((Ipv4Addr::unspecified(), 1313)).unwrap();

    for client_stream in control_soc.incoming()
    {
        match client_stream
        {
            Ok(client) =>
            {
                println!("New client peer: {:?}", client.peer_addr());
                crossbeam::scope(|scope| // Threads spawned in this scope will be destroyed at the end of said scope
                {
                    scope.spawn(||
                    {
                        client_recv(&client);
                    });
                    scope.spawn(||
                    {
                        client_send(&client);
                    });
                });
            }
            Err(_) =>
            {
                println!("We got nothing?");
            }
        }
    }
}

fn client_recv(client: &TcpStream)
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
    println!("We're out of lines from {:?}", client.peer_addr());
}

fn client_send(client: &TcpStream)
{
    println!("Thread for sending data to client: {:?}", client.peer_addr());
    let mut writer = BufWriter::new(client);
    let stdin = BufReader::new(io::stdin());
    for line in stdin.lines()
    {
        match line
        {
            Ok(line) => // String
            {
                write!(&mut writer, "{}\n", line).unwrap();
                writer.flush().unwrap();
            }
            Err(err) => println!("No data from stdin: {:?}", err)
        }
    }
    println!("We're out of lines from stdin");
}
