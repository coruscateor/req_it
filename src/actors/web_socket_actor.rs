use act_rs::{ActorFrontend, ActorState, impl_mac_task_actor, impl_default_start_async, impl_default_end_async, impl_default_start_and_end_async};

//impl_mac_runtime_task_actor

use act_rs::tokio::io::mpsc::{ActorIOClient, ActorIOServer, actor_io};

use fastwebsockets::{handshake, FragmentCollector, Frame, WebSocket, WebSocketError};

//use gtk_estate::corlib::MovableText;

use corlib::text::MovableText;

use hyper::body::Incoming;

use hyper::Response;

use tokio::io::AsyncWriteExt;

use tokio::select;

use tokio::sync::mpsc::Sender;

use tokio::{sync::mpsc::Receiver, runtime::Handle};

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

use super::{ReadFrameProcessorActorOutputMessage, WebSocketActorInputMessage, WebSocketActorOutputClientMessage, WebSocketActorOutputMessage};

use paste::paste;

use std::sync::atomic::Ordering;

static CONNECTION_SUCCEEDED: &str = "Connection succeeded!";

static ERROR_EMPTY_URL_PROVIDED: &str = "Error: Empty URL provided";

static SERVER_DISCONNECTED: &str = "Server disconnected";

static ERROR_NO_SERVER_CONNECTED: &str = "Error: No server connected";

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

    //sender_input: ActorIOInteractorClient<WebSocketActorInputMessage, WebSocketActorOutputMessage>, //Sender<WebSocketActorInputMessage>,
    actor_io_server: ActorIOServer<WebSocketActorInputMessage, WebSocketActorOutputMessage>, //Receiver<WebSocketActorInputMessage>,
    //connection_stream: Option<TcpStream>
    current_connection: Option<CurrentConnection>, //web_socket: Option<WebSocket<TokioIo<Upgraded>>>, //Option<Arc<WebSocket<TokioIo<Upgraded>>>>,
    url: Option<String>,
    //temp_frame: Frame<'_>
    read_frame_processor_actor_io: ActorIOClient<ReadFrameProcessorActorInputMessage, ReadFrameProcessorActorOutputMessage>, //ReadFrameProcessorActor,
    //read_frame_proccessor_input_sender: Sender<ReadFrameProcessorActorInputMessage> //Next stage input sender
    in_the_read_pipeline_count: Arc<AtomicUsize>

}

impl WebSocketActorState
{

    pub fn new(read_frame_processor_actor_io: ActorIOClient<ReadFrameProcessorActorInputMessage, ReadFrameProcessorActorOutputMessage>, in_the_read_pipeline_count: &Arc<AtomicUsize>) -> (ActorIOClient<WebSocketActorInputMessage, WebSocketActorOutputMessage>, Self) //read_frame_processor_actor: ReadFrameProcessorActor) -> Self //, read_frame_proccessor_input_sender: Sender<ReadFrameProcessorActorInputMessage>) -> Self
    {

        //let (sender_input, reciver_input) = channel(50);

        //let (sender_input, reciver_input) = tokio::sync::mpsc::channel(50);

        let (actor_io_client, actor_io_server) = actor_io(10, 1000);

        (actor_io_client,
        Self
        {

            //sender_input,
            actor_io_server,
            //web_socket: None,
            current_connection: None,
            url: None,
            //read_frame_processor_actor, //: ReadFrameProcessorActor::new(state)
            //read_frame_proccessor_input_sender
            read_frame_processor_actor_io,
            in_the_read_pipeline_count: in_the_read_pipeline_count.clone()

        })

    }

    pub fn spawn(read_frame_processor_actor_io: ActorIOClient<ReadFrameProcessorActorInputMessage, ReadFrameProcessorActorOutputMessage>, in_the_read_pipeline_count: &Arc<AtomicUsize>) -> ActorIOClient<WebSocketActorInputMessage, WebSocketActorOutputMessage>
    {

        let (io_client, state) = WebSocketActorState::new(read_frame_processor_actor_io, in_the_read_pipeline_count);

        WebSocketActor::spawn(state);

        io_client

    }

    //Default on_enter_async and on_exit_async implementations.

    //impl_default_on_enter_and_exit_async!();

    impl_default_start_async!();

    //The non-connected loop

    async fn run_async(&mut self) -> bool
    {

        if let Some(message) = self.actor_io_server.input_receiver().recv().await
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

        self.disconnect_from_server().await;

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

    async fn disconnect_from_server(&mut self) -> Option<ConnectedLoopExitReason>
    {

        match self.current_connection.take() //.web_socket.take()
        {

            Some(connection) => //ws) =>
            {

                //Make sure the stream gets shutdown correctly.

                //let _ = ws.into_inner().shutdown().await;

                connection.into_inner().shutdown().await.unwrap();

                //Send error or other message.

                //Make sure to get rid of the URL as well.

                self.url = None;

                if let Err(_) = self.actor_io_server.output_sender().send(WebSocketActorOutputMessage::ClientMessage(WebSocketActorOutputClientMessage::Disconnected(MovableText::Str(SERVER_DISCONNECTED)))).await
                {

                    return Some(ConnectedLoopExitReason::ActorIOClientDisconnected);

                }

            }
            None => {}

        }

        None

    }

    async fn prepare_for_new_connection_and_connect(&mut self, url: String) -> Option<ConnectedLoopExitReason> //bool
    {

        //Check if a zero length string has been provided for the connection URL.

        if url.is_empty()
        {

            if let Err(_) = self.actor_io_server.output_sender().send(WebSocketActorOutputMessage::ClientMessage(WebSocketActorOutputClientMessage::ConnectionFailed(MovableText::Str(ERROR_EMPTY_URL_PROVIDED)))).await
            {

                return Some(ConnectedLoopExitReason::ActorIOClientDisconnected);

            }

            //return false;

            return Some(ConnectedLoopExitReason::InvalidInput);

        }

        //Disconnet from current server.

        let res = self.disconnect_from_server().await;

        if res.is_some()
        {

            return res;

        }

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

                self.current_connection = Some(connection);

                //Do something with the connection response,

                self.url = Some(url);

                //Connected!

                if let Err(_) = self.actor_io_server.output_sender().send(WebSocketActorOutputMessage::ClientMessage(WebSocketActorOutputClientMessage::ConnectionSucceed(MovableText::Str(CONNECTION_SUCCEEDED)))).await
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

                if let Err(_) = self.actor_io_server.output_sender().send(WebSocketActorOutputMessage::ClientMessage(WebSocketActorOutputClientMessage::ConnectionFailed(MovableText::String(err_string)))).await
                {

                    return Some(ConnectedLoopExitReason::ActorIOClientDisconnected);

                    //return false;

                }

                return Some(ConnectedLoopExitReason::ServerDisconnectedOrConnectionError);

            }

        }

        //false

        None

    }

    //Not connected to a server, should continue?

    async fn process_received_actor_input_message(&mut self, message: WebSocketActorInputMessage) -> bool
    {

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

                if let Err(_) = self.actor_io_server.output_sender().send(WebSocketActorOutputMessage::ClientMessage(WebSocketActorOutputClientMessage::NotConnected(MovableText::Str(ERROR_NO_SERVER_CONNECTED)))).await
                {

                    return false;

                }

                true

            },
            WebSocketActorInputMessage::WriteFrame(_owned_frame) =>
            {

                if let Err(_) = self.actor_io_server.output_sender().send(WebSocketActorOutputMessage::ClientMessage(WebSocketActorOutputClientMessage::NotConnected(MovableText::Str(ERROR_NO_SERVER_CONNECTED)))).await
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

    async fn connected_loop(&mut self) -> bool //ConnectedLoopExitReason
    {

        //Connected to a server

        /*
        cannot borrow `*self` as mutable more than once at a time
        second mutable borrow occurs hererustcClick for full compiler diagnostic
        web_socket_actor.rs(461, 18): first mutable borrow occurs here
        web_socket_actor.rs(523, 23): first borrow later used here
        */

        //let ws = self.current_connection.as_mut().unwrap();

        loop
        {

            let ws = self.current_connection.as_mut().unwrap();

            select! {

                res = self.actor_io_server.input_receiver().recv() =>
                {

                    if let Some(message) = res
                    {

                        match message
                        {
                
                            WebSocketActorInputMessage::ConnectTo(url) =>
                            {
                
                                return Self::continue_or_not(self.prepare_for_new_connection_and_connect(url).await);
                
                            }
                            WebSocketActorInputMessage::Disconnect =>
                            {
        
                                return Self::continue_or_not(self.disconnect_from_server().await);
        
                            }
                            WebSocketActorInputMessage::WriteFrame(mut owned_frame) =>
                            {
                
                                //Write frame

                                let frame = owned_frame.new_frame_to_be_written();

                                if let Err(err) = ws.write_frame(frame).await
                                {

                                    return Self::continue_or_not(self.on_web_socket_error(err).await);

                                }

                                //Cache owned_frame...

                            }                
                
                        }

                    }
                    else
                    {

                        //ActorIOCleint input receiver has disconnected.

                        return false;

                    }

                },
                res = ws.read_frame() =>
                {

                    match res
                    {

                        Ok(frame) =>
                        {

                            self.in_the_read_pipeline_count.fetch_add(1, Ordering::SeqCst);

                            //Get from cache...

                            let mut of = OwnedFrame::new();

                            of.copy_from_read_frame(&frame);

                            let _ = self.read_frame_processor_actor_io.input_sender().send(ReadFrameProcessorActorInputMessage::Frame(of)).await; //read_frame_proccessor_input_sender.send(ReadFrameProcessorActorInputMessage::Frame(of)).await.unwrap();

                        },
                        Err(err) =>
                        {

                            //Send Error

                            //Disconnect

                            return Self::continue_or_not(self.on_web_socket_error(err).await);

                        }

                    }    

                }

            }

            /*
            if let Some(message) = self.receiver_input.input_receiver().recv().await
            {
    
                match message
                {
        
                    WebSocketActorInputMessage::ConnectTo(url) =>
                    {
        
                        if !self.prepare_for_new_connection_and_connect(url).await
                        {
        
                            return; // false;
        
                        }
        
                    }
                    WebSocketActorInputMessage::Disconnect =>
                    {

                        self.disconnect_from_server().await;

                        return;

                    }
        
                }
    
            }
            */

        }

    }

    async fn on_web_socket_error(&mut self, error: WebSocketError) -> Option<ConnectedLoopExitReason>
    {

        if let Err(_) = self.actor_io_server.output_sender().send(WebSocketActorOutputMessage::ServerMessage(WebSocketActorOutputServerMessage::Error(error.to_string()))).await
        {

            return Some(ConnectedLoopExitReason::ActorIOClientDisconnected);

        }

        self.disconnect_from_server().await

    }

}

//Setup the macro generated Task actor.

impl_mac_task_actor!(WebSocketActor);


