# filerric-oxserv

## A dead simple Rusty file server!

This is a mostly proof-of-concept simple file server written in Rust. It only accepts one control client at a time.

## Compiling

I didn't do anything fancy, so `cargo run` *should* Just Work™.

Depencencies:
 - crossbeam 0.3.1
 - ascii_utils 0.9.3
 
 ## Usage
 
 Again as this is just a proof of concept, the binding address and port are hard coded to unspecified and 1313 respectively. There are three commands available:

|u8		|	Operation	 |
|-------|----------------|
|0x01	|	Exit		 |
|0x02	|	List		 |
|0x03	|	Get(u8 param)|

Each request takes the form of a u8 which represents the operation to perform, a parameter if specified by that operation, and a null character. 

### Exit

Requests a disconnect.

### List

Send back ASCII `file_listing\0` followed by a list of files, separated by ASCII US (0x1F). 

### Get(u8 param)

Request. to start a UDP server to server the file specified by `param`. As `0x00` is a null character, the file list is one-indexed. 
