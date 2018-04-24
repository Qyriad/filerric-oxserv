#![feature(ip_constructors)]

use std::net::{TcpListener, TcpStream, IpAddr, Ipv4Addr, SocketAddr, UdpSocket};
use std::io::{BufRead, BufReader, Read, Write, BufWriter};
use std::sync::mpsc;
use std::fs::{DirEntry, File};
use std::path::PathBuf;
use std::thread;

extern crate crossbeam;
extern crate ascii_utils;

use ascii_utils::table as ascii;

mod operation; // Look for operation.rs

use operation::Operation;

macro_rules! crash
{
	($e:expr) => { format!("{}; {}:{}", $e, line!(), column!()).as_str() };
	($fmt:expr, $($arg:tt)+) => { format!(concat!($fmt, "; {}:{}"), $($arg)+, line!(), column!()).as_str() }
}

fn main()
{
	println!("Starting server");

    let control_soc = TcpListener::bind((Ipv4Addr::unspecified(), 1313)).expect(crash!("Failed to bind to address"));

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
                let op = op.unwrap_or_else(|_| unreachable!());
                println!("Got op {:?}", op);
                if let Operation::Exit = op { tx.send(op).expect(crash!("Failed to send operation to slave thread")); return; }
                tx.send(op).expect(crash!("Failed to send operation to slave thread"));

                println!("<\t{}", String::from_utf8(bytes).expect(crash!("Invalid UTF-8")));
            },
            Err(err) => { println!("Error splitting string: {:?}", err); return; }
        }
    }
	tx.send(Operation::Exit).expect(crash!("Failed to send exit operation to slave thread"));
}

fn client_send(client: &TcpStream, rx: mpsc::Receiver<Operation>)
{
    println!("Thread for sending data to client: {:?}", client.peer_addr());
    let mut writer = BufWriter::new(client);
	let cur_dir = std::env::current_dir().expect(crash!("Unable to get current directory"));

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
                        let mut entries = std::fs::read_dir(&cur_dir)
							.expect(crash!("Error iterating over directory entries"))
							.filter(|entry| //: &Result<DirEntry, io::Error>
									{
										match entry
										{
											&Ok(ref entry) => match entry.file_type()
											{
												Ok(ft) => ft.is_file(),
												Err(err) =>
												{
													println!("IO error: {}", err);
													false
												}
											}
											&Err(ref err) =>
											{
												println!("IO error: {}", err);
												false
											}
										}
									});
                        let mut result = String::new();
                        for item in entries
                        {
                            match item
                            {
                                Ok(item) =>
                                {
                                    result.push_str(&item.file_name().into_string().unwrap());
                                    result.push(ascii::US.into()); // 0x1F, Unit Separator
                                }
                                Err(err) => println!("IO Error: {}", err)
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

						let mut entries = std::fs::read_dir(&cur_dir).expect(crash!("Error iterating over directory entries"))
							.filter(|entry| //: &Result<DirEntry, io::Error>
									{
										match entry
										{
											&Ok(ref entry) => match entry.file_type()
											{
												Ok(ft) => ft.is_file(),
												Err(err) =>
												{
													println!("IO error: {}", err);
													false
												}
											}
											&Err(ref err) =>
											{
												println!("IO error: {}", err);
												false
											}
										}
									});
						// entries.nth(): Option<io::Result<DirEntry>>
						let item: DirEntry = entries.nth(selection as usize)
							.expect(crash!("Requested non-existent file"))
							.expect(crash!("IO Error"));

						let (udp_tx, udp_rx): (mpsc::Sender<SocketAddr>, mpsc::Receiver<SocketAddr>) = mpsc::channel();
						let (exit_tx, exit_rx): (mpsc::Sender<()>, mpsc::Receiver<()>) = mpsc::channel();

						let loc_addr = client.local_addr().expect(crash!("Unable to get TCP local address")).ip();
						let path = item.path();
						thread::spawn(move || udp_server(udp_tx, exit_rx, loc_addr, path));

						let udp_addr = udp_rx.recv().expect(crash!("Unable to get UDP server's address"));
						println!("UDP Server is listening on port {}", udp_addr.port());

						let res = write!(&mut writer, "file_{}port_{}\x00", item.file_name().into_string().unwrap(), udp_addr.port());
						if let Err(_) = res { exit_tx.send(()).unwrap(); return; }
						let res = writer.flush();
						if let Err(_) = res { exit_tx.send(()).unwrap(); return; }
                    }
                }
            },
            Err(_) => ()
        }
    }
}

fn udp_server(tx: mpsc::Sender<SocketAddr>, exit_rx: mpsc::Receiver<()>, bind_addr: IpAddr, path: PathBuf)
{
	let soc = UdpSocket::bind((bind_addr, 0)).expect(crash!("Failed to bind UDP server"));
	let addr = soc.local_addr().expect(crash!("Failed to get UDP address"));
	println!("Bound UDP server to {:?}", addr);

	tx.send(addr).expect(crash!("Failed to send UDP address"));

	// 1 megabit
	let mut buffer: Vec<u8> = Vec::new();
	let (byte_count, addr) = soc.recv_from(&mut buffer).expect(format!("Failed to receive data to UDP server").as_str());
	println!("Received {} bytes from UDP {:?}", byte_count, addr);

	soc.connect(addr).expect(format!("Failed to connect to {:?}", addr).as_str());

	buffer.clear();
	buffer.push(b'A');
	buffer.push(b'C');
	buffer.push(b'K');

	soc.send(&buffer).expect(crash!("Error sending data to client"));

	let mut f = File::open(path).expect(crash!("Error opening file"));
	let mut filebuf = Vec::new();
	f.read_to_end(&mut filebuf).expect(crash!("Error reading file"));

	soc.send(&filebuf).expect(crash!("Error sending file to client"));

	if exit_rx.try_recv().is_ok() { return; }
}
