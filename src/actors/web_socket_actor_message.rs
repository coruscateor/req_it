use tokio::sync::oneshot::Sender;

pub enum WebSocketActorFormat
{

    JsonToCbor,
    Text

}

//pub type ConnectToResult = Result<&'static str, &'static str>;

//static CONNECTION_SUCCEEDED

//static CONNECTION_FAILED_MESSAGE: &str = "Connection Faild";

pub enum WebSocketActorInputMessage
{

    ConnectTo(String, Sender<()>) //URL, Has the WebSockect actor started trying to connect to the server?

}

pub enum WebSocketActorOutputMessage
{

    ClientMessage,
    ServerMessage

}


