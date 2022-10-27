#![allow(dead_code)]

mod client;
mod packets;

fn main() {
    let mut args = std::env::args();

    // Step past the executable name
    args.next();

    let address = args.next().map(|v| v.to_string()).unwrap_or(String::from("normandy"));
    let port = args.next().map(|v| v.parse::<u16>().unwrap_or(6014)).unwrap_or(6014);

    if let Err(e) = runner(&address, port) {
        eprintln!("error: {}", e);
    }
}

fn runner(server_name: &str, port: u16) -> Result<(), String> {
    let remote = format!("{}:{}", server_name, port);

    let socket = std::net::UdpSocket::bind("0.0.0.0:0").map_err(|e| format!("unable to bind udp socket {}", e))?;
    socket.connect(&remote).map_err(|e| format!("unable to connect to remote host {} : {}", &remote, e))?;

    let mut client = client::Client::new(socket);

    display_progress_until_n_files(&mut client, 3)?;

    client.finalize_files()?;
    Ok(())
}

fn display_progress_until_n_files(client: &mut client::Client, file_count: usize) -> Result<(), String> {
    client.send_request()?;
    println!("{}", client);
    let mut last_lines = client.print_line_length();

    while client.file_count() < file_count {
        client.recv_packet()?;
        println!("\x1B[{}A", last_lines + 3);
        for _ in 0..last_lines + 3 {
            println!("                                                                ");
        }
        println!("\x1B[{}A", last_lines + 3);
        println!("{}", client);
        last_lines = client.print_line_length();
    }

    Ok(())
}