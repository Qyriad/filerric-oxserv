#![feature(ip_constructors)]

use std::net::{TcpListener, TcpStream, Ipv4Addr};
use std::io::{BufRead, BufReader, Write, BufWriter};
use std::sync::mpsc;

extern crate crossbeam;

#[derive(PartialEq, Debug)]
#[repr(u8)]
enum Operation
{
    Exit = 0,
    List = 1,
}

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
                    let (tx, rx): (mpsc::Sender<Operation>, mpsc::Receiver<Operation>) = mpsc::channel();
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

fn client_recv(client: &TcpStream, tx: mpsc::Sender<Operation>)
{
    println!("Thread for receiving data from client: {:?}", client.peer_addr());
    let reader = BufReader::new(client);
    for line in reader.lines()
    {
        match line
        {
            Ok(line) =>
            {
                let ch = line.chars().nth(0).expect("Unable to index");
                println!("Got char {}", ch as u8);
                let op: Operation = unsafe { std::mem::transmute(ch as u8) };
                println!("Got op {:?}", op);
                if let Operation::Exit = op
                {
                    tx.send(op).unwrap();
                    return;
                }
                // Must not be exit

                tx.send(op).unwrap();

                println!("<\t{}", line);
            }
            Err(err) => println!("No data from client {:?}: {:?}", client.peer_addr(), err)
        }
    }
    println!("We're out of lines from client");
    let _ = tx.send(Operation::Exit);
}

fn client_send(client: &TcpStream, rx: mpsc::Receiver<Operation>)
{
    println!("Thread for sending data to client: {:?}", client.peer_addr());
    let mut writer = BufWriter::new(client);
    //let stdin = BufReader::new(io::stdin());
    //for line in stdin.lines()
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
                        let res = write!(&mut writer, "file_listing_here");
                        if let Err(_) = res { return; }
                        let res = writer.flush();
                        if let Err(_) = res { return; }
                    }
                }
            },
            Err(_) => ()
        }

        /*match line
        {
            Ok(line) => // String
            {
                let res = write!(&mut writer, "{}\n", line);
                if let Err(error_value) = res { return; }
                let res = writer.flush();
                if let Err(_) = res { return; }
            }
            Err(err) => { println!("No data from stdin: {:?}", err); return; }
        }*/
        /*if rx.try_recv() == Ok(Operation::Exit)
        {
            return;
        }*/
    }
    //println!("We're out of lines from stdin");
}
