use act_rs::{impl_default_end_async, impl_default_start_and_end_async, impl_default_start_async, impl_mac_task_actor, impl_mac_task_actor_built_state, ActorFrontend, ActorState};

//impl_mac_runtime_task_actor

use act_rs::tokio::io::mpsc::{ActorIOClient, ActorIOServer, actor_io};

use fastwebsockets::{handshake, FragmentCollector, FragmentCollectorRead, Frame, OpCode, WebSocket, WebSocketError, WebSocketRead, WebSocketWrite};

//use gtk_estate::corlib::MovableText;

use corlib::text::SendableText;

//use http_body_util::combinators::Frame;

use gtk_estate::should_continue;

use hyper::body::Incoming;

use hyper::Response;

use tokio::io::{self, AsyncWriteExt, ReadHalf, WriteHalf};

use tokio::select;

//use tokio::sync::mpsc::{channel, Sender};

//use tokio::{sync::mpsc::Receiver, runtime::Handle};

use url::Url;

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

use anyhow::{Error, Result};

use hyper_util::rt::TokioIo;

use tokio::task::JoinHandle;

use crate::actors::websockets::fastwebsockets::{OwnedFrame, ReadFrameProcessorActorInputMessage}; //, WebSocketActorOutputServerMessage};

use super::websocket_read_and_write::{WebSocketReadHalf, WebSocketWriteHalf};
use super::{ReadFrameProcessorActorOutputMessage, WebSocketActorInputMessage, WebSocketActorOutputClientMessage}; //, WebSocketConnectionState}; //, /WebSocketActorOutputClientMessage, WebSocketActorOutputMessage};

use paste::paste;

use std::sync::atomic::Ordering;

//use crate::actors::websockets::fastwebsockets::pipeline_message_counter::Incrementor;

//use super::WebSocketActorStateBuilder;

use tokio::time::timeout_at;

use libsync::crossbeam::mpmc::tokio::array_queue::{Sender, Receiver, channel};

use crate::actors::websockets::fastwebsockets::websocket_read_and_write::WebSocketReader;

static CONNECTION_SUCCEEDED: &str = "Connection succeeded!";

static ERROR_EMPTY_URL_PROVIDED: &str = "Error: Empty URL provided.";

static SERVER_DISCONNECTED: &str = "Server disconnected";

static ERROR_NO_SERVER_CONNECTED: &str = "Error: No server connected.";

static CLOSE_FRAME_SENT: &str = "Close frame Sent.";

static TIME_ELAPSED_FORCED_CLOSURE_NOTICE: &str = "Close Connection Response Time Has Elapsed: Forcing closure of connection.";

static SERVER_DISCONNECTION_FORCED: &str = "Forced server disconnection";

static PING_FRAME_RECEIVED: &str = "Ping frame received - pong frame already sent"; //(pong frame automatically sent).";

static PING_FRAME_SENT: &str = "Ping frame sent.";

static PONG_FRAME_RECEIVED: &str = "Pong frame received.";

static CLOSE_FRAME_RECEIVED: &str = "Close frame received - Close frame already sent."; //"Close frame received (Close frame sent automatically)";

//static CONNECTION_FAILED: &str = "Connection Faild!";

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
enum ConnectedLoopNextMove
{

    PrepareForNewConnectionAndConnect(String),
    DisconnectFromServer,
    WriteFrame(OwnedFrame),
    OnWebSocketError(WebSocketError),
    ProcessFrame(OwnedFrame),
    SendPing
    
}
*/

enum ContinueOrConnected
{

    ShouldContinue(bool),
    Connected(WebSocketWrite<TokioIo<Upgraded>>)

}

///
/// CLER: ConnectedLoopExitReason
/// 
enum CLEROrConnected
{

    CLER(Option<ConnectedLoopExitReason>),
    Connected(WebSocketWriteHalf)

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

//Writing WebSocket Actor

pub struct WebSocketActorState
{

    input_receiver: Receiver<WebSocketActorInputMessage>,
    url: Option<Url>, //Option<String>,
    read_frame_processor_actor_io: ActorIOClient<ReadFrameProcessorActorInputMessage, ReadFrameProcessorActorOutputMessage>,
    //in_the_read_pipeline_count: Arc<AtomicUsize>
    pipline_output_count_incrementor: Incrementor

}

impl WebSocketActorState
{

    pub fn new(read_frame_processor_actor_io: ActorIOClient<ReadFrameProcessorActorInputMessage, ReadFrameProcessorActorOutputMessage>, input_receiver: Receiver<WebSocketActorInputMessage>, pipline_output_count_incrementor: Incrementor) -> Self //in_the_read_pipeline_count: Arc<AtomicUsize>, //(Sender<WebSocketActorInputMessage>, Self) //(ActorIOClient<WebSocketActorInputMessage, WebSocketActorOutputMessage>, Self) //read_frame_processor_actor: ReadFrameProcessorActor) -> Self //, read_frame_proccessor_input_sender: Sender<ReadFrameProcessorActorInputMessage>) -> Self
    {

        Self
        {

            input_receiver,
            url: None,
            read_frame_processor_actor_io,
            //in_the_read_pipeline_count
            pipline_output_count_incrementor

        }

    }

    pub fn spawn(read_frame_processor_actor_io: ActorIOClient<ReadFrameProcessorActorInputMessage, ReadFrameProcessorActorOutputMessage>,  pipline_output_count_incrementor: Incrementor) -> Sender<WebSocketActorInputMessage> //in_the_read_pipeline_count: &Arc<AtomicUsize> //ActorIOClient<WebSocketActorInputMessage, WebSocketActorOutputMessage>
    {

        let (input_sender, input_receiver) = channel(50);

        let state = WebSocketActorState::new(read_frame_processor_actor_io, input_receiver, pipline_output_count_incrementor); //in_the_read_pipeline_count.clone(), 

        WebSocketActor::spawn(state);

        input_sender

    }

    //Default on_enter_async and on_exit_async implementations.

    impl_default_start_and_end_async!();

    //impl_default_start_async!();

    //The non-connected loop

    async fn run_async(&mut self) -> bool
    {

        let recv_res = self.input_receiver.recv().await;

        //if let Some(message) = self.input_receiver.recv().await
        if let Ok(message) = recv_res
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
    
    async fn connect_to_server(&mut self, url: &Url) -> Result<(WebSocket<TokioIo<Upgraded>>, Response<Incoming>)> //url: &String) -> Result<(WebSocket<TokioIo<Upgraded>>, Response<Incoming>)>
    {

        //Get from cache...

        let mut host_and_port = String::with_capacity(50);

        //let host;

        match url.host_str()
        {

            Some(the_host) =>
            {

                //host = the_host;

                host_and_port.push_str(the_host);

            }
            None =>
            {

                return Result::Err(Error::msg("Host section not found in the provided URL."));

            }

        }

        match url.port()
        {

            Some(port) =>
            {

                host_and_port.push(':');

                host_and_port.push_str(&port.to_string());

            }
            None =>
            {

                //Assume port 80 if no port number has been provided as part of the URL. 

                host_and_port.push_str(":80");

            }

        }

        //let url_str = url.as_str();

        //let connection_stream = TcpStream::connect(url_str).await?;

        //let connection_stream = TcpStream::connect("localhost:3000").await?; //"0.0.0.0:3000").await?;

        //let connection_stream = TcpStream::connect("0.0.0.0:3000").await?; 

        let connection_stream = TcpStream::connect(&host_and_port).await?; 

        //println!("connection_stream\n");

        //println!("{connection_stream:?}\n");

        let request = Request::builder()
            .method("GET")
            //.uri(url_str)
            //.header("Host", url_str)
            //.uri("http://0.0.0.0:3000")
            //.uri("/")
            //.uri("0.0.0.0:3000") //Invalid status code 404
            //.uri("ws://0.0.0.0:3000")
            .uri(url.as_str())
            //.header("Host", "0.0.0.0:3000")
            .header("Host", host_and_port)
            //.uri("http://localhost:3000")
            //.uri("localhost:3000/")
            //.header("Host", "localhost:3000")
            .header(UPGRADE, "websocket")
            .header(CONNECTION, "upgrade")
            .header("Sec-WebSocket-Key", handshake::generate_key())
            .header("Sec-WebSocket-Version", "13") //"Sec-WebSocket-Verion" OMG!!!!!!
            .body(Empty::<Bytes>::new())?;

            println!("request\n");

            println!("{request:?}");

        //connection_stream.split()

        let (ws, res) = handshake::client(&SpawnExecutor, request, connection_stream).await?;

        /*
        let res = ws.split(|connection| {

            connection.inner().

        });
        */

        //println!("{res:?}");

        Ok((ws, res))

    }

    ///
    /// Shuts down and drops the connection. The WebSocket connection closure process should've been comnpleted by this point.
    /// 
    async fn disconnect_from_server(&mut self, mut web_socket_reader: WebSocketReadHalf, received_close_frame: bool) -> Option<ConnectedLoopExitReason>
    {

        /*
        async fn empty()
        {
        }
        */

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

            if let Err(_) = self.read_frame_processor_actor_io.input_sender().send(ReadFrameProcessorActorInputMessage::ClientMessage(WebSocketActorOutputClientMessage::Disconnecting(SendableText::Str(CLOSE_FRAME_SENT)))).await
            {

                shutdown(current_connection).await;
    
                return Some(ConnectedLoopExitReason::ActorIOClientDisconnected);
    
            }

            self.pipline_output_count_incrementor.inc();

        }

        if !received_close_frame
        {

            //Send the disconnection frame

            let now = Instant::now();

            let soon = now.checked_add(Duration::from_secs(10)).expect("Error: Instant problems");

            //Disconnection loop
            
            loop
            {
                
                match timeout_at(soon, Self::read_frame(&mut current_connection, &self.pipline_output_count_incrementor)).await //.clone() //&self.in_the_read_pipeline_count)).await
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

                                        if let Err(_err) = self.read_frame_processor_actor_io.input_sender().send(ReadFrameProcessorActorInputMessage::FrameAndMessage(frame, WebSocketActorOutputClientMessage::CloseFrameReceived(SendableText::Str(CLOSE_FRAME_RECEIVED)))).await
                                        {

                                            shutdown(current_connection).await;
                                
                                            return Some(ConnectedLoopExitReason::ActorIOClientDisconnected);
                    
                                        }

                                        //self.pipline_output_count_incrementor.inc();

                                        /*
                                        let res = self.report_close_frame_received().await;

                                        if !Self::continue_or_not(&res)
                                        {

                                            shutdown(current_connection).await;

                                            return res;

                                        }
                                        */

                                        ////This ain't going down the pipleline.

                                        //self.in_the_read_pipeline_count.fetch_sub(1, Ordering::SeqCst);

                                        break;

                                    }
                                    OpCode::Continuation | OpCode::Text | OpCode::Binary | OpCode::Pong =>
                                    {

                                        if let Err(_err) = self.read_frame_processor_actor_io.input_sender().send(ReadFrameProcessorActorInputMessage::Frame(frame)).await
                                        {

                                            shutdown(current_connection).await;
                                
                                            return Some(ConnectedLoopExitReason::ActorIOClientDisconnected);
                    
                                        }

                                        //self.pipline_output_count_incrementor.inc();

                                    }
                                    OpCode::Ping =>
                                    {

                                        if let Err(_err) = self.read_frame_processor_actor_io.input_sender().send(ReadFrameProcessorActorInputMessage::FrameAndMessage(frame, WebSocketActorOutputClientMessage::PingFrameReceived(SendableText::Str(PING_FRAME_RECEIVED)))).await
                                        {

                                            shutdown(current_connection).await;
                                
                                            return Some(ConnectedLoopExitReason::ActorIOClientDisconnected);
                    
                                        }

                                        //self.pipline_output_count_incrementor.inc();

                                        /*
                                        let res = self.report_ping_frame_received().await;

                                        if !Self::continue_or_not(&res)
                                        {

                                            shutdown(current_connection).await;

                                            return res;

                                        }
                                        */

                                    }
                                    /*
                                    OpCode::Pong =>
                                    {

                                        let res = self.report_pong_frame_received().await;

                                        if !Self::continue_or_not(&res)
                                        {

                                            shutdown(current_connection).await;

                                            return res;

                                        }

                                    }
                                     */
            
                                }

                                self.pipline_output_count_incrementor.inc();

                                //Give the timeout an opportunity to occur.

                                //empty().await;

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

                        if let Err(_) = self.read_frame_processor_actor_io.input_sender().send(ReadFrameProcessorActorInputMessage::ClientMessage(WebSocketActorOutputClientMessage::Disconnected(SendableText::Str(TIME_ELAPSED_FORCED_CLOSURE_NOTICE)))).await
                        {

                            shutdown(current_connection).await;
                
                            return Some(ConnectedLoopExitReason::ActorIOClientDisconnected);
                
                        }

                        self.pipline_output_count_incrementor.inc();

                        break;

                    }

                }

            }
            
        }

        shutdown(current_connection).await;

        //Send error or other message.

        //Make sure to get rid of the URL as well.

        self.url = None;

        if let Err(_) = self.read_frame_processor_actor_io.input_sender().send(ReadFrameProcessorActorInputMessage::ClientMessage(WebSocketActorOutputClientMessage::Disconnected(SendableText::Str(SERVER_DISCONNECTED)))).await
        {

            return Some(ConnectedLoopExitReason::ActorIOClientDisconnected);

        }

        self.pipline_output_count_incrementor.inc();

        None

    }

    async fn force_disconnection_from_server(&mut self, web_socket_reader: WebSocketReadHalf) -> Option<ConnectedLoopExitReason>
    {

        current_connection.shutdown().await.unwrap();

        self.url = None;

        if let Err(_) = self.read_frame_processor_actor_io.input_sender().send(ReadFrameProcessorActorInputMessage::ClientMessage(WebSocketActorOutputClientMessage::Disconnected(SendableText::Str(SERVER_DISCONNECTION_FORCED)))).await
        {

            return Some(ConnectedLoopExitReason::ActorIOClientDisconnected);

        }

        self.pipline_output_count_incrementor.inc();

        None

    }

    async fn prepare_for_new_connection_and_connect(&mut self, url: String) -> CLEROrConnected //Option<ConnectedLoopExitReason> //bool
    {

        //Check if a zero length string has been provided for the connection URL.

        if url.is_empty()
        {

            if let Err(_) = self.read_frame_processor_actor_io.input_sender().send(ReadFrameProcessorActorInputMessage::ClientMessage(WebSocketActorOutputClientMessage::ConnectionFailed(SendableText::Str(ERROR_EMPTY_URL_PROVIDED)))).await //self.actor_io_server.output_sender().send(WebSocketActorOutputMessage::ClientMessage(WebSocketActorOutputClientMessage::ConnectionFailed(MovableText::Str(ERROR_EMPTY_URL_PROVIDED)))).await
            {

                return CLEROrConnected::CLER(Some(ConnectedLoopExitReason::ActorIOClientDisconnected));

            }

            self.pipline_output_count_incrementor.inc();

            return CLEROrConnected::CLER(Some(ConnectedLoopExitReason::InvalidInput));

        }

        let parsed_url;
        
        match Url::parse(&url)
        {

            Ok(res) =>
            {

                parsed_url = res;

            }
            Err(err) =>
            {

                if let Err(_) = self.read_frame_processor_actor_io.input_sender().send(ReadFrameProcessorActorInputMessage::ClientMessage(WebSocketActorOutputClientMessage::NotConnected(SendableText::String(err.to_string())))).await
                {
        
                    return CLEROrConnected::CLER(Some(ConnectedLoopExitReason::ActorIOClientDisconnected));
        
                }
    
                self.pipline_output_count_incrementor.inc();

                return CLEROrConnected::CLER(Some(ConnectedLoopExitReason::InvalidInput));

            }

        }

        match self.connect_to_server(&parsed_url).await //&url).await
        {

            Ok(res) => 
            {

                //Split the stream here

                let (read, write) = res.0.split(tokio::io::split);

                /*
                let tokio_upgraded = res.0.into_inner();

                let upgraded = tokio_upgraded.into_inner();

                //let stream: TcpStream = upgraded.into();

                let dc = upgraded.downcast::<TcpStream>();
                */

                let reader = WebSocketReader::FragmentCollectorRead(FragmentCollectorRead::new(read));

                //Do something with the connection response,

                self.url = Some(parsed_url); //Some(url);

                //self.current_state = WebSocketConnectionState::Connected;

                //Connected!

                if let Err(_) = self.read_frame_processor_actor_io.input_sender().send(ReadFrameProcessorActorInputMessage::ClientMessage(WebSocketActorOutputClientMessage::ConnectionSucceed(SendableText::Str(CONNECTION_SUCCEEDED)))).await
                {

                    return CLEROrConnected::CLER(Some(ConnectedLoopExitReason::ActorIOClientDisconnected));

                }

                self.pipline_output_count_incrementor.inc();

                //Return the writer side.

                return CLEROrConnected::Connected(write);

            },
            Err(err) =>
            {

                //println!("{err:?}");

                let err_string = err.to_string();

                //Send Error message to the actor-client

                if let Err(_) = self.read_frame_processor_actor_io.input_sender().send(ReadFrameProcessorActorInputMessage::ClientMessage(WebSocketActorOutputClientMessage::ConnectionFailed(SendableText::String(err_string)))).await
                {

                    return CLEROrConnected::CLER(Some(ConnectedLoopExitReason::ActorIOClientDisconnected));

                }

                self.pipline_output_count_incrementor.inc();

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
            WebSocketActorInputMessage::Disconnect | WebSocketActorInputMessage::WriteFrame(_) => //| WebSocketActorInputMessage::SendPing(_) =>
            {

                return self.report_not_connected().await;

            }

        }

    }

    async fn report_not_connected(&mut self) -> ContinueOrConnected
    {

        if let Err(_) = self.read_frame_processor_actor_io.input_sender().send(ReadFrameProcessorActorInputMessage::ClientMessage(WebSocketActorOutputClientMessage::NotConnected(SendableText::Str(ERROR_NO_SERVER_CONNECTED)))).await
        {

            return ContinueOrConnected::ShouldContinue(false);

        }

        self.pipline_output_count_incrementor.inc();

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

    async fn connected_loop(&mut self, mut web_socket_writer: WebSocketWriteHalf) -> bool
    {

        /*
        enum InputOrReadFrame
        {

            Input(Option<WebSocketActorInputMessage>),
            ReadFrame(Result<OwnedFrame, WebSocketError>)

        }
        */

        //Input only

        loop
        {

            let res = self.input_receiver.recv().await; //=>

            match res
            {

                Ok(message) =>
                {

                    match message
                    {

                        WebSocketActorInputMessage::ConnectTo(url) =>
                        {

                            if !Self::continue_or_not(&self.disconnect_from_server(web_socket_writer, false).await)
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

                        }
                        WebSocketActorInputMessage::Disconnect =>
                        {

                            //User initiated disconnection

                            return Self::continue_or_not(&self.disconnect_from_server(current_connection, false).await);

                        }
                        WebSocketActorInputMessage::WriteFrame(frame) =>
                        {

                            if let Err(err) = self.write_frame(&mut current_connection, frame).await
                            {
        
                                return Self::continue_or_not(&self.on_web_socket_error(err, current_connection).await);
        
                            }

                        }
                        /*
                        WebSocketActorInputMessage::SendPing(message) =>
                        {



                        }
                        */

                    }

                }
                Err(err) =>
                {



                }

            }

                /*
                {

                    connected_loop_next_move = res; //= InputOrReadFrame::Input(res);

                }
                */

                /*
            match connected_loop_next_move
            {

                InputOrReadFrame::Input(input) =>
                {

                    if let Some(message) = input
                    {

                        match message
                        {

                            WebSocketActorInputMessage::ConnectTo(url) =>
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

                            }
                            WebSocketActorInputMessage::Disconnect =>
                            {

                                //User initiated disconnection

                                return Self::continue_or_not(&self.disconnect_from_server(current_connection, false).await);

                            }
                            WebSocketActorInputMessage::WriteFrame(frame) =>
                            {

                                if let Err(err) = self.write_frame(&mut current_connection, frame).await
                                {
            
                                    return Self::continue_or_not(&self.on_web_socket_error(err, current_connection).await);
            
                                }

                            }
                            /*
                            WebSocketActorInputMessage::SendPing(message) =>
                            {



                            }
                            */

                        }

                    }
                    else
                    {

                        //ActorIOCleint input receiver has disconnected.

                        return false;

                    }

                }
                InputOrReadFrame::ReadFrame(res) =>
                {

                    match res
                    {

                        Ok(frame) =>
                        {

                            //Process the frame

                            match frame.opcode
                            {
        
                                OpCode::Close =>
                                {
        
                                    //Close frame not sent, connection closure initiated by the server.
        
                                    //let res = self.report_close_frame_received().await;

                                    if let Err(_res) = self.read_frame_processor_actor_io.input_sender().send(ReadFrameProcessorActorInputMessage::FrameAndMessage(frame, WebSocketActorOutputClientMessage::CloseFrameReceived(SendableText::Str(CLOSE_FRAME_RECEIVED)))).await //ReadFrameProcessorActorInputMessage::Frame(frame)).await
                                    {

                                        shutdown(current_connection).await;
                            
                                        return false;
                
                                    }
        
                                    /*
                                    if !Self::continue_or_not(&res)
                                    {
        
                                        current_connection.shutdown().await.unwrap();
        
                                        return false;
        
                                    }
                                    */
        
                                    return Self::continue_or_not(&self.disconnect_from_server(current_connection, true).await);
        
                                }
                                OpCode::Ping =>
                                {
        
                                   //return Self::continue_or_not(&self.report_ping_frame_received().await);

                                   if let Err(_res) = self.read_frame_processor_actor_io.input_sender().send(ReadFrameProcessorActorInputMessage::FrameAndMessage(frame, WebSocketActorOutputClientMessage::PingFrameReceived(SendableText::Str(PING_FRAME_RECEIVED)))).await
                                   {

                                       shutdown(current_connection).await;
                           
                                       return false;
               
                                   }
        
                                }
                                /*
                                OpCode::Pong =>
                                {
        
                                    //return Self::continue_or_not(&self.report_pong_frame_received().await);
        
                                    if let Err(res) = self.read_frame_processor_actor_io.input_sender().send(ReadFrameProcessorActorInputMessage::Frame(frame)).await
                                    {

                                        shutdown(current_connection);
                            
                                        return false;
                
                                    }

                                }
                                */
                                OpCode::Continuation | OpCode::Text | OpCode::Binary | OpCode::Pong =>
                                {
        
                                    if let Err(_err) = self.read_frame_processor_actor_io.input_sender().send(ReadFrameProcessorActorInputMessage::Frame(frame)).await
                                    {
                            
                                        return false;
                
                                    }
        
                                    self.pipline_output_count_incrementor.inc();
        
                                }
        
                            }

                        }
                        Err(error) =>
                        {

                            return Self::continue_or_not(&self.on_web_socket_error(error, current_connection).await);

                        }

                    }

                }

            }
            */

        }

    }

    //Reads a frame from the proveded connection reference into an OwnedFrame.

    /*
    async fn read_frame(current_connection: &mut CurrentConnection, pipline_output_count_incrementor: &Incrementor) -> Result<OwnedFrame, WebSocketError> //in_the_read_pipeline_count: &AtomicUsize) -> Result<OwnedFrame, WebSocketError>
    {

        let frame = current_connection.read_frame().await?;

        //in_the_read_pipeline_count.fetch_add(1, Ordering::SeqCst);

        pipline_output_count_incrementor.inc();

        //Get OwnedFrame from cache...

        let mut of = OwnedFrame::new();

        of.copy_from_read_frame(&frame);

        Ok(of)

    }
    */

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

    async fn on_web_socket_error(&mut self, error: WebSocketError, web_socket_reader: WebSocketReadHalf) -> Option<ConnectedLoopExitReason> //, received_close_frame: bool) -> Option<ConnectedLoopExitReason>
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

        if let Err(_) = self.read_frame_processor_actor_io.input_sender().send(ReadFrameProcessorActorInputMessage::ClientMessage(WebSocketActorOutputClientMessage::ConnectionFailed(SendableText::String(error.to_string())))).await
        {

            return Some(ConnectedLoopExitReason::ActorIOClientDisconnected);

        }

        self.pipline_output_count_incrementor.inc();

        None

    }

    /*
    async fn report_ping_received(&mut self) -> Option<ConnectedLoopExitReason>
    {

        if let Err(_) = self.read_frame_processor_actor_io.input_sender().send(ReadFrameProcessorActorInputMessage::ClientMessage(WebSocketActorOutputClientMessage::PingFrameReceived(SendableText::Str(PING_FRAME_RECEIVED)))).await
        {

            return Some(ConnectedLoopExitReason::ActorIOClientDisconnected);

        }

        self.pipline_output_count_incrementor.inc();

        None

    }
    */

    /*
    async fn send_ping(&mut self, current_connection: &mut CurrentConnection) -> Option<ConnectedLoopExitReason>
    {

        //Send ping, report error or report that the ping has been sent, basically.

        let ping_frame = Frame::new(true, OpCode::Ping, None, vec![].into());

        if let Err(err) = current_connection.write_frame(ping_frame).await
        {

            return self.on_web_socket_error_report_only(err).await;

        }

        if let Err(_) = self.read_frame_processor_actor_io.input_sender().send(ReadFrameProcessorActorInputMessage::ClientMessage(WebSocketActorOutputClientMessage::PingFrameReceived(SendableText::Str(PING_FRAME_SENT)))).await
        {

            return Some(ConnectedLoopExitReason::ActorIOClientDisconnected);

        }

        self.pipline_output_count_incrementor.inc();

        None

    }
    */

    /*
    async fn report_ping_frame_received(&mut self) -> Option<ConnectedLoopExitReason>
    {

        if let Err(_) = self.read_frame_processor_actor_io.input_sender().send(ReadFrameProcessorActorInputMessage::ClientMessage(WebSocketActorOutputClientMessage::PingFrameReceived(SendableText::Str(PING_FRAME_RECEIVED)))).await
        {

            return Some(ConnectedLoopExitReason::ActorIOClientDisconnected);

        }

        self.pipline_output_count_incrementor.inc();

        None

    }

    async fn report_pong_frame_received(&mut self) -> Option<ConnectedLoopExitReason>
    {

        if let Err(_) = self.read_frame_processor_actor_io.input_sender().send(ReadFrameProcessorActorInputMessage::ClientMessage(WebSocketActorOutputClientMessage::PongFrameReceived(SendableText::Str(PONG_FRAME_RECEIVED)))).await
        {

            return Some(ConnectedLoopExitReason::ActorIOClientDisconnected);

        }

        self.pipline_output_count_incrementor.inc();

        None

    }

    async fn report_close_frame_received(&mut self) -> Option<ConnectedLoopExitReason>
    {

        if let Err(_) = self.read_frame_processor_actor_io.input_sender().send(ReadFrameProcessorActorInputMessage::ClientMessage(WebSocketActorOutputClientMessage::CloseFrameReceived(SendableText::Str(CLOSE_FRAME_RECEIVED)))).await
        {

            return Some(ConnectedLoopExitReason::ActorIOClientDisconnected);

        }

        self.pipline_output_count_incrementor.inc();

        None

    }
    */

}

//Setup the macro generated Task actor.

impl_mac_task_actor!(WebSocketActor);


struct ReadWebSocketActorState
{



}

impl WebSocketActorState
{

    pub fn new(read_frame_processor_actor_io: ActorIOClient<ReadFrameProcessorActorInputMessage, ReadFrameProcessorActorOutputMessage>, input_receiver: Receiver<WebSocketActorInputMessage>, pipline_output_count_incrementor: Incrementor) -> Self //in_the_read_pipeline_count: Arc<AtomicUsize>, //(Sender<WebSocketActorInputMessage>, Self) //(ActorIOClient<WebSocketActorInputMessage, WebSocketActorOutputMessage>, Self) //read_frame_processor_actor: ReadFrameProcessorActor) -> Self //, read_frame_proccessor_input_sender: Sender<ReadFrameProcessorActorInputMessage>) -> Self
    {

        Self
        {

        }

    }

}


/*
//Shutdown the current connection.

async fn shutdown(web_socket_reader: WebSocketRead)
{

    current_connection.shutdown().await.unwrap();

}
*/