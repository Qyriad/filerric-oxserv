#![feature(ip_constructors)]

use std::net::{TcpListener, Ipv4Addr};

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
            }
            Err(_) =>
            {
                println!("We got nothing?");
            }
        }
    }
}
