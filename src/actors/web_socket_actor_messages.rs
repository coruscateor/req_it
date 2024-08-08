//use gtk_estate::corlib::{AsStr, SendableText};

use corlib::text::{AsStr, SendableText};

use tokio::sync::oneshot::Sender;

//use corlib::AsStr;

use fastwebsockets::{Frame};

use super::OwnedFrame;

const MAX_HEAD_SIZE: usize = 16;

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

#[derive(Debug)]
pub enum WebSocketActorInputMessage
{

    ConnectTo(String), //, Sender<()>) //URL, Has the WebSockect actor started trying to connect to the server?
    Disconnect,
    WriteFrame(OwnedFrame),
    //SendPing(SendableText),
    //SendPingZero

}

//Messages from the WebSocketActor itself.

#[derive(Debug)]
pub enum WebSocketActorOutputClientMessage
{

    ConnectionSucceed(SendableText),
    ConnectionFailed(SendableText),
    Disconnected(SendableText),
    Disconnecting(SendableText),
    NotConnected(SendableText),
    PingFrameReceived(SendableText),
    PongFrameReceived(SendableText),
    CloseFrameReceived(SendableText)

}

impl AsStr for WebSocketActorOutputClientMessage
{

    fn as_str(&self) -> &str
    {

        match self
        {

            WebSocketActorOutputClientMessage::ConnectionSucceed(message) |
            WebSocketActorOutputClientMessage::ConnectionFailed(message) |
            WebSocketActorOutputClientMessage::Disconnected(message) |
            WebSocketActorOutputClientMessage::NotConnected(message) |
            WebSocketActorOutputClientMessage::Disconnecting(message) |
            WebSocketActorOutputClientMessage::PingFrameReceived(message) |
            WebSocketActorOutputClientMessage::PongFrameReceived(message) |
            WebSocketActorOutputClientMessage::CloseFrameReceived(message) =>
            {

                message.as_str()

            }

        }

    }

}

//Remote server message

/*
#[derive(Debug)]
pub enum WebSocketActorOutputServerMessage
{

    Read(OwnedFrame),
    Error(String)

}
*/


/*
#[derive(Debug)]
pub enum WebSocketActorOutputMessage
{

    ClientMessage(WebSocketActorOutputClientMessage),
    ServerMessage(WebSocketActorOutputServerMessage)

}
*/

#[derive(Debug, Default, Clone, Copy)]
pub enum ProcessingFormat
{

    #[default]
    Text

}

#[derive(Debug)]
pub enum WriteFrameProcessorActorInputMessage
{

    Process(String, ProcessingFormat),
    //PassThroughToWebSocketActor()
    SendPing(SendableText)
    
}

/*
#[derive(Debug)]
pub enum WriteFrameProcessorActorOutputMessage
{

    Frame(OwnedFrame) //Frame),

}
*/

//WriteFrameProcessorActor output

//The frame has been read...

#[derive(Debug)]
pub enum ReadFrameProcessorActorInputMessage
{

    ClientMessage(WebSocketActorOutputClientMessage),
    Frame(OwnedFrame),
    FrameAndMessage(OwnedFrame, WebSocketActorOutputClientMessage)

}

#[derive(Debug)]
pub enum ReadFrameProcessorActorOutputMessage
{

    ClientMessage(WebSocketActorOutputClientMessage),
    Processed(String)

}

//The current WebSocket client state for both actor and GUI states.

/*
#[derive(Debug, Default, Eq, PartialEq)]
pub enum WebSocketConnectionState
{

    #[default]
    NotConnected,
    Connecting,
    Connected,
    Disconnecting

}
*/

