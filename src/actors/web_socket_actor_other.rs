use act_rs::{impl_default_end_async, impl_default_start_and_end_async, impl_default_start_async, impl_mac_task_actor, impl_mac_task_actor_built_state, ActorFrontend, ActorState};

//impl_mac_runtime_task_actor

use act_rs::tokio::io::mpsc::{ActorIOClient, ActorIOServer, actor_io};

use fastwebsockets::{handshake, FragmentCollector, Frame, OpCode, WebSocket, WebSocketError};

//use gtk_estate::corlib::MovableText;

use corlib::text::MovableText;

//use http_body_util::combinators::Frame;

use hyper::body::Incoming;

use hyper::Response;

use tokio::io::{self, AsyncWriteExt};

use tokio::select;

use tokio::sync::mpsc::{channel, Sender};

use tokio::{sync::mpsc::Receiver, runtime::Handle};

use std::cell::RefCell;
use std::future::Future;

use std::sync::atomic::AtomicUsize;
use std::{marker::PhantomData, sync::Arc};

use tokio::runtime::Runtime;

//use reqwest::{self, Client, header::{HeaderMap, HeaderValue}};

use std::collections::HashMap;

use std::time::{Duration, Instant};

//use pretty_goodness::json::PrettyEr;

use tokio::net::TcpStream;

use hyper::{Request, body::Bytes, upgrade::Upgraded, header::{UPGRADE, CONNECTION}};

use http_body_util::Empty;

use anyhow::Result;

use hyper_util::rt::TokioIo;

use tokio::task::JoinHandle;

use crate::actors::{OwnedFrame, ReadFrameProcessorActorInputMessage, WebSocketActorOutputServerMessage};

use super::{ReadFrameProcessorActorOutputMessage, WebSocketActorInputMessage, WebSocketActorOutputClientMessage, WebSocketConnectionState}; //, /WebSocketActorOutputClientMessage, WebSocketActorOutputMessage};

use paste::paste;

use std::sync::atomic::Ordering;

use super::WebSocketActorStateBuilder;

static CONNECTION_SUCCEEDED: &str = "Connection succeeded!";

static ERROR_EMPTY_URL_PROVIDED: &str = "Error: Empty URL provided";

static SERVER_DISCONNECTED: &str = "Server disconnected";

static ERROR_NO_SERVER_CONNECTED: &str = "Error: No server connected";

static DISCONNECTION_FRAME_SENT: &str = "Disconnection Frame Sent";

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

/*
struct MutState
{

    pub current_connection: Option<CurrentConnection>

}

impl MutState
{

    pub fn new() -> Self
    {

        Self
        {

            current_connection: None

        }

    }

    pub fn new_rfc() -> RefCell<Self>
    {

        RefCell::new(Self::new())

    }

}
*/

enum ConnectedLoopNextMove //<'f>
{

    PrepareForNewConnectionAndConnect(String),
    DisconnectFromServer,
    WriteFrame(OwnedFrame),
    OnWebSocketError(WebSocketError),
    ProcessFrame(OwnedFrame) //(Frame<'f>)
    
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

//Time to put stuff in Mutexes.

pub struct WebSocketActorState
{

    input_receiver: Receiver<WebSocketActorInputMessage>,
    //sender_input: ActorIOInteractorClient<WebSocketActorInputMessage, WebSocketActorOutputMessage>, //Sender<WebSocketActorInputMessage>,
    //actor_io_server: ActorIOServer<WebSocketActorInputMessage, WebSocketActorOutputMessage>, //Receiver<WebSocketActorInputMessage>,
    //connection_stream: Option<TcpStream>,
    //current_connection: RefCell<Option<CurrentConnection>>, 
    //current_connection: RefCell<Option<CurrentConnection>>, //The reference borrowing with this object has been troublesome... //web_socket: Option<WebSocket<TokioIo<Upgraded>>>, //Option<Arc<WebSocket<TokioIo<Upgraded>>>>,
    url: Option<String>,
    //temp_frame: Frame<'_>
    read_frame_processor_actor_io: ActorIOClient<ReadFrameProcessorActorInputMessage, ReadFrameProcessorActorOutputMessage>, //ReadFrameProcessorActor,
    //read_frame_proccessor_input_sender: Sender<ReadFrameProcessorActorInputMessage> //Next stage input sender
    in_the_read_pipeline_count: Arc<AtomicUsize>,
    current_state: WebSocketConnectionState,
    //mut_state: RefCell<MutState>

}

impl WebSocketActorState
{

    pub fn new(read_frame_processor_actor_io: ActorIOClient<ReadFrameProcessorActorInputMessage, ReadFrameProcessorActorOutputMessage>, in_the_read_pipeline_count: Arc<AtomicUsize>, input_receiver: Receiver<WebSocketActorInputMessage>) -> Self //(Sender<WebSocketActorInputMessage>, Self) //(ActorIOClient<WebSocketActorInputMessage, WebSocketActorOutputMessage>, Self) //read_frame_processor_actor: ReadFrameProcessorActor) -> Self //, read_frame_proccessor_input_sender: Sender<ReadFrameProcessorActorInputMessage>) -> Self
    {

        //let (input_sender, input_receiver) = channel(50);

        //let (sender_input, reciver_input) = tokio::sync::mpsc::channel(50);

        //let (actor_io_client, actor_io_server) = actor_io(10, 1000);

        //(actor_io_client,
        //(input_sender,
        Self
        {

            input_receiver,
            //input_receiver: RefCell::new(input_receiver),
            //actor_io_server,
            //web_socket: None,
            //current_connection: None,
            //current_connection: RefCell::new(None),
            url: None,
            //read_frame_processor_actor, //: ReadFrameProcessorActor::new(state)
            //read_frame_proccessor_input_sender
            read_frame_processor_actor_io,
            in_the_read_pipeline_count, //: in_the_read_pipeline_count.clone(),
            current_state: WebSocketConnectionState::default(),
            //mut_state: MutState::new_rfc()

        }
    //)

    }

    /*
    pub fn spawn(read_frame_processor_actor_io: ActorIOClient<ReadFrameProcessorActorInputMessage, ReadFrameProcessorActorOutputMessage>, in_the_read_pipeline_count: &Arc<AtomicUsize>) -> Sender<WebSocketActorInputMessage> //ActorIOClient<WebSocketActorInputMessage, WebSocketActorOutputMessage>
    {

        //let (io_client, state) = WebSocketActorState::new(read_frame_processor_actor_io, in_the_read_pipeline_count);

        let (sender, state) = WebSocketActorState::new(read_frame_processor_actor_io, in_the_read_pipeline_count);

        WebSocketActor::spawn(state);

        //io_client

        sender

    }
    */

    //Default on_enter_async and on_exit_async implementations.

    //impl_default_on_enter_and_exit_async!();

    impl_default_start_async!();

    //The non-connected loop

    async fn run_async(&mut self) -> bool
    {

        if let Some(message) = self.input_receiver_recv().await //self.input_receiver.recv().await //actor_io_server.input_receiver().recv().await
        {

            if self.process_received_actor_input_message(message).await
            {

                //Connected loop

                if !self.connected_loop().await
                {

                    return false;

                }

            }
            
            return true;

        }

        false

    }

    //Make sure that the server gets disconnected. 

    async fn end_async(&mut self)
    {

        //self.disconnect_from_server().await;

    }

    async fn connect_to_server(&mut self, url: &String) -> Result<(WebSocket<TokioIo<Upgraded>>, Response<Incoming>)>
    {

        let url_str = url.as_str();

        let connection_stream = TcpStream::connect(url_str).await?;

        //mut 

        //self.connection_stream = Some(TcpStream::connect(url_str).await?);

        let request = Request::builder() //Request::new(Empty::<Bytes>::new());
            .method("GET")
            .uri(url_str)
            .header("Host", url_str)
            .header(UPGRADE, "websocket")
            .header(CONNECTION, "upgrade")
            .header("Sec-WebSocket-Verion", "13")
            .body(Empty::<Bytes>::new())?;

        //let connection_stream = self.connection_stream.as_mut().expect("There should be a TcpStream here.");

        //let connection_stream_mut = &mut connection_stream;

        let (ws, res) = handshake::client(&SpawnExecutor, request, connection_stream).await?;

        Ok((ws, res))

    }

    ///
    /// Shuts down and drops the connection. The WebSocket connection closure process should've been comnpleted by this point.
    /// 
    /// Does NOT handle the WebSockets disconnection procedure.
    /// 
    async fn disconnect_from_server(&mut self, current_connection: CurrentConnection) -> Option<ConnectedLoopExitReason>
    {

        current_connection.shutdown().await.unwrap();

        //Send error or other message.

        //Make sure to get rid of the URL as well.

        self.url = None;

        self.current_state = WebSocketConnectionState::NotConnected;

        if let Err(_) = self.read_frame_processor_actor_io.input_sender().send(ReadFrameProcessorActorInputMessage::ClientMessage(WebSocketActorOutputClientMessage::Disconnected(MovableText::Str(SERVER_DISCONNECTED)))).await //self.actor_io_server.output_sender().send(WebSocketActorOutputMessage::ClientMessage(WebSocketActorOutputClientMessage::Disconnected(MovableText::Str(SERVER_DISCONNECTED)))).await
        {

            return Some(ConnectedLoopExitReason::ActorIOClientDisconnected);

        }

        None

        /*
        match current_connection //self.current_connection.take() //.web_socket.take()
        {

            Some(connection) => //ws) =>
            {

                //Make sure the stream gets shutdown correctly.

                //let _ = ws.into_inner().shutdown().await;

                connection.into_inner().shutdown().await.unwrap();

                //Send error or other message.

                //Make sure to get rid of the URL as well.

                self.url = None;

                self.current_state = WebSocketConnectionState::NotConnected;

                if let Err(_) = self.read_frame_processor_actor_io.input_sender().send(ReadFrameProcessorActorInputMessage::ClientMessage(WebSocketActorOutputClientMessage::Disconnected(MovableText::Str(SERVER_DISCONNECTED)))).await //self.actor_io_server.output_sender().send(WebSocketActorOutputMessage::ClientMessage(WebSocketActorOutputClientMessage::Disconnected(MovableText::Str(SERVER_DISCONNECTED)))).await
                {

                    return Some(ConnectedLoopExitReason::ActorIOClientDisconnected);

                }

            }
            None => {}

        }
        */

        //None

    }

    async fn prepare_for_new_connection_and_connect(&mut self, url: String) -> Option<ConnectedLoopExitReason> //bool
    {

        //mut 

        //Check if a zero length string has been provided for the connection URL.

        if url.is_empty()
        {

            if let Err(_) = self.read_frame_processor_actor_io.input_sender().send(ReadFrameProcessorActorInputMessage::ClientMessage(WebSocketActorOutputClientMessage::ConnectionFailed(MovableText::Str(ERROR_EMPTY_URL_PROVIDED)))).await //self.actor_io_server.output_sender().send(WebSocketActorOutputMessage::ClientMessage(WebSocketActorOutputClientMessage::ConnectionFailed(MovableText::Str(ERROR_EMPTY_URL_PROVIDED)))).await
            {

                return Some(ConnectedLoopExitReason::ActorIOClientDisconnected);

            }

            //return false;

            self.current_state = WebSocketConnectionState::NotConnected;

            return Some(ConnectedLoopExitReason::InvalidInput);

        }

        //Disconnet from current server.

        if self.current_state == WebSocketConnectionState::Connected
        {

            //loop
            //{

                let res: Result<OwnedFrame, WebSocketError>;

                {

                    self.current_state = WebSocketConnectionState::Disconnecting;

                    //let ws = self.current_connection.as_mut().unwrap();

                    if let Err(_res) = self.send_close_frame().await
                    {

                        self.current_state = WebSocketConnectionState::NotConnected;

                    }

                    res = self.read_frame().await;

                    /*
                    if let Some(ws) = self.current_connection.as_mut()
                    {

                        if let Err(_res) = ws.write_frame(Frame::close_raw(vec![].into())).await
                        {
    
                            self.current_state = WebSocketConnectionState::NotConnected;
    
                        }
    
                       //if self.current_state == WebSocketConnectionState::Disconnecting
                       //{
    
                        res = self.read_frame().await; //ws.read_frame().await;

                    }
                    else {
                        
                        panic!("etc");

                    }
                    */


                        /*
                        let res = ws.read_frame().await;

                        match res
                        {
            
                            Ok(frame) =>
                            {
            
                                if frame.opcode == OpCode::Close
                                {
            
                                }

                                /*
                                if Self::continue_or_not(self.process_frame(frame).await)
                                {



                                }
                                */
            
                            }
                            Err(_err) =>
                            {

                                self.current_state = WebSocketConnectionState::NotConnected;

                            }

                        }
                        */

                    //}

                }

                /*
                cannot borrow `*self` as mutable more than once at a time
                second mutable borrow occurs hererustcClick for full compiler diagnostic
                web_socket_actor.rs(504, 30): first mutable borrow occurs here
                web_socket_actor.rs(564, 73): first borrow later used here
                 */

                {

                    match res
                    {

                        Ok(frame) =>
                        {

                            if Self::continue_or_not(self.process_frame(frame).await)
                            {



                            }

                        },
                        Err(err) =>
                        {



                        }

                    }

                }

            //}

            let res = self.disconnect_from_server().await;

            if res.is_some()
            {
    
                return res;
    
            }

        }

        self.current_state = WebSocketConnectionState::Connecting;

        match self.connect_to_server(&url).await
        {

            Ok(res) => 
            {

                //self.web_socket = Some(Arc::new(res.0));

                //let clone_me = Arc::new(res.0);

                //let send_me = clone_me.clone();

                /*
                future cannot be sent between threads safely
                the trait `Sync` is not implemented for `(dyn hyper::upgrade::Io + Send + 'static)`, which is required by `{async block@src/actors/web_socket_actor.rs:219:38: 223:26}: Send`rustcClick for full compiler diagnostic
                web_socket_actor.rs(221, 33): captured value is not `Send`
                spawn.rs(163, 21): required by a bound in `tokio::spawn`
                */

                /*
                tokio::spawn(async move {

                    _ = send_me.read_frame().await;

                });
                */

                //self.web_socket = Some(res.0);

                let connection = CurrentConnection::FragmentCollector(FragmentCollector::new(res.0));

                {

                    let mut current_connection_mut = self.current_connection.borrow_mut();

                    *current_connection_mut = Some(connection);

                }

                //Do something with the connection response,

                self.url = Some(url);

                self.current_state = WebSocketConnectionState::Connected;

                //Connected!

                if let Err(_) = self.read_frame_processor_actor_io.input_sender().send(ReadFrameProcessorActorInputMessage::ClientMessage(WebSocketActorOutputClientMessage::ConnectionSucceed(MovableText::Str(CONNECTION_SUCCEEDED)))).await //self.actor_io_server.output_sender().send(WebSocketActorOutputMessage::ClientMessage(WebSocketActorOutputClientMessage::ConnectionSucceed(MovableText::Str(CONNECTION_SUCCEEDED)))).await
                {

                    //return false;

                    return Some(ConnectedLoopExitReason::ActorIOClientDisconnected);

                }

                //return true;

            },
            Err(err) =>
            {

                let err_string = err.to_string();

                //Send Error message to the actor-client

                if let Err(_) = self.read_frame_processor_actor_io.input_sender().send(ReadFrameProcessorActorInputMessage::ClientMessage(WebSocketActorOutputClientMessage::ConnectionFailed(MovableText::String(err_string)))).await //self.actor_io_server.output_sender().send(WebSocketActorOutputMessage::ClientMessage(WebSocketActorOutputClientMessage::ConnectionFailed(MovableText::String(err_string)))).await
                {

                    return Some(ConnectedLoopExitReason::ActorIOClientDisconnected);

                    //return false;

                }

                return Some(ConnectedLoopExitReason::ServerDisconnectedOrConnectionError);

            }

        }

        //false

        //This worked...

        //self.process_frame(Frame::close_raw(vec![].into())).await

        None

    }

    //Not connected to a server, should continue?

    async fn process_received_actor_input_message(&mut self, message: WebSocketActorInputMessage) -> bool
    {

        //let mut_current_connection = self.current_connection.borrow_mut();

        //Not connected

        match message
        {

            WebSocketActorInputMessage::ConnectTo(url) =>
            {

                //return self.prepare_for_new_connection_and_connect(url).await;

                return Self::continue_or_not(self.prepare_for_new_connection_and_connect(url).await);

            }
            WebSocketActorInputMessage::Disconnect =>
            {

                if let Err(_) = self.read_frame_processor_actor_io.input_sender().send(ReadFrameProcessorActorInputMessage::ClientMessage(WebSocketActorOutputClientMessage::NotConnected(MovableText::Str(ERROR_NO_SERVER_CONNECTED)))).await //self.actor_io_server.output_sender().send(WebSocketActorOutputMessage::ClientMessage(WebSocketActorOutputClientMessage::NotConnected(MovableText::Str(ERROR_NO_SERVER_CONNECTED)))).await
                {

                    return false;

                }

                true

            },
            WebSocketActorInputMessage::WriteFrame(_owned_frame) =>
            {

                if let Err(_) = self.read_frame_processor_actor_io.input_sender().send(ReadFrameProcessorActorInputMessage::ClientMessage(WebSocketActorOutputClientMessage::NotConnected(MovableText::Str(ERROR_NO_SERVER_CONNECTED)))).await //self.actor_io_server.output_sender().send(WebSocketActorOutputMessage::ClientMessage(WebSocketActorOutputClientMessage::NotConnected(MovableText::Str(ERROR_NO_SERVER_CONNECTED)))).await
                {

                    return false;

                }

                true

            }

        }

    }

    fn continue_or_not(opt_cler: Option<ConnectedLoopExitReason>) -> bool
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

    async fn connected_loop(&mut self, current_connection: CurrentConnection) -> bool //ConnectedLoopExitReason
    {

        //Connected to a server

        /*
        cannot borrow `*self` as mutable more than once at a time
        second mutable borrow occurs hererustcClick for full compiler diagnostic
        web_socket_actor.rs(461, 18): first mutable borrow occurs here
        web_socket_actor.rs(523, 23): first borrow later used here
        */

        //let ws = self.current_connection.as_mut().unwrap();

        //let mut continue_or_not= true;

        //let mut next_state: WebSocketConnectionState;

        loop
        {

            let connected_loop_next_move: ConnectedLoopNextMove;

            {

                //let ws = self.current_connection.as_mut().unwrap();

                select! {

                    res = self.input_receiver_recv() => //self.input_receiver.recv() => //.actor_io_server.input_receiver().recv() =>
                    {

                        if let Some(message) = res
                        {

                            match message
                            {
                    
                                WebSocketActorInputMessage::ConnectTo(url) =>
                                {
                    
                                    /*
                                    continue_or_not = Self::continue_or_not(self.prepare_for_new_connection_and_connect(url).await);

                                    if !continue_or_not
                                    {

                                        break;

                                    }
                                    */

                                    connected_loop_next_move = ConnectedLoopNextMove::PrepareForNewConnectionAndConnect(url)
                    
                                }
                                WebSocketActorInputMessage::Disconnect =>
                                {
            
                                    //Is Disconnecting...

                                    //return Self::continue_or_not(self.disconnect_from_server().await);
            
                                    //return true; //self.initiate_disconnection(ws).await;

                                    //next_state = WebSocketConnectionState::Disconnecting;

                                    //break;

                                    connected_loop_next_move = ConnectedLoopNextMove::DisconnectFromServer;

                                }
                                WebSocketActorInputMessage::WriteFrame(owned_frame) =>
                                {
                    
                                    //Write frame

                                    /*
                                    let frame = owned_frame.new_frame_to_be_written();

                                    if let Err(err) = ws.write_frame(frame).await
                                    {

                                        return Self::continue_or_not(self.on_web_socket_error(err).await);

                                    }
                                    */

                                    //Cache owned_frame...

                                    connected_loop_next_move = ConnectedLoopNextMove::WriteFrame(owned_frame);

                                }                
                    
                            }

                        }
                        else
                        {

                            //ActorIOCleint input receiver has disconnected.

                            return false;

                        }

                    },
                    res = self.read_frame() => //ws.read_frame() =>
                    {

                        match res
                        {

                            Ok(frame) =>
                            {

                                if frame.opcode == OpCode::Close
                                {

                                    //The close frame should've already been send by the current WebSocket<TokioIo<Upgraded>> instance.

                                    /*
                                    if let Err(_) = self.read_frame_processor_actor_io.input_sender().send(ReadFrameProcessorActorInputMessage::ClientMessage(WebSocketActorOutputClientMessage::Disconnected(MovableText::Str(SERVER_DISCONNECTED)))).await
                                    {

                                        return false;

                                    }
                                    */

                                    //return Self::continue_or_not(self.disconnect_from_server().await); //true;

                                    connected_loop_next_move = ConnectedLoopNextMove::DisconnectFromServer;

                                    //continue;

                                }
                                else
                                {

                                    /* 
                                    self.in_the_read_pipeline_count.fetch_add(1, Ordering::SeqCst);

                                    //Get from cache...

                                    let mut of = OwnedFrame::new();

                                    of.copy_from_read_frame(&frame);

                                    let _ = self.read_frame_processor_actor_io.input_sender().send(ReadFrameProcessorActorInputMessage::Frame(of)).await; //read_frame_proccessor_input_sender.send(ReadFrameProcessorActorInputMessage::Frame(of)).await.unwrap();
                                    */

                                    //drop(ws);

                                    //The borrow checker complained about "process_frame" before connection RefCell intallation.

                                    /*

                                    cannot borrow `*self` as immutable because it is also borrowed as mutable
                                    immutable borrow occurs hererustcClick for full compiler diagnostic
                                    web_socket_actor.rs(712, 22): mutable borrow occurs here
                                    web_socket_actor.rs(827, 74): mutable borrow later used here

                                    */

                                    /*
                                    if !Self::continue_or_not(self.process_frame(frame).await)
                                    {

                                        return false;

                                    }
                                    */

                                    //self.process_frame2();

                                    connected_loop_next_move = ConnectedLoopNextMove::ProcessFrame(frame);

                                }

                            },
                            Err(err) =>
                            {

                                //Send Error

                                //Disconnect

                                //return Self::continue_or_not(self.on_web_socket_error(err).await);

                                connected_loop_next_move = ConnectedLoopNextMove::OnWebSocketError(err);

                            }

                        }    

                    }

                }

                //drop(ws);

            }

            match connected_loop_next_move 
            {

                ConnectedLoopNextMove::PrepareForNewConnectionAndConnect(url) =>
                {

                    if !Self::continue_or_not(self.prepare_for_new_connection_and_connect(url).await)
                    {

                        return false;

                    }

                },
                ConnectedLoopNextMove::DisconnectFromServer =>
                {

                    return Self::continue_or_not(self.disconnect_from_server().await);

                },
                ConnectedLoopNextMove::WriteFrame(mut owned_frame) =>
                {

                    /*
                    {

                        let ws2 = self.current_connection.as_mut().unwrap();

                        let frame = owned_frame.new_frame_to_be_written();
    
                        if let Err(err) = ws2.write_frame(frame).await
                        {
    
                            return Self::continue_or_not(self.on_web_socket_error(err).await);
    
                        }

                    }
                    */

                },
                ConnectedLoopNextMove::OnWebSocketError(error) =>
                {

                    return Self::continue_or_not(self.on_web_socket_error(error).await);

                },
                ConnectedLoopNextMove::ProcessFrame(frame) =>
                {

                    /*
                    cannot borrow `*self` as mutable more than once at a time
                    second mutable borrow occurs hererustcClick for full compiler diagnostic
                    web_socket_actor.rs(730, 26): first mutable borrow occurs here
                    web_socket_actor.rs(954, 66): first borrow later used here
                     */

                    //I don't get this...

                     /*
                    if !Self::continue_or_not(self.process_frame(frame).await)
                    {

                        return false;

                    }
                    */

                    //self.in_the_read_pipeline_count.fetch_add(1, Ordering::SeqCst);

                    //Get from cache...
                    
                    //let mut of = OwnedFrame::new();
            
                    //of.copy_from_read_frame(&frame);
            
                    if let Err(_err) = self.read_frame_processor_actor_io.input_sender().send(ReadFrameProcessorActorInputMessage::Frame(frame)).await
                    {
            
                        //return Some(ConnectedLoopExitReason::ActorIOClientDisconnected);
            
                        return false;

                    }
            

                }

            }

        }

        //continue_or_not

    }

    /*

        error[E0499]: cannot borrow `*self` as mutable more than once at a time
    --> src/actors/web_socket_actor.rs:583:54
        |
    506 |                     if let Some(ws) = self.current_connection.as_mut()
        |                                       ----------------------- first mutable borrow occurs here
    ...
    583 |                             if Self::continue_or_not(self.process_frame(frame).await)
        |                                                      ^^^^               ----- first borrow later used here
        |                                                      |
        |                                                      second mutable borrow occurs here
     */

    //Due to borrowing rule complications, frame reads and writes need to be isolated in their own methods (which is probably the better strategy anyway).

    async fn read_frame(&self) -> Result<OwnedFrame, WebSocketError>
    {

        let mut current_connection_mut = self.current_connection.borrow_mut();

        let frame = current_connection_mut.as_mut().expect("Error: No connection").read_frame().await?;

        self.in_the_read_pipeline_count.fetch_add(1, Ordering::SeqCst);

        //Get OwnedFrame from cache...

        let mut of = OwnedFrame::new();

        of.copy_from_read_frame(&frame);

        Ok(of)

    }

    async fn write_frame(&self, mut of: OwnedFrame) -> Result<OwnedFrame, WebSocketError>
    {

        let frame = of.new_frame_to_be_written();

        //self.current_connection.as_mut().expect("Error: No connection").write_frame(frame).await?;

        self.current_connection.borrow_mut().as_mut().expect("Error: No connection").write_frame(frame).await?;

        //Put OwnedFrame into cache...

        //of...

        Ok(of)

    }

    async fn input_receiver_recv(&self) -> Option<WebSocketActorInputMessage>
    {

        let mut recv_mut = self.input_receiver.borrow_mut(); //.await

        recv_mut.recv().await

    }

    async fn send_close_frame(&self) -> Result<(), WebSocketError>
    {

        let mut current_connection_mut = self.current_connection.borrow_mut();

        let current_connection = current_connection_mut.as_mut().expect("Error: No connection");

        current_connection.write_frame(Frame::close_raw(vec![].into())).await

    }

    /*
    async fn process_frame<'f>(&'f mut self, frame: Frame<'f>) -> Option<ConnectedLoopExitReason> 
    {

        self.in_the_read_pipeline_count.fetch_add(1, Ordering::SeqCst);

        //Get from cache...

        let mut of = OwnedFrame::new();

        of.copy_from_read_frame(&frame);

        if let Err(_err) = self.read_frame_processor_actor_io.input_sender().send(ReadFrameProcessorActorInputMessage::Frame(of)).await
        {

            return Some(ConnectedLoopExitReason::ActorIOClientDisconnected);

        }

        None

    }
    */

    async fn process_frame(&mut self, of: OwnedFrame) -> Option<ConnectedLoopExitReason> 
    {

        if let Err(_err) = self.read_frame_processor_actor_io.input_sender().send(ReadFrameProcessorActorInputMessage::Frame(of)).await
        {

            return Some(ConnectedLoopExitReason::ActorIOClientDisconnected);

        }

        None

    }

    /*
    async fn process_frame2(&mut self) -> Option<ConnectedLoopExitReason> 
    {

        None

    }
    */

    async fn initiate_disconnection(&mut self, current_connection: &mut CurrentConnection) -> bool
    {

        let _ = current_connection.write_frame(Frame::close_raw(vec![].into()));

        if let Err(_) = self.read_frame_processor_actor_io.input_sender().send(ReadFrameProcessorActorInputMessage::ClientMessage(WebSocketActorOutputClientMessage::Disconnecting(MovableText::Str(DISCONNECTION_FRAME_SENT)))).await //self.actor_io_server.output_sender().send(WebSocketActorOutputMessage::ServerMessage(WebSocketActorOutputServerMessage::Error(error.to_string()))).await
        {

            return false;

        }

        true

    }

    async fn on_web_socket_error(&mut self, error: WebSocketError) -> Option<ConnectedLoopExitReason>
    {

        if let Err(_) = self.read_frame_processor_actor_io.input_sender().send(ReadFrameProcessorActorInputMessage::ClientMessage(WebSocketActorOutputClientMessage::ConnectionFailed(MovableText::String(error.to_string())))).await //self.actor_io_server.output_sender().send(WebSocketActorOutputMessage::ServerMessage(WebSocketActorOutputServerMessage::Error(error.to_string()))).await
        {

            return Some(ConnectedLoopExitReason::ActorIOClientDisconnected);

        }

        self.disconnect_from_server().await

    }

}

//Setup the macro generated Task actor.

//impl_mac_task_actor!(WebSocketActor);

/*

future cannot be sent between threads safely
within `{async block@/run/media/paul/Main Stuff/SoftwareProjects/Rust/act_rs/src/tokio/mac_task_actor.rs:122:34: 126:22}`, the trait `Send` is not implemented for `NonNull<tokio::sync::mpsc::Receiver<web_socket_actor_messages::WebSocketActorInputMessage>>`, which is required by `{async block@/run/media/paul/Main Stuff/SoftwareProjects/Rust/act_rs/src/tokio/mac_task_actor.rs:122:34: 126:22}: Send`rustcClick for full compiler diagnostic
mac_task_actor.rs(122, 21): Actual error occurred here
web_socket_actor.rs(1156, 25): future is not `Send` as this value is used across an await
spawn.rs(163, 21): required by a bound in `tokio::spawn`
future cannot be sent between threads safely
within `web_socket_actor::WebSocketActorState`, the trait `Sync` is not implemented for `RefCell<tokio::sync::mpsc::Receiver<web_socket_actor_messages::WebSocketActorInputMessage>>`, which is required by `{async block@/run/media/paul/Main Stuff/SoftwareProjects/Rust/act_rs/src/tokio/mac_task_actor.rs:122:34: 126:22}: Send`
if you want to do aliasing and mutation between multiple threads, use `std::sync::RwLock` insteadrustcClick for full compiler diagnostic
mac_task_actor.rs(122, 21): Actual error occurred here
web_socket_actor.rs(1151, 34): captured value is not `Send` because `&` references cannot be sent unless their referent is `Sync`
spawn.rs(163, 21): required by a bound in `tokio::spawn`
future cannot be sent between threads safely
within `web_socket_actor::WebSocketActorState`, the trait `Sync` is not implemented for `RefCell<std::option::Option<CurrentConnection>>`, which is required by `{async block@/run/media/paul/Main Stuff/SoftwareProjects/Rust/act_rs/src/tokio/mac_task_actor.rs:122:34: 126:22}: Send`
if you want to do aliasing and mutation between multiple threads, use `std::sync::RwLock` insteadrustcClick for full compiler diagnostic
mac_task_actor.rs(122, 21): Actual error occurred here
web_socket_actor.rs(1151, 34): captured value is not `Send` because `&` references cannot be sent unless their referent is `Sync`
spawn.rs(163, 21): required by a bound in `tokio::spawn`
future cannot be sent between threads safely
the trait `Sync` is not implemented for `std::cell::Cell<isize>`, which is required by `{async block@/run/media/paul/Main Stuff/SoftwareProjects/Rust/act_rs/src/tokio/mac_task_actor.rs:122:34: 126:22}: Send`
if you want to do aliasing and mutation between multiple threads, use `std::sync::RwLock` or `std::sync::atomic::AtomicIsize` insteadrustcClick for full compiler diagnostic
mac_task_actor.rs(122, 21): Actual error occurred here
web_socket_actor.rs(1156, 25): future is not `Send` as this value is used across an await
spawn.rs(163, 21): required by a bound in `tokio::spawn`
future cannot be sent between threads safely
within `{async block@/run/media/paul/Main Stuff/SoftwareProjects/Rust/act_rs/src/tokio/mac_task_actor.rs:122:34: 126:22}`, the trait `Send` is not implemented for `NonNull<std::option::Option<CurrentConnection>>`, which is required by `{async block@/run/media/paul/Main Stuff/SoftwareProjects/Rust/act_rs/src/tokio/mac_task_actor.rs:122:34: 126:22}: Send`rustcClick for full compiler diagnostic
mac_task_actor.rs(122, 21): Actual error occurred here
web_socket_actor.rs(1120, 97): future is not `Send` as this value is used across an await
spawn.rs(163, 21): required by a bound in `tokio::spawn`

 */

impl_mac_task_actor_built_state!(WebSocketActor);


