use std::{net::{SocketAddr, UdpSocket}, time::Instant};

use anyhow::{bail, Result};
use clap::Parser;


/// Simple program to greet a person
#[derive(Parser, Debug)]
struct Cli {
    #[clap(long)]
    listen_port: u16,
    #[clap(long)]
    send_port: u16,
    #[clap(long)]
    send_address: String,
    // default is false
    #[clap(long)]
    send_first: Option<bool>,
    #[clap(long)]
    data_length: usize
}

trait SendRecv {
    fn send(&self, data: &[u8]) -> Result<()>;
    fn recv(&self, data: &mut [u8]) -> Result<usize>;
}
impl SendRecv for (UdpSocket,SocketAddr) {
    fn send(&self, data: &[u8]) -> Result<()> {
        let (socket, addr) = self;
        socket.send_to(data, addr)?;
        Ok(())
    }
    fn recv(&self, data: &mut [u8]) -> Result<usize> {
        let (socket, _addr) = self;
        let (size,_sender) = socket.recv_from(data)?;
        Ok(size)
    }
}

fn main() -> Result<()> {
    udp_main()
}

fn udp_main() -> Result<()> {
    let cli = Cli::parse();
    let udpsocket = UdpSocket::bind(format!("0.0.0.0:{port}", port = cli.listen_port))?;

    let send_addr = format!("{address}:{port}", address = cli.send_address, port = cli.send_port);
    // Conver this into an addr that is sendable with udp
    let send_addr : SocketAddr = send_addr.parse()?;
    let mut data = vec![0; cli.data_length];
    send_recv(data.as_mut_slice(),&(udpsocket,send_addr),cli.send_first.unwrap_or(false))
}

fn send_recv(data: &mut [u8], send_recv: &impl SendRecv, send_first: bool) -> Result<()> {
    if  send_first {
        send_recv.send(&data)?;
    }

    let mut last_print = Instant::now();
    let mut last_recv = Instant::now();
    let mut count = 0u32;

    let mut shortest_recv = None;
    let mut longest_recv = None;

    loop {
        // Receive data and send it back ast fast as we can
        let size = send_recv.recv(data)?;
        let this_recv = Instant::now();
        if size != data.len() {
            bail!("Received data of unexpected size: {}", size);
        }
        send_recv.send(data)?;

        // Do housekeeping tasks to update our statistics
        count = count.checked_add(1).ok_or_else(|| anyhow::anyhow!("Overflow"))?;

        // update shortest and longest
        let elapsed = last_recv.elapsed();
        if elapsed <= shortest_recv.unwrap_or(elapsed) {
            shortest_recv = Some(elapsed);
        }
        if elapsed >= longest_recv.unwrap_or(elapsed) {
            longest_recv = Some(elapsed);
        }
        last_recv = this_recv;

        // Print statistics
        let elapsed_duration = last_print.elapsed();
        let elapsed = elapsed_duration.as_secs_f64();
        if elapsed > 1.0 {
            let avg_time = elapsed_duration / count;
            println!("Received {}/sec", f64::from(count) / elapsed);
            println!("Shortest: {:?}, Longest: {:?}", shortest_recv, longest_recv);
            println!("Average: {:?}", avg_time);
            count = 0;
            last_print = Instant::now();
            shortest_recv = None;
            longest_recv = None;
        }
    }

    #[allow(unreachable_code)]
    Ok(())
}
