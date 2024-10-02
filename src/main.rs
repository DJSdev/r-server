use std::str::FromStr;
use std::sync::Mutex;
use std::net::SocketAddr;
use tokio::net;

#[derive(Default)]
struct Peer {
    session_id: String,
    public_ip_port: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "0.0.0.0:58008";

    println!("Listening on {}", addr);

    let socket = net::UdpSocket::bind(&addr)
        .await
        .expect("Couldnt bind socket");

    let storage: Mutex<Vec<Peer>> = Mutex::new(Vec::new());

    loop {
        let mut buf = [0; 4];

        println!("Waiting to recv...");

        let (size, socket_addr) = socket.recv_from(&mut buf).await.expect("Unable to receive");
        println!("Socket recv_from {} {}", size, socket_addr.to_string());

        if size <= 4 {
            println!("Buffer: {:?}", &buf.to_ascii_lowercase());

            let buf_string =
                String::from_utf8(buf.to_vec()).expect("Unable to make string from utf buffer");

            let guard = storage.lock().expect("Unable to get mutex lock");

            if let Some(peer) = guard.iter().find(|peer| peer.session_id == buf_string) {
                println!("Found peer address in memory {} with session {}", peer.public_ip_port, peer.session_id);
                let tmp = format!("peer_public_ip_port:{}", peer.public_ip_port);
                let buf = tmp.as_bytes();
                let tmp = format!("peer_public_ip_port:{}", socket_addr.to_string());
                let buf2 = tmp.as_bytes();

                let bytes_written = socket
                    .send_to(buf, socket_addr)
                    .await
                    .expect("Not able to send data on socket");

                println!("Bytes written one {}", bytes_written);

                let bytes_written = socket
                    .send_to(buf2, SocketAddr::from_str(peer.public_ip_port.as_str()).unwrap())
                    .await
                    .expect("Not able to send data on socket");

                println!("Bytes written two {}", bytes_written);

                drop(guard);

            } else {

                drop(guard);
                println!("Storing {} in memory", socket_addr.to_string());
                storage
                    .lock()
                    .expect("Could not get mutex lock")
                    .push(Peer {
                        session_id: buf_string,
                        public_ip_port: socket_addr.to_string(),
                    });
            }
        }
    }
}
