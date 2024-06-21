pub mod pb {
    tonic::include_proto!("chat");
}

use tokio::io;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::mpsc;
use tokio_stream::StreamExt;
use tokio_stream::wrappers::ReceiverStream;
use tonic::Request;
use tonic::transport::Channel;
use pb::{chat_service_client::ChatServiceClient,ChatMessage};


pub async fn input() -> String {
    println!("Type Something...!");

    let mut inp = String::new();
    let stdin = io::stdin();
    let mut reader = BufReader::new(stdin);
    reader.read_line(& mut inp).await.expect("Failed to read line");
    inp.trim().to_string()
}

async fn chat(client: & mut ChatServiceClient<Channel>) {
    let (tx, rx) = mpsc::channel(128);

    let in_stream = ReceiverStream::new(rx);

    tokio::spawn(async move {
        loop {
            let user_msg = input().await;
            if user_msg.eq_ignore_ascii_case("exit") {
                break;
            } else {
                let msg = ChatMessage {
                    message: user_msg,
                    from: String::from("Client"),
                };

                if tx.send(msg).await.is_err() {
                    break;
                }
            }
        }
    });

    let response = client
        .chat_message_streaming(Request::new(in_stream))
        .await
        .unwrap();

    let mut resp_stream = response.into_inner();

    while let Some(receive) = resp_stream.next().await {
        let item = receive.unwrap();
        println!("Received {:?} from {:?}", item.message, item.from);
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = ChatServiceClient::connect(
        "http://[::1]:50051"
    )
        .await
        .unwrap();
    println!("Client Started");

    chat(&mut client).await;

    Ok(())
}
