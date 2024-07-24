use act_rs::{impl_default_end_async, impl_default_start_and_end_async, impl_default_start_async, impl_mac_task_actor, impl_mac_task_actor_built_state, ActorFrontend, ActorState};

//impl_mac_runtime_task_actor

use act_rs::tokio::io::mpsc::{ActorIOClient, ActorIOServer, actor_io};

use fastwebsockets::{handshake, FragmentCollector, Frame, OpCode, WebSocket, WebSocketError};

//use gtk_estate::corlib::MovableText;

use corlib::text::MovableText;

//use http_body_util::combinators::Frame;

use gtk_estate::should_continue;
use hyper::body::Incoming;

use hyper::Response;

use tokio::io::{self, AsyncWriteExt};

use tokio::select;

use tokio::sync::mpsc::{channel, Sender};

use tokio::{sync::mpsc::Receiver, runtime::Handle};

use std::cell::RefCell;
use std::future::Future;

use std::sync::atomic::{AtomicI32, AtomicUsize};
use std::{marker::PhantomData, sync::Arc};

use tokio::runtime::Runtime;

use std::collections::HashMap;

//use std::time::{Duration, Instant};

use tokio::time::{Duration, Instant};

//use pretty_goodness::json::PrettyEr;

use tokio::net::TcpStream;

use hyper::{Request, body::Bytes, upgrade::Upgraded, header::{UPGRADE, CONNECTION}};

use http_body_util::Empty;

use anyhow::Result;

use hyper_util::rt::TokioIo;

use tokio::task::JoinHandle;

use crate::actors::{OwnedFrame, ReadFrameProcessorActorInputMessage}; //, WebSocketActorOutputServerMessage};

use super::{ReadFrameProcessorActorOutputMessage, WebSocketActorInputMessage, WebSocketActorOutputClientMessage, WebSocketConnectionState}; //, /WebSocketActorOutputClientMessage, WebSocketActorOutputMessage};

use paste::paste;

use std::sync::atomic::Ordering;

//use super::WebSocketActorStateBuilder;

use tokio::time::timeout_at;

static CONNECTION_SUCCEEDED: &str = "Connection succeeded!";

static ERROR_EMPTY_URL_PROVIDED: &str = "Error: Empty URL provided.";

static SERVER_DISCONNECTED: &str = "Server disconnected";

static ERROR_NO_SERVER_CONNECTED: &str = "Error: No server connected.";

static CLOSE_FRAME_SENT: &str = "Close frame Sent.";

static TIME_ELAPSED_FORCED_CLOSURE_NOTICE: &str = "Close Connection Response Time Has Elapsed: Forcing closure of connection.";

static SERVER_DISCONNECTION_FORCED: &str = "Forced server disconnection";

static PING_RECEIVED: &str = "Ping frame received (pong frame automatically sent).";

static PING_SENT: &str = "Ping frame sent.";

static PONG_RECEIVED: &str = "Pong frame received.";

//static CONNECTION_FAILED: &str = "Connection Faild!";

//#[derive(Debug)]
enum CurrentConnection
{

    WebSocket(WebSocket<TokioIo<Upgraded>>),
    FragmentCollector(FragmentCollector<TokioIo<Upgraded>>)

}

impl CurrentConnection
{

    pub fn is_websocket(&self) -> bool
    {

        if let CurrentConnection::WebSocket(_) = self
        {
            
            return true;

        }

        false

    }

    pub fn is_fragmentcollector(&self) -> bool
    {

        if let CurrentConnection::FragmentCollector(_) = self
        {
            
            return true;

        }

        false

    }

    pub async fn read_frame(&mut self) -> Result<Frame<'_>, WebSocketError>
    {

        match self
        {

            CurrentConnection::WebSocket(ws) =>
            {

                ws.read_frame().await

            },
            CurrentConnection::FragmentCollector(fc) =>
            {

                fc.read_frame().await

            }

        }

    }

    pub async fn write_frame(&mut self, frame: Frame<'_>) -> Result<(), WebSocketError>
    {

        match self
        {

            CurrentConnection::WebSocket(ws) =>
            {

                ws.write_frame(frame).await

            },
            CurrentConnection::FragmentCollector(fc) =>
            {

                fc.write_frame(frame).await

            }

        }

    }

    pub fn into_inner(self) -> TokioIo<Upgraded>
    {

        match self
        {

            CurrentConnection::WebSocket(ws) =>
            {

                ws.into_inner()

            },
            CurrentConnection::FragmentCollector(fc) =>
            {

                fc.into_inner()

            }

        }

    }

    pub async fn shutdown(self) -> Result<(), std::io::Error>
    {

        self.into_inner().shutdown().await

    }

}

enum ConnectedLoopExitReason
{

    ActorIOClientDisconnected,
    ServerDisconnectedOrConnectionError,
    InvalidInput

}

impl ConnectedLoopExitReason
{

    pub fn should_continue(&self) -> bool
    {

        match self
        {
            ConnectedLoopExitReason::ActorIOClientDisconnected => false,
            ConnectedLoopExitReason::ServerDisconnectedOrConnectionError | ConnectedLoopExitReason::InvalidInput => true
        }

    }

}

enum ConnectedLoopNextMove
{

    PrepareForNewConnectionAndConnect(String),
    DisconnectFromServer,
    WriteFrame(OwnedFrame),
    OnWebSocketError(WebSocketError),
    ProcessFrame(OwnedFrame),
    SendPing
    
}

enum ContinueOrConnected
{

    ShouldContinue(bool),
    Connected(CurrentConnection)

}

enum CLEROrConnected
{

    CLER(Option<ConnectedLoopExitReason>),
    Connected(CurrentConnection)

}

struct SpawnExecutor;

impl<Fut> hyper::rt::Executor<Fut> for SpawnExecutor
    where Fut: Future + Send + 'static,
          Fut::Output: Send + 'static
{

    //I presume this will be called (if at all) in a scope after an EnterGuard has been created via a Tokio Runtime or Handle.

    //https://docs.rs/tokio/1.38.0/tokio/runtime/struct.EnterGuard.html  

    fn execute(&self, fut: Fut)
    {

        tokio::task::spawn(fut);
        
    }

}

//WriteFrameProcessorActor -> WebSocketActor -> ReadFrameProcessorActor

//WebSocketActors input queue/channel can be accessed via WriteFrameProcessorActors inter-actor directly, allowing it to be bypassed.

pub struct WebSocketActorState
{

    input_receiver: Receiver<WebSocketActorInputMessage>,
    url: Option<String>,
    read_frame_processor_actor_io: ActorIOClient<ReadFrameProcessorActorInputMessage, ReadFrameProcessorActorOutputMessage>,
    in_the_read_pipeline_count: Arc<AtomicUsize>

}

impl WebSocketActorState
{

    pub fn new(read_frame_processor_actor_io: ActorIOClient<ReadFrameProcessorActorInputMessage, ReadFrameProcessorActorOutputMessage>, in_the_read_pipeline_count: Arc<AtomicUsize>, input_receiver: Receiver<WebSocketActorInputMessage>) -> Self //(Sender<WebSocketActorInputMessage>, Self) //(ActorIOClient<WebSocketActorInputMessage, WebSocketActorOutputMessage>, Self) //read_frame_processor_actor: ReadFrameProcessorActor) -> Self //, read_frame_proccessor_input_sender: Sender<ReadFrameProcessorActorInputMessage>) -> Self
    {

        Self
        {

            input_receiver,
            url: None,
            read_frame_processor_actor_io,
            in_the_read_pipeline_count

        }

    }

    pub fn spawn(read_frame_processor_actor_io: ActorIOClient<ReadFrameProcessorActorInputMessage, ReadFrameProcessorActorOutputMessage>, in_the_read_pipeline_count: &Arc<AtomicUsize>) -> Sender<WebSocketActorInputMessage> //ActorIOClient<WebSocketActorInputMessage, WebSocketActorOutputMessage>
    {

        let (input_sender, input_receiver) = channel(50);

        let state = WebSocketActorState::new(read_frame_processor_actor_io, in_the_read_pipeline_count.clone(), input_receiver);

        WebSocketActor::spawn(state);

        input_sender

    }

    //Default on_enter_async and on_exit_async implementations.

    impl_default_start_and_end_async!();

    //impl_default_start_async!();

    //The non-connected loop

    async fn run_async(&mut self) -> bool
    {

        if let Some(message) = self.input_receiver.recv().await
        {

            let res = self.process_received_actor_input_message(message).await;

            match res
            {

                ContinueOrConnected::ShouldContinue(should_continue) => should_continue,
                ContinueOrConnected::Connected(connection) =>
                {

                    //Connected loop
    
                    if !self.connected_loop(connection).await
                    {
    
                        return false;
    
                    }

                    true
    
                }

            }

        }
        else
        {

            false

        }

    }
    
    async fn connect_to_server(&mut self, url: &String) -> Result<(WebSocket<TokioIo<Upgraded>>, Response<Incoming>)>
    {

        let url_str = url.as_str();

        let connection_stream = TcpStream::connect(url_str).await?;

        let request = Request::builder()
            .method("GET")
            .uri(url_str)
            .header("Host", url_str)
            .header(UPGRADE, "websocket")
            .header(CONNECTION, "upgrade")
            .header("Sec-WebSocket-Verion", "13")
            .body(Empty::<Bytes>::new())?;

        let (ws, res) = handshake::client(&SpawnExecutor, request, connection_stream).await?;

        Ok((ws, res))

    }

    ///
    /// Shuts down and drops the connection. The WebSocket connection closure process should've been comnpleted by this point.
    /// 
    async fn disconnect_from_server(&mut self, mut current_connection: CurrentConnection, received_close_frame: bool) -> Option<ConnectedLoopExitReason>
    {

        async fn shutdown(current_connection: CurrentConnection)
        {

            current_connection.shutdown().await.unwrap();

        }

        //If received_close_frame is true, send the close response frame, otherwise initiate connection closure process.

        if let Err(err) = self.send_close_frame(&mut current_connection).await
        {

            let res = self.on_web_socket_error_report_only(err).await;

            if !Self::continue_or_not(&res)
            {

                shutdown(current_connection).await;

                return res;

            }

        }
        else
        {

            if let Err(_) = self.read_frame_processor_actor_io.input_sender().send(ReadFrameProcessorActorInputMessage::ClientMessage(WebSocketActorOutputClientMessage::Disconnecting(MovableText::Str(CLOSE_FRAME_SENT)))).await
            {

                shutdown(current_connection).await;
    
                return Some(ConnectedLoopExitReason::ActorIOClientDisconnected);
    
            }

        }

        if !received_close_frame
        {

            //Send the disconnection frame

            let now = Instant::now();

            let soon = now.checked_add(Duration::from_secs(10)).expect("Error: Instant problems");

            //Disconnection loop
            
            loop
            {
                
                match timeout_at(soon.clone(), Self::read_frame(&mut current_connection, &self.in_the_read_pipeline_count)).await
                {

                    Ok(res) =>
                    {
                        
                        match res
                        {

                            Ok(frame) =>
                            {

                                match frame.opcode
                                {

                                    OpCode::Close =>
                                    {

                                        //This ain't going down the pipleline.

                                        self.in_the_read_pipeline_count.fetch_sub(1, Ordering::SeqCst);

                                        break;

                                    }
                                    OpCode::Continuation | OpCode::Text | OpCode::Binary =>
                                    {

                                        if let Err(_err) = self.read_frame_processor_actor_io.input_sender().send(ReadFrameProcessorActorInputMessage::Frame(frame)).await
                                        {
                                
                                            return Some(ConnectedLoopExitReason::ActorIOClientDisconnected);
                    
                                        }

                                    }
                                    OpCode::Ping =>
                                    {

                                        let res = self.report_ping_received().await;

                                        if !Self::continue_or_not(&res)
                                        {

                                            shutdown(current_connection).await;

                                            return res;

                                        }

                                    }
                                    OpCode::Pong =>
                                    {

                                        let res = self.report_pong_received().await;

                                        if !Self::continue_or_not(&res)
                                        {

                                            shutdown(current_connection).await;

                                            return res;

                                        }

                                    }
            
                                }

                            }
                            Err(err) =>
                            {

                                let res = self.on_web_socket_error_report_only(err).await;

                                if !Self::continue_or_not(&res)
                                {

                                    shutdown(current_connection).await;
                        
                                    return res;
                        
                                }

                            }

                        }

                    },
                    Err(_err) =>
                    {

                        if let Err(_) = self.read_frame_processor_actor_io.input_sender().send(ReadFrameProcessorActorInputMessage::ClientMessage(WebSocketActorOutputClientMessage::Disconnected(MovableText::Str(TIME_ELAPSED_FORCED_CLOSURE_NOTICE)))).await
                        {

                            shutdown(current_connection).await;
                
                            return Some(ConnectedLoopExitReason::ActorIOClientDisconnected);
                
                        }

                    }

                }

            }
            
        }

        shutdown(current_connection).await;

        //Send error or other message.

        //Make sure to get rid of the URL as well.

        self.url = None;

        if let Err(_) = self.read_frame_processor_actor_io.input_sender().send(ReadFrameProcessorActorInputMessage::ClientMessage(WebSocketActorOutputClientMessage::Disconnected(MovableText::Str(SERVER_DISCONNECTED)))).await
        {

            return Some(ConnectedLoopExitReason::ActorIOClientDisconnected);

        }

        None

    }

    async fn force_disconnection_from_server(&mut self, current_connection: CurrentConnection) -> Option<ConnectedLoopExitReason>
    {

        current_connection.shutdown().await.unwrap();

        self.url = None;

        if let Err(_) = self.read_frame_processor_actor_io.input_sender().send(ReadFrameProcessorActorInputMessage::ClientMessage(WebSocketActorOutputClientMessage::Disconnected(MovableText::Str(SERVER_DISCONNECTION_FORCED)))).await
        {

            return Some(ConnectedLoopExitReason::ActorIOClientDisconnected);

        }

        None

    }

    async fn prepare_for_new_connection_and_connect(&mut self, url: String) -> CLEROrConnected //Option<ConnectedLoopExitReason> //bool
    {

        //mut 

        //Check if a zero length string has been provided for the connection URL.

        if url.is_empty()
        {

            if let Err(_) = self.read_frame_processor_actor_io.input_sender().send(ReadFrameProcessorActorInputMessage::ClientMessage(WebSocketActorOutputClientMessage::ConnectionFailed(MovableText::Str(ERROR_EMPTY_URL_PROVIDED)))).await //self.actor_io_server.output_sender().send(WebSocketActorOutputMessage::ClientMessage(WebSocketActorOutputClientMessage::ConnectionFailed(MovableText::Str(ERROR_EMPTY_URL_PROVIDED)))).await
            {

                return CLEROrConnected::CLER(Some(ConnectedLoopExitReason::ActorIOClientDisconnected));

            }

            return CLEROrConnected::CLER(Some(ConnectedLoopExitReason::InvalidInput));

        }

        match self.connect_to_server(&url).await
        {

            Ok(res) => 
            {

                let connection = CurrentConnection::FragmentCollector(FragmentCollector::new(res.0));

                //Do something with the connection response,

                self.url = Some(url);

                //self.current_state = WebSocketConnectionState::Connected;

                //Connected!

                if let Err(_) = self.read_frame_processor_actor_io.input_sender().send(ReadFrameProcessorActorInputMessage::ClientMessage(WebSocketActorOutputClientMessage::ConnectionSucceed(MovableText::Str(CONNECTION_SUCCEEDED)))).await
                {

                    return CLEROrConnected::CLER(Some(ConnectedLoopExitReason::ActorIOClientDisconnected));

                }

                return CLEROrConnected::Connected(connection);

            },
            Err(err) =>
            {

                let err_string = err.to_string();

                //Send Error message to the actor-client

                if let Err(_) = self.read_frame_processor_actor_io.input_sender().send(ReadFrameProcessorActorInputMessage::ClientMessage(WebSocketActorOutputClientMessage::ConnectionFailed(MovableText::String(err_string)))).await
                {

                    return CLEROrConnected::CLER(Some(ConnectedLoopExitReason::ActorIOClientDisconnected));

                }

                return CLEROrConnected::CLER(Some(ConnectedLoopExitReason::ServerDisconnectedOrConnectionError));

            }

        }

    }

    //Not connected to a server, should continue?

    async fn process_received_actor_input_message(&mut self, message: WebSocketActorInputMessage) -> ContinueOrConnected
    {

        //Not connected

        match message
        {

            WebSocketActorInputMessage::ConnectTo(url) =>
            {

                match self.prepare_for_new_connection_and_connect(url).await
                {

                    CLEROrConnected::CLER(res) =>
                    {

                        return ContinueOrConnected::ShouldContinue(Self::continue_or_not(&res));

                    }
                    CLEROrConnected::Connected(connection) =>
                    {

                        //Connected to a server, return the connection.

                        return ContinueOrConnected::Connected(connection);

                    }
                    
                }

            }
            WebSocketActorInputMessage::Disconnect =>
            {

                return self.report_not_connected().await;

            },
            WebSocketActorInputMessage::WriteFrame(_owned_frame) =>
            {

                return self.report_not_connected().await;

            }
            WebSocketActorInputMessage::SendPing =>
            {

                return self.report_not_connected().await;

            }

        }

    }

    async fn report_not_connected(&mut self) -> ContinueOrConnected
    {

        if let Err(_) = self.read_frame_processor_actor_io.input_sender().send(ReadFrameProcessorActorInputMessage::ClientMessage(WebSocketActorOutputClientMessage::NotConnected(MovableText::Str(ERROR_NO_SERVER_CONNECTED)))).await
        {

            return ContinueOrConnected::ShouldContinue(false);

        }

        ContinueOrConnected::ShouldContinue(true)

    }

    fn continue_or_not(opt_cler: &Option<ConnectedLoopExitReason>) -> bool
    {

        match opt_cler
        {

            Some(res) =>
            {

                return res.should_continue();

            },
            None =>
            {

                return true;
                
            }

        }

    }

    //The loop for when connected to a server, should the actor continue after this?

    async fn connected_loop(&mut self, mut current_connection: CurrentConnection) -> bool
    {

        loop
        {

            let connected_loop_next_move: ConnectedLoopNextMove;

            select! {

                res = self.input_receiver.recv() =>
                {

                    if let Some(message) = res
                    {

                        match message
                        {
                
                            WebSocketActorInputMessage::ConnectTo(url) =>
                            {

                                connected_loop_next_move = ConnectedLoopNextMove::PrepareForNewConnectionAndConnect(url)
                
                            }
                            WebSocketActorInputMessage::Disconnect =>
                            {

                                connected_loop_next_move = ConnectedLoopNextMove::DisconnectFromServer;

                            }
                            WebSocketActorInputMessage::WriteFrame(owned_frame) =>
                            {

                                connected_loop_next_move = ConnectedLoopNextMove::WriteFrame(owned_frame);

                            }
                            WebSocketActorInputMessage::SendPing =>
                            {

                                connected_loop_next_move = ConnectedLoopNextMove::SendPing;

                            }           
                
                        }

                    }
                    else
                    {

                        //ActorIOCleint input receiver has disconnected.

                        return false;

                    }

                },
                res = Self::read_frame(&mut current_connection, &self.in_the_read_pipeline_count) =>
                {

                    //Read a frame out of the current connection?

                    match res
                    {

                        Ok(frame) =>
                        {

                            connected_loop_next_move = ConnectedLoopNextMove::ProcessFrame(frame);

                        },
                        Err(err) =>
                        {

                            connected_loop_next_move = ConnectedLoopNextMove::OnWebSocketError(err);

                        }

                    }    

                }

            }

            match connected_loop_next_move 
            {

                ConnectedLoopNextMove::PrepareForNewConnectionAndConnect(url) =>
                {

                    if !Self::continue_or_not(&self.disconnect_from_server(current_connection, false).await)
                    {

                        return false;

                    }

                    let res = self.prepare_for_new_connection_and_connect(url).await;

                    match res
                    {

                        CLEROrConnected::CLER(should_continue) =>
                        {

                            return Self::continue_or_not(&should_continue);

                        },
                        CLEROrConnected::Connected(connection) =>
                        {

                            current_connection = connection;

                            //The new connection is set, stay in the loop. 

                        }

                    }

                },
                ConnectedLoopNextMove::DisconnectFromServer =>
                {

                    //User initiated disconnection

                    return Self::continue_or_not(&self.disconnect_from_server(current_connection, false).await);

                },
                ConnectedLoopNextMove::WriteFrame(owned_frame) =>
                {

                    if let Err(err) = self.write_frame(&mut current_connection, owned_frame).await
                    {

                        return Self::continue_or_not(&self.on_web_socket_error(err, current_connection).await); //, false).await);

                    }

                },
                ConnectedLoopNextMove::OnWebSocketError(error) =>
                {

                    return Self::continue_or_not(&self.on_web_socket_error(error, current_connection).await); //, false).await);

                },
                ConnectedLoopNextMove::ProcessFrame(frame) =>
                {

                    match frame.opcode
                    {

                        OpCode::Close =>
                        {

                            //Close frame not sent, connection closure initiated by the server.

                            //connected_loop_next_move = ConnectedLoopNextMove::DisconnectFromServer;

                            //This ain't going down the pipleline.

                            self.in_the_read_pipeline_count.fetch_sub(1, Ordering::SeqCst);

                            return Self::continue_or_not(&self.disconnect_from_server(current_connection, true).await);

                        }
                        OpCode::Ping =>
                        {

                           return Self::continue_or_not(&self.report_ping_received().await);

                        }
                        OpCode::Pong =>
                        {

                            return Self::continue_or_not(&self.report_pong_received().await);

                        }
                        OpCode::Continuation | OpCode::Text | OpCode::Binary =>
                        {

                            if let Err(_err) = self.read_frame_processor_actor_io.input_sender().send(ReadFrameProcessorActorInputMessage::Frame(frame)).await
                            {
                    
                                return false;
        
                            }

                        }

                    }
        

                }
                ConnectedLoopNextMove::SendPing =>
                {

                    return Self::continue_or_not(&self.send_ping(&mut current_connection).await);

                }

            }

        }

    }

    //Reads a frame from the proveded connection reference into an OwnedFrame.

    async fn read_frame(current_connection: &mut CurrentConnection, in_the_read_pipeline_count: &AtomicUsize) -> Result<OwnedFrame, WebSocketError>
    {

        let frame = current_connection.read_frame().await?;

        in_the_read_pipeline_count.fetch_add(1, Ordering::SeqCst);

        //Get OwnedFrame from cache...

        let mut of = OwnedFrame::new();

        of.copy_from_read_frame(&frame);

        Ok(of)

    }

    async fn write_frame(&self, current_connection: &mut CurrentConnection, mut of: OwnedFrame) -> Result<(), WebSocketError>
    {

        let frame = of.new_frame_to_be_written();

        current_connection.write_frame(frame).await?;

        //Put OwnedFrame into cache...

        Ok(())

    }

    async fn send_close_frame(&self, current_connection: &mut CurrentConnection) -> Result<(), WebSocketError>
    {

        current_connection.write_frame(Frame::close_raw(vec![].into())).await

    }

    async fn on_web_socket_error(&mut self, error: WebSocketError, current_connection: CurrentConnection) -> Option<ConnectedLoopExitReason> //, received_close_frame: bool) -> Option<ConnectedLoopExitReason>
    {
        let res = self.on_web_socket_error_report_only(error).await;
        
        if !Self::continue_or_not(&res)
        {

            return res;

        }

        //Force?

        self.force_disconnection_from_server(current_connection).await //, received_close_frame).await

    }

    async fn on_web_socket_error_report_only(&mut self, error: WebSocketError) -> Option<ConnectedLoopExitReason>
    {

        if let Err(_) = self.read_frame_processor_actor_io.input_sender().send(ReadFrameProcessorActorInputMessage::ClientMessage(WebSocketActorOutputClientMessage::ConnectionFailed(MovableText::String(error.to_string())))).await
        {

            return Some(ConnectedLoopExitReason::ActorIOClientDisconnected);

        }

        None

    }

    async fn report_ping_received(&mut self) -> Option<ConnectedLoopExitReason>
    {

        if let Err(_) = self.read_frame_processor_actor_io.input_sender().send(ReadFrameProcessorActorInputMessage::ClientMessage(WebSocketActorOutputClientMessage::PingReceived(MovableText::Str(PING_RECEIVED)))).await
        {

            return Some(ConnectedLoopExitReason::ActorIOClientDisconnected);

        }

        None

    }

    async fn send_ping(&mut self, current_connection: &mut CurrentConnection) -> Option<ConnectedLoopExitReason>
    {

        //Send ping, report error or report that the ping has been sent, basically.

        let ping_frame = Frame::new(true, OpCode::Ping, None, vec![].into());

        if let Err(err) = current_connection.write_frame(ping_frame).await
        {

            return self.on_web_socket_error_report_only(err).await;

        }

        if let Err(_) = self.read_frame_processor_actor_io.input_sender().send(ReadFrameProcessorActorInputMessage::ClientMessage(WebSocketActorOutputClientMessage::PingReceived(MovableText::Str(PING_SENT)))).await
        {

            return Some(ConnectedLoopExitReason::ActorIOClientDisconnected);

        }

        None

    }

    async fn report_pong_received(&mut self) -> Option<ConnectedLoopExitReason>
    {

        if let Err(_) = self.read_frame_processor_actor_io.input_sender().send(ReadFrameProcessorActorInputMessage::ClientMessage(WebSocketActorOutputClientMessage::PingReceived(MovableText::Str(PONG_RECEIVED)))).await
        {

            return Some(ConnectedLoopExitReason::ActorIOClientDisconnected);

        }

        None

    }


}

//Setup the macro generated Task actor.

impl_mac_task_actor!(WebSocketActor);
