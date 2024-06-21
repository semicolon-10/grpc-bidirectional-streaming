
pub mod pb {
    tonic::include_proto!("chat");
}

use std::net::ToSocketAddrs;
use tokio::sync::mpsc;
use tokio_stream::{Stream, StreamExt};
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status, Streaming};
use tonic::transport::Server;
use std::pin::Pin;
use pb::ChatMessage;



#[derive(Debug)]
pub struct ChatServer {}

type ResponseStream = Pin<Box<dyn Stream<Item = Result<ChatMessage, Status>>+ Send>>;

type ChatResult<T> = Result<Response<T>, Status>;

#[tonic::async_trait]
impl pb::chat_service_server::ChatService for ChatServer {
    type ChatMessageStreamingStream = ResponseStream;

    async fn chat_message_streaming(&self, request: Request<Streaming<ChatMessage>>) -> ChatResult<Self::ChatMessageStreamingStream> {
        let mut in_stream = request.into_inner();

        let (tx, rx) = mpsc::channel(128);

        tokio::spawn(async move {
            while let Some(result) = in_stream.next().await {
                match result {
                    Ok(item) => {
                        println!("Received {:?} from {:?}", item.message, item.from);
                        tx.send(
                            Ok(ChatMessage {
                                message: format!("{:?} Server", item.message),
                                from: String::from("Server Side")
                            })
                        ).await.unwrap();
                    }
                    Err(status) =>
                        {
                            println!("Error : {}", status);
                            break;
                        }
                }
            }

            println!("Chat Session Ended...!");
        });

        let out = ReceiverStream::new(rx);

        Ok(
            Response::new(Box::pin(out) as Self::ChatMessageStreamingStream)
        )
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let server = ChatServer {};
    println!("Server Started..!");
    Server::builder()
        .add_service(pb::chat_service_server::ChatServiceServer::new(server))
        .serve("[::1]:50051".to_socket_addrs().unwrap().next().unwrap())
        .await
        .unwrap();

    Ok(())
}
