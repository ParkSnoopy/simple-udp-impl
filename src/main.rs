#![no_std]

use core::{ net, fmt, env };

use log::*;
use heapless::*;
use anyhow::{ Result, anyhow };

const BUFFER_INIT_VALUE_U8: u8 = 0;
const BUFFER_INIT_VALUE_CHAR: char = '\0';


fn main() -> Result<()> {
    const FALLBACK_ARGV: &str = "__RANDOM_STRING_THAT_NEVER_MATCH_TO_MODE__";

    let mut args = env::args().skip(1);
    let mode: &str = &args.next().unwrap_or(FALLBACK_ARGV.to_string());
    let addr: &str = &args.next().unwrap_or(FALLBACK_ARGV.to_string());

    match mode {
        "server" => {
            loop_server(addr)?;
        },
        "client" => {
            loop_client(addr)?;
        },
        _ => {
            help();
        }
    }

    Ok(())
}

fn help() {
    info!("");
    info!("[  INIT  ] Failed to parse command line arguments.");
    info!("");
    info!("  myftp-rs <mode> <address>");
    info!("");
    info!("    mode    : execution mode (server, client)");
    info!("    address : socket address to bind (e.g. '127.0.0.1:8000')");
    info!("");
}

fn bind_server<I>(address: I) -> Result<net::UdpSocket>
where I: net::ToSocketAddrs + fmt::Debug
{
    let socket = net::UdpSocket::bind(&address)?;
    info!("[ SERVER ] Server established on `{:?}`", &address);

    Ok(socket)
}

#[allow(unreachable_code)]
fn loop_server<I>(address: I) -> Result<()>
where I: net::ToSocketAddrs + fmt::Debug
{
    let server = bind_server(address)?;
    loop {

        // Initialize receive buffer ( size = 2048 bytes )
        let mut buffer: [u8; 2048] = [BUFFER_INIT_VALUE_U8; 2048];

        // Server Receives
        let (rx_bytes, source) = server.recv_from(&mut buffer)?;
        let body = String::from_utf8_lossy(&buffer).trim_end_matches(BUFFER_INIT_VALUE_CHAR).to_owned();
        info!("");
        info!("[ SERVER ] Message received! ({:?} Bytes)", &rx_bytes);
        info!("[ SERVER ]   FROM: {:?}", &source);
        info!("[ SERVER ]   BODY:");
        info!("{}", &body);

        // Server Answers
        let answer = make_server_answer(&buffer, &source, &rx_bytes);
        let tx_bytes = server.send_to(&answer, &source)?;
        info!("[ SERVER ] Answer sent! ({:?} Bytes)", &tx_bytes);
    }

    Ok(())
}

// temporal hard-coded function
fn make_server_answer(_buffer: &[u8], source: &net::SocketAddr, bytes: &usize) -> Vec<u8>
{
    format!("
        /// THIS IS ANSWER FROM SERVER ///

          - Request origin: {source}
          - Bytes received: {bytes}

        /// END OF ANSWER FROM SERVER  ///
    ").into()
}

struct Client {
    client: net::UdpSocket,
    server: net::SocketAddr,
}

impl Client {
    fn connect<I>(server_addr: I) -> Result<Self>
    where I: net::ToSocketAddrs + fmt::Debug
    {
        const DEFAULT_BIND_SELF: &str = "0.0.0.0:0";

        let client = net::UdpSocket::bind(DEFAULT_BIND_SELF)?;
        info!("[ CLIENT ] Client established on `{:?}`", &DEFAULT_BIND_SELF);

        let server = {
            let parsed: Vec<net::SocketAddr> = server_addr
                .to_socket_addrs()?
                .collect();

            if parsed.len() < 1 {
                return Err(anyhow!("Invalid socket address input"));
            } else if parsed.len() > 1 {
                return Err(anyhow!("This program only support single socket per client"));
            }

            // SAFE: Length of `parsed` is verified to be `1`
            parsed[0]
        };

        Ok(Self {
            client: client,
            server: server,
        })
    }

    fn send(&self, msg: &[u8]) -> Result<usize> {
        self.client.send_to(msg, self.server)
    }

    fn recv(&self, buffer: &mut [u8]) -> Result<usize> {
        self.client.recv(buffer)
    }
}

#[allow(unreachable_code)]
fn loop_client<I>(address: I) -> Result<()>
where I: net::ToSocketAddrs + fmt::Debug
{
    let client = Client::connect(address)?;
    let mut in_buf: String  = String::new();
    let stdin = io::stdin();
    loop {

        // Initialize receive buffer ( size = 2048 bytes )
        in_buf.clear();
        let mut rx_buf: [u8; 2048] = [BUFFER_INIT_VALUE_U8; 2048];

        // Read from STDIN
        stdin.read_line(&mut in_buf)?;
        let request = in_buf.as_bytes();

        // Client Requests
        let tx_bytes = client.send(&request)?;
        info!("[ CLIENT ] Request sent! ({:?} Bytes)", &tx_bytes);

        // Client Receives
        let rx_bytes = client.recv(&mut rx_buf)?;
        let rx_body = String::from_utf8_lossy(&rx_buf).trim_end_matches(BUFFER_INIT_VALUE_CHAR).to_owned();
        info!("");
        info!("[ CLIENT ] Message received! ({:?} Bytes)", &rx_bytes);
        info!("[ CLIENT ]   BODY:");
        info!("{}", &rx_body);
    }

    Ok(())
}
