#![feature(ip_constructors)]

use std::net::{TcpListener, Ipv4Addr};
use std::io;
use std::io::{BufRead, BufReader, Write, BufWriter};
use std::thread;

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
                let mut reader = BufReader::new(&client);
                let mut writer = BufWriter::new(&client);
                crossbeam::scope(|scope| // Threads spawned in this scope will be destroyed at the end of said scope
                {
                    scope.spawn(||
                    {
                        println!("Thread for receiving data from client: {:?}", client.peer_addr());
                        for line in reader.lines()
                        {
                            match line
                            {
                                Ok(line) =>
                                {
                                    println!("\t{}", line);
                                }
                                Err(err) => println!("No data: {:?}", err)
                            }
                        }
                        println!("We're out of lines from {:?}", client.peer_addr());
                    });
                    scope.spawn(||
                    {
                        println!("Thread for sending data to client {:?}", client.peer_addr());
                        let stdin = BufReader::new(io::stdin());
                        for line in stdin.lines()
                        {
                            match line
                            {
                                Ok(line) =>
                                {
                                    write!(&mut writer, "{}\n", line);
                                    writer.flush().unwrap();
                                }
                                Err(err) => println!("No data from stdin: {:?}", err)
                            }
                        }
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
