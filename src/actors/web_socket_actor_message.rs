use gtk_estate::corlib::{AsStr, MovableText};
use tokio::sync::oneshot::Sender;

//use corlib::AsStr;

pub enum WebSocketActorFormat
{

    JsonToCbor,
    Text

}

//pub type ConnectToResult = Result<&'static str, &'static str>;

//static CONNECTION_SUCCEEDED

//static CONNECTION_FAILED_MESSAGE: &str = "Connection Faild";



//A request is sent to the WebSocket actor to connect to a server.

//The WebSocket actor closes its current connection, if it has one, and attempts to connect to the server at the provided address.

//The WebSocket actor sends a unit via the provided sender to the actor-client to indicate that the message has been acknowledged and that a new connection to the server at the provided address is being made.

//The WebSocket actor then communicates its connection ptogress and the connected servers output via its provided output queue.

//#[derive(Debug)]
pub enum WebSocketActorInputMessage
{

    ConnectTo(String) //, Sender<()>) //URL, Has the WebSockect actor started trying to connect to the server?

}

//Messages from the WebSocketActor itself.

//#[derive(Debug)]
pub enum WebSocketActorOutputClientMessage
{

    ConnectionResult(MovableText)

}

impl AsStr for WebSocketActorOutputClientMessage
{

    fn as_str(&self) -> &str
    {

        match self
        {

            WebSocketActorOutputClientMessage::ConnectionResult(message) =>
            {

                message.as_str()

            }

        }

    }

}

//Remote server message

#[derive(Debug)]
pub enum WebSocketActorOutputServerMessage
{



}



//#[derive(Debug)]
pub enum WebSocketActorOutputMessage
{

    ClientMessage(WebSocketActorOutputClientMessage),
    ServerMessage(WebSocketActorOutputServerMessage)

}




