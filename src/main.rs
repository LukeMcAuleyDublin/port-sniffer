use bpaf::Bpaf;
use std::io::{ self, Write };
use std::net::{ IpAddr, Ipv4Addr };
use std::sync::mpsc::{ channel, Sender };
use tokio::net::TcpStream;
use tokio::task;

// Highest IP port
const MAX: u16 = 65535;

// Address fallback: localhost
const IP_FALLBACK: IpAddr = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));

#[tokio::main]
async fn main() {
    // collect args
    let opts = arguments().run();
    // Init channel
    let (tx, rx) = channel();
    for i in opts.start_port..opts.end_port {
        let tx = tx.clone();

        task::spawn(async move { scan(tx, i, opts.address).await });
    }
    // Create vector out of all of the outputs
    let mut out = vec![];
    // drop tx clones
    drop(tx);
    // wait for all outputs to finish and push them into vector

    for p in rx {
        out.push(p);
    }
    println!("");
    out.sort();
    for v in out {
        println!("{} is open", v);
    }
}

// CLI Arguments
#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
pub struct Arguments {
    // -a or --address
    #[bpaf(long, short, argument("Address"), fallback(IP_FALLBACK))]
    pub address: IpAddr,
    #[bpaf(
        long("start"),
        short('s'),
        guard(start_port_guard, "Must be greater than 0"),
        fallback(1u16)
    )]
    pub start_port: u16,
    #[bpaf(
        long("end"),
        short('e'),
        guard(end_port_guard, "Must be less than or equal to 65535"),
        fallback(MAX)
    )]
    pub end_port: u16,
}

fn start_port_guard(input: &u16) -> bool {
    *input > 0
}

fn end_port_guard(input: &u16) -> bool {
    *input <= MAX
}

async fn scan(tx: Sender<u16>, start_port: u16, addr: IpAddr) {
    match TcpStream::connect(format!("{}:{}", addr, start_port)).await {
        Ok(_) => {
            print!(".");
            io::stdout().flush().unwrap();
            tx.send(start_port).unwrap();
        }
        Err(_) => {}
    }
}