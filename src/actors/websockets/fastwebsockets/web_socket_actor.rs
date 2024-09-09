use act_rs::{impl_default_end_async, impl_default_start_and_end_async, impl_default_start_async, impl_mac_task_actor, impl_mac_task_actor_built_state, ActorFrontend, ActorState};

use crossbeam::queue::ArrayQueue;

use fastwebsockets::{handshake, FragmentCollector, FragmentCollectorRead, Frame, OpCode, WebSocket, WebSocketError, WebSocketRead, WebSocketWrite};

use corlib::text::SendableText;

use gtk_estate::should_continue;

use hyper::body::Incoming;

use hyper::Response;

use libsync::std::{CountedPipelineMessage, IncrementedPipelineMessageCounter, CountedPipelineMessageMut, PipelineMessageCounter};

use tokio::io::{self, AsyncWriteExt, ReadHalf, WriteHalf};

use tokio::select;

use url::Url;

use std::future::Future;

use std::sync::atomic::{AtomicI32, AtomicUsize};

use std::{marker::PhantomData, sync::Arc};

use tokio::runtime::Runtime;

use std::collections::HashMap;

use tokio::time::{Duration, Instant};

use tokio::net::TcpStream;

use hyper::{Request, body::Bytes, upgrade::Upgraded, header::{UPGRADE, CONNECTION}};

use http_body_util::Empty;

use anyhow::{Error, Result};

use hyper_util::rt::TokioIo;

use tokio::task::JoinHandle;

use crate::actors::websockets::fastwebsockets::{OwnedFrame, ReadFrameProcessorActorInputMessage};

use super::websocket_read_and_write::{WebSocketReadHalf, WebSocketWriteHalf};

use super::{ReadFrameProcessorActorOutputMessage, WebSocketActorInputMessage, WebSocketActorOutputClientMessage};

use paste::paste;

use tokio::time::timeout_at;

use libsync::crossbeam::mpmc::tokio::array_queue::{Sender, Receiver, channel};

use crate::actors::websockets::fastwebsockets::websocket_read_and_write::WebSocketReader;

use super::array_queue::{ActorIOClient, ActorIOServer, actor_io};

static CONNECTION_SUCCEEDED: &str = "Connection succeeded!";

static ERROR_EMPTY_URL_PROVIDED: &str = "Error: Empty URL provided.";

static SERVER_DISCONNECTED: &str = "Server disconnected";

static ERROR_NO_SERVER_CONNECTED: &str = "Error: No server connected.";

static CLOSE_FRAME_SENT: &str = "Close frame Sent.";

static TIME_ELAPSED_FORCED_CLOSURE_NOTICE: &str = "Close Connection Response Time Has Elapsed: Forcing closure of connection.";

static SERVER_DISCONNECTION_FORCED: &str = "Forced server disconnection";

static PING_FRAME_RECEIVED: &str = "Ping frame received - pong frame already sent";

static PING_FRAME_SENT: &str = "Ping frame sent.";

static PONG_FRAME_RECEIVED: &str = "Pong frame received.";

static CLOSE_FRAME_RECEIVED: &str = "Close frame received - Close frame already sent.";

enum ConnectedLoopExitReason
{

    ReadFrameProcessorActorDropped,
    ReadWebSocketActorDropped,
    ServerDisconnectedOrConnectionError,
    InvalidInput

}

impl ConnectedLoopExitReason
{

    pub fn should_continue(&self) -> bool
    {

        match self
        {

            ConnectedLoopExitReason::ReadFrameProcessorActorDropped | ConnectedLoopExitReason::ReadWebSocketActorDropped => false,
            ConnectedLoopExitReason::ServerDisconnectedOrConnectionError | ConnectedLoopExitReason::InvalidInput => true,

        }

    }

}

enum ContinueOrConnected
{

    ShouldContinue(bool),
    Connected(WebSocketWriteHalf)

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

    //https://docs.rs/tokio/latest/tokio/runtime/struct.EnterGuard.html

    fn execute(&self, fut: Fut)
    {

        tokio::task::spawn(fut);
        
    }

}

//WriteFrameProcessorActor -> WebSocketActor <-> ReadWebSocketActor -> ReadFrameProcessorActor

//WebSocketActor -> ReadFrameProcessorActor

//WebSocketActors input queue/channel can be accessed via WriteFrameProcessorActors ActorIOClient directly, allowing it to be bypassed.

//Writing WebSocket Actor

pub struct WebSocketActorState
{

    input_receiver: Receiver<WebSocketActorInputMessage>,
    url: Option<Url>,
    pipeline_message_counter: PipelineMessageCounter,
    read_web_socket_actor_io_client: ActorIOClient<ReadWebSocketActorInputMessage, WebSocketActorInputMessage>,
    read_frame_processor_actor_input_sender: Sender<CountedPipelineMessageMut<ReadFrameProcessorActorInputMessage>>

}

impl WebSocketActorState
{

    pub fn new(input_receiver: Receiver<WebSocketActorInputMessage>, pipeline_message_counter: &PipelineMessageCounter, read_frame_processor_actor_input_sender: &Sender<CountedPipelineMessageMut<ReadFrameProcessorActorInputMessage>>) -> Self
    {

        let read_web_socket_actor_io_client = ReadWebSocketActorState::spawn(read_frame_processor_actor_input_sender, &pipeline_message_counter);

        Self
        {

            input_receiver,
            url: None,
            pipeline_message_counter: pipeline_message_counter.clone(),
            read_web_socket_actor_io_client: read_web_socket_actor_io_client,
            read_frame_processor_actor_input_sender: read_frame_processor_actor_input_sender.clone()
        }

    }

    pub fn spawn(pipeline_message_counter: &PipelineMessageCounter, read_frame_processor_actor_input_sender: &Sender<CountedPipelineMessageMut<ReadFrameProcessorActorInputMessage>>) -> Sender<WebSocketActorInputMessage>
    {

        let (input_sender, input_receiver) = channel(50);

        let state = WebSocketActorState::new(input_receiver, pipeline_message_counter, read_frame_processor_actor_input_sender);

        WebSocketActor::spawn(state);

        input_sender

    }

    //Default on_enter_async and on_exit_async implementations.

    impl_default_start_and_end_async!();

    //The non-connected loop

    async fn run_async(&mut self) -> bool
    {

        if let Some(message) = self.input_receiver.recv().await
        {

            let res = self.process_received_actor_input_message(message).await;

            match res
            {

                ContinueOrConnected::ShouldContinue(should_continue) => should_continue,
                ContinueOrConnected::Connected(web_socket_writer) =>
                {

                    //Connected loop
    
                    if !self.connected_loop(web_socket_writer).await
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
    
    async fn connect_to_server(&mut self, url: &Url) -> Result<(WebSocket<TokioIo<Upgraded>>, Response<Incoming>)>
    {

        //Get from cache...

        let mut host_and_port = String::with_capacity(50);

        match url.host_str()
        {

            Some(the_host) =>
            {

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

        let connection_stream = TcpStream::connect(&host_and_port).await?;

        let request = Request::builder()
            .method("GET")
            .uri(url.as_str())
            .header("Host", host_and_port)
            .header(UPGRADE, "websocket")
            .header(CONNECTION, "upgrade")
            .header("Sec-WebSocket-Key", handshake::generate_key())
            .header("Sec-WebSocket-Version", "13") //"Sec-WebSocket-Verion" OMG!!!!!!
            .body(Empty::<Bytes>::new())?;

        let (ws, res) = handshake::client(&SpawnExecutor, request, connection_stream).await?;

        Ok((ws, res))

    }

    ///
    /// Shuts down and drops the connection. The WebSocket connection closure process should've been comnpleted by this point.
    /// 
    async fn disconnect_from_server(&mut self, mut web_socket_writer: WebSocketWriteHalf, received_close_frame: bool) -> Option<ConnectedLoopExitReason>
    {

        //If received_close_frame is false, send the close response frame, otherwise initiate the connection closure process.

        if let Err(err) = self.send_close_frame(&mut web_socket_writer).await
        {

            let res = self.on_web_socket_error_report_only(err).await;

            if !Self::continue_or_not(&res)
            {

                return res;

            }

        }

        let counted = self.pipeline_message_counter.increment_with_message_mut(ReadFrameProcessorActorInputMessage::ClientMessage(WebSocketActorOutputClientMessage::Disconnecting(SendableText::Str(CLOSE_FRAME_SENT))));

        if let Err(_) = self.read_frame_processor_actor_input_sender.send(counted).await
        {

            return Some(ConnectedLoopExitReason::ReadFrameProcessorActorDropped);

        }

        if !received_close_frame
        {

            //Wait fot the close frame response.

            let now = Instant::now();

            let soon = now.checked_add(Duration::from_secs(10)).expect("Error: Instant problems");

            //Disconnection loop
            
            loop
            {
                
                let output_receiver_recv = self.read_web_socket_actor_io_client.output_receiver().recv();

                match timeout_at(soon, output_receiver_recv).await
                {

                    Ok(res) =>
                    {
                        
                        match res
                        {

                            Some(message) =>
                            {

                                match message
                                {

                                    WebSocketActorInputMessage::ConnectTo(url) =>
                                    {

                                        let error_message = format!("Error: Cannot connect to {url} as the client is disconnecting.");

                                        let counted_message = self.pipeline_message_counter.increment_with_message_mut(ReadFrameProcessorActorInputMessage::ClientMessage(WebSocketActorOutputClientMessage::ConnectionError(SendableText::String(error_message))));

                                        if let Err(_err) = self.read_frame_processor_actor_input_sender.send(counted_message).await
                                        {
                                
                                            return Some(ConnectedLoopExitReason::ReadFrameProcessorActorDropped);
                    
                                        }

                                    }
                                    WebSocketActorInputMessage::Disconnect =>
                                    {

                                        return Some(ConnectedLoopExitReason::ReadWebSocketActorDropped);

                                    }
                                    WebSocketActorInputMessage::WriteFrame(frame) =>
                                    {

                                        match frame.opcode
                                        {
        
                                            OpCode::Close =>
                                            {
        
                                                let message = self.pipeline_message_counter.increment_with_message_mut(ReadFrameProcessorActorInputMessage::FrameAndMessage(frame, WebSocketActorOutputClientMessage::CloseFrameReceived(SendableText::Str(CLOSE_FRAME_RECEIVED))));
        
                                                if let Err(_err) = self.read_frame_processor_actor_input_sender.send(message).await
                                                {
                                                    
                                                    return Some(ConnectedLoopExitReason::ReadFrameProcessorActorDropped);
                            
                                                }

                                                //Tell the ReadWebSocketActor to drop its side of the WebSocket stream.

                                                if let Err(_) = self.read_web_socket_actor_io_client.input_sender().send(ReadWebSocketActorInputMessage::Disconnect).await
                                                {

                                                    return Some(ConnectedLoopExitReason::ReadFrameProcessorActorDropped);

                                                }
                                                
                                                break;
        
                                            }
                                            OpCode::Continuation | OpCode::Text | OpCode::Binary | OpCode::Pong =>
                                            {
        
                                                let message = self.pipeline_message_counter.increment_with_message_mut(ReadFrameProcessorActorInputMessage::Frame(frame));
        
                                                if let Err(_err) = self.read_frame_processor_actor_input_sender.send(message).await
                                                {
                                        
                                                    return Some(ConnectedLoopExitReason::ReadFrameProcessorActorDropped);
                            
                                                }

                                            }
                                            OpCode::Ping =>
                                            {
        
                                                let message = self.pipeline_message_counter.increment_with_message_mut(ReadFrameProcessorActorInputMessage::FrameAndMessage(frame, WebSocketActorOutputClientMessage::PingFrameReceived(SendableText::Str(PING_FRAME_RECEIVED))));
        
                                                if let Err(_err) = self.read_frame_processor_actor_input_sender.send(message).await
                                                {
                                        
                                                    return Some(ConnectedLoopExitReason::ReadFrameProcessorActorDropped);
                            
                                                }
        
                                            }
                    
                                        }

                                    }

                                }

                            }
                            None =>
                            {

                                return Some(ConnectedLoopExitReason::ReadWebSocketActorDropped);

                            }

                        }

                    },
                    Err(_err) =>
                    {

                        let message = self.pipeline_message_counter.increment_with_message_mut(ReadFrameProcessorActorInputMessage::ClientMessage(WebSocketActorOutputClientMessage::Disconnected(SendableText::Str(TIME_ELAPSED_FORCED_CLOSURE_NOTICE))));

                        if let Err(_) = self.read_frame_processor_actor_input_sender.send(message).await
                        {
                
                            return Some(ConnectedLoopExitReason::ReadFrameProcessorActorDropped);
                
                        }

                        break;

                    }

                }

            }
            
        }
        else
        {

            //Tell the ReadWebSocketActor to drop its side of the WebSocket stream.

            if let Err(_) = self.read_web_socket_actor_io_client.input_sender().send(ReadWebSocketActorInputMessage::Disconnect).await
            {

                return Some(ConnectedLoopExitReason::ReadFrameProcessorActorDropped);

            }
            
        }

        //Send error or other message.

        //Make sure to drop the URL as well.

        self.url = None;

        let message = self.pipeline_message_counter.increment_with_message_mut(ReadFrameProcessorActorInputMessage::ClientMessage(WebSocketActorOutputClientMessage::Disconnected(SendableText::Str(SERVER_DISCONNECTED))));

        if let Err(_) = self.read_frame_processor_actor_input_sender.send(message).await
        {

            return Some(ConnectedLoopExitReason::ReadFrameProcessorActorDropped);

        }

        None

    }

    async fn prepare_for_new_connection_and_connect(&mut self, url: String) -> CLEROrConnected
    {

        //Check if a zero length String has been provided for the connection URL.

        if url.is_empty()
        {

            let counted_message = self.pipeline_message_counter.increment_with_message_mut(ReadFrameProcessorActorInputMessage::ClientMessage(WebSocketActorOutputClientMessage::ConnectionError(SendableText::Str(ERROR_EMPTY_URL_PROVIDED))));

            if let Err(_) = self.read_frame_processor_actor_input_sender.send(counted_message).await
            {

                return CLEROrConnected::CLER(Some(ConnectedLoopExitReason::ReadFrameProcessorActorDropped));

            }

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

                let counted_message = self.pipeline_message_counter.increment_with_message_mut(ReadFrameProcessorActorInputMessage::ClientMessage(WebSocketActorOutputClientMessage::NotConnected(SendableText::String(err.to_string()))));

                if let Err(_) = self.read_frame_processor_actor_input_sender.send(counted_message).await
                {
        
                    return CLEROrConnected::CLER(Some(ConnectedLoopExitReason::ReadFrameProcessorActorDropped));
        
                }

                return CLEROrConnected::CLER(Some(ConnectedLoopExitReason::InvalidInput));

            }

        }

        match self.connect_to_server(&parsed_url).await
        {

            Ok(res) => 
            {

                //Split the stream here

                let (read, write) = res.0.split(tokio::io::split);

                //Setup the read-half of the websocket stream in the WebSocketReader and send it to the ReadWebSocketActor.

                let reader = WebSocketReader::FragmentCollectorRead(FragmentCollectorRead::new(read));

                if let Err(_) = self.read_web_socket_actor_io_client.input_sender().send(ReadWebSocketActorInputMessage::Connected(reader)).await
                {

                    return CLEROrConnected::CLER(Some(ConnectedLoopExitReason::ReadWebSocketActorDropped));

                }

                //Do something with the connection response,

                self.url = Some(parsed_url);

                //Connected!

                let counted_message = self.pipeline_message_counter.increment_with_message_mut(ReadFrameProcessorActorInputMessage::ClientMessage(WebSocketActorOutputClientMessage::ConnectionSucceed(SendableText::Str(CONNECTION_SUCCEEDED))));

                if let Err(_) = self.read_frame_processor_actor_input_sender.send(counted_message).await
                {

                    return CLEROrConnected::CLER(Some(ConnectedLoopExitReason::ReadFrameProcessorActorDropped));

                }

                //Return the writer side.

                return CLEROrConnected::Connected(write);

            },
            Err(err) =>
            {
                
                let err_string = err.to_string();

                //Send Error message to the actor-client

                let counted_message = self.pipeline_message_counter.increment_with_message_mut(ReadFrameProcessorActorInputMessage::ClientMessage(WebSocketActorOutputClientMessage::ConnectionError(SendableText::String(err_string))));

                if let Err(_) = self.read_frame_processor_actor_input_sender.send(counted_message).await
                {

                    return CLEROrConnected::CLER(Some(ConnectedLoopExitReason::ReadFrameProcessorActorDropped));

                }

                return CLEROrConnected::CLER(Some(ConnectedLoopExitReason::ServerDisconnectedOrConnectionError));

            }

        }

    }

    //Disconnected: Not connected to a server, should continue?

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
            WebSocketActorInputMessage::Disconnect | WebSocketActorInputMessage::WriteFrame(_) =>
            {

                return self.report_not_connected().await;

            }

        }

    }

    async fn report_not_connected(&mut self) -> ContinueOrConnected
    {

        let counted_message = self.pipeline_message_counter.increment_with_message_mut(ReadFrameProcessorActorInputMessage::ClientMessage(WebSocketActorOutputClientMessage::NotConnected(SendableText::Str(ERROR_NO_SERVER_CONNECTED))));

        if let Err(_) = self.read_frame_processor_actor_input_sender.send(counted_message).await
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

    async fn connected_loop(&mut self, mut web_socket_writer: WebSocketWriteHalf) -> bool
    {

        loop
        {

            let res;

            let read_web_socket_actor_io_client_output_receiver_recv = self.read_web_socket_actor_io_client.output_receiver().recv();

            let input_receiver_recv = self.input_receiver.recv();

            //et res2;
    
            //let _ = self.read_web_socket_actor_io_client.output_receiver().try_recv();

            let from_server;

            select!
            {
                
                biased;

                connected_res = read_web_socket_actor_io_client_output_receiver_recv =>
                {

                    //From the associated ReadWebSocketActor.

                    res = connected_res;

                    from_server = true;

                    //res2 = Some(self.read_web_socket_actor_io_client.output_receiver().try_recv());

                }
                connected_res = input_receiver_recv =>
                {

                    //From the GUI.

                    res = connected_res;

                    from_server = false;

                    //res2 = None;

                }
    
            }
            
            match res
            {

                Some(message) =>
                {

                    match message
                    {

                        WebSocketActorInputMessage::ConnectTo(url) =>
                        {

                            //Ignore any ConnectTo messages from the ReadWebSocketActor.

                            if from_server
                            {

                                continue;

                            }

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
                                CLEROrConnected::Connected(new_web_socket_writer) =>
                                {
        
                                    web_socket_writer = new_web_socket_writer;

                                    //The new connection is set, stay in the loop. 
        
                                }
        
                            }

                        }
                        WebSocketActorInputMessage::Disconnect =>
                        {

                            //Likely user initiated disconnection

                            return Self::continue_or_not(&self.disconnect_from_server(web_socket_writer, false).await);

                        }
                        WebSocketActorInputMessage::WriteFrame(frame) =>
                        {

                            let opcode = frame.opcode;

                            if let Err(err) = self.write_frame(&mut web_socket_writer, frame).await
                            {
        
                                return Self::continue_or_not(&self.on_web_socket_error(err, web_socket_writer).await);
        
                            }

                            //Did the UI send a close frome? Then from_server = false.

                            //Did the server send the close frame? Then from_server = true.

                            if opcode == OpCode::Close
                            {

                                return Self::continue_or_not( &self.disconnect_from_server(web_socket_writer, from_server).await);

                            }

                        }

                    }

                }
                None =>
                {

                    return false;

                }

            }

        }

    }

    async fn write_frame(&self, web_socket_writer: &mut WebSocketWriteHalf, mut of: OwnedFrame) -> Result<(), WebSocketError>
    {

        let frame = of.new_frame_to_be_written();

        web_socket_writer.write_frame(frame).await?;

        //Put OwnedFrame into cache...

        Ok(())

    }

    async fn send_close_frame(&self, web_socket_writer: &mut WebSocketWriteHalf) -> Result<(), WebSocketError>
    {

        web_socket_writer.write_frame(Frame::close_raw(vec![].into())).await

    }

    async fn on_web_socket_error(&mut self, error: WebSocketError, _web_socket_writer: WebSocketWriteHalf) -> Option<ConnectedLoopExitReason>
    {

        //Make sure to drop the URL as well.

        self.url = None;

        let res = self.on_web_socket_error_report_only(error).await;
        
        if !Self::continue_or_not(&res)
        {

            return res;

        }

        res

    }

    async fn on_web_socket_error_report_only(&mut self, error: WebSocketError) -> Option<ConnectedLoopExitReason>
    {

        let message = self.pipeline_message_counter.increment_with_message_mut(ReadFrameProcessorActorInputMessage::ClientMessage(WebSocketActorOutputClientMessage::ConnectionError(SendableText::String(error.to_string()))));

        if let Err(_) = self.read_frame_processor_actor_input_sender.send(message).await
        {

            return Some(ConnectedLoopExitReason::ReadFrameProcessorActorDropped);

        }

        None

    }

}

//Setup the macro generated Task actor.

impl_mac_task_actor!(WebSocketActor);

//The ReadWebSocketActor

enum ReadWebSocketActorInputMessage
{

    Connected(WebSocketReader),
    Disconnect

}

//Subordinate actor, beginning of read-frame pipeline.

struct ReadWebSocketActorState
{

    read_frame_processor_actor_io_sender: Sender<CountedPipelineMessageMut<ReadFrameProcessorActorInputMessage>>,
    pipeline_message_counter: PipelineMessageCounter,
    obligated_send_frame_holder: Arc<ArrayQueue<OwnedFrame>>,
    io_server: ActorIOServer<ReadWebSocketActorInputMessage, WebSocketActorInputMessage> //Outputs to is contaning WebSocketActor.

}

impl ReadWebSocketActorState
{

    pub fn new(read_frame_processor_actor_io_sender: &Sender<CountedPipelineMessageMut<ReadFrameProcessorActorInputMessage>>, pipeline_message_counter: &PipelineMessageCounter) -> (ActorIOClient<ReadWebSocketActorInputMessage, WebSocketActorInputMessage>, Self)
    {

        let (io_client, io_server) = actor_io(2, 10);

        (io_client,
        Self
        {

            read_frame_processor_actor_io_sender: read_frame_processor_actor_io_sender.clone(),
            pipeline_message_counter: pipeline_message_counter.clone(),
            obligated_send_frame_holder: Arc::new(ArrayQueue::new(1)),
            io_server

        })

    }

    pub fn spawn(read_frame_processor_actor_io_sender: &Sender<CountedPipelineMessageMut<ReadFrameProcessorActorInputMessage>>, pipeline_message_counter: &PipelineMessageCounter) -> ActorIOClient<ReadWebSocketActorInputMessage, WebSocketActorInputMessage>
    {

        let (io_client, state) = ReadWebSocketActorState::new(read_frame_processor_actor_io_sender, pipeline_message_counter);

        ReadWebSocketActor::spawn(state);

        io_client

    }

    impl_default_start_and_end_async!();

    //The non-connected loop.

    async fn run_async(&mut self) -> bool
    {

        loop
        {

            match self.io_server.input_receiver().recv().await
            {

                Some(message) =>
                {

                    match message
                    {

                        ReadWebSocketActorInputMessage::Connected(reader) =>
                        {

                            if !self.connected_loop(reader).await
                            {

                                return false;

                            }

                        }
                        ReadWebSocketActorInputMessage::Disconnect => { /* Continue */ }
                    }

                }
                None => return false
            }
            
        }

    }

    //The connected loop.

    async fn connected_loop(&mut self, mut reader: WebSocketReader) -> bool
    {

        enum SelectResult<'f>
        {

            ReadFrame(Result<Frame<'f>, WebSocketError>),
            Connection(Option<ReadWebSocketActorInputMessage>)

        }

        let obligated_send_frame_holder = self.obligated_send_frame_holder.clone();

        let mut send_fn = |obligated_send_frame: Frame|
        {

            //Get from cache

            let mut of = OwnedFrame::new();

            of.copy_all_from_read_frame(&obligated_send_frame);

            async
            {

                let _ = obligated_send_frame_holder.push(of).expect("Error: The obligated_send_frame_holder should've been checked.");

                Result::<(), WebSocketError>::Ok(())

            }

        };

        loop
        {

            //From the WebSocketActor

            let connection_future = self.io_server.input_receiver().recv();

            //From the WebSocketReadHalf or FragmentCollectorRead.

            let read_frame_future = reader.read_frame(&mut send_fn);

            let select_result;

            //Doesn't look like reader.read_frame is cancel-safe.
            
            //Should only loose one frame at most however.

            select!
            {

                biased;

                res = connection_future =>
                {

                    //This select invocation is biased so that the actor will disconnect or change connection when told. 

                    select_result = SelectResult::Connection(res);

                }
                res = read_frame_future =>
                {

                    //Read from WebSocketReadHalf or FragmentCollectorRead.

                    select_result = SelectResult::ReadFrame(res);

                }

            }

            //Does an obligated frame need to be sent?

            if let Some(mut of) = obligated_send_frame_holder.pop()
            {

                match of.opcode
                {
    
                    OpCode::Close =>
                    {

                        //Send close frame
    
                        of.clear_payload();

                        //From this message the WebSocketActor must either initiate the disconnection process or complete it.
                        
                        if let Err(_err) = self.io_server.output_sender().send(WebSocketActorInputMessage::WriteFrame(of)).await
                        {
        
                            return false;
        
                        }
                        
                    }
                    OpCode::Ping =>
                    {

                        //Now send a pong frame.

                        of.pong_setup();
                        
                        if let Err(_err) = self.io_server.output_sender().send(WebSocketActorInputMessage::WriteFrame(of)).await
                        {
        
                            return false;
        
                        }

                    }
                    OpCode::Continuation | OpCode::Text | OpCode::Binary | OpCode::Pong =>
                    {

                        //If for some reason you get another type of frame...
                        
                        let message = ReadFrameProcessorActorInputMessage::Frame(of);
    
                        let counted_message = self.pipeline_message_counter.increment_with_message_mut(message);
    
                        if let Err(_err) = self.read_frame_processor_actor_io_sender.send(counted_message).await
                        {

                            return false;

                        }
    
                    }
    
                }

            }

            //Send a read frame to the ReadFrameProcessorActor, change connection or disconnect.

            match select_result
            {

                SelectResult::ReadFrame(frame_res) =>
                {

                    match frame_res
                    {

                        Ok(frame) =>
                        {

                            //Get from cache

                            let mut of = OwnedFrame::new();

                            of.copy_all_from_read_frame(&frame);

                            let message = ReadFrameProcessorActorInputMessage::Frame(of);
    
                            let counted_message = self.pipeline_message_counter.increment_with_message_mut(message);
        
                            if let Err(_err) = self.read_frame_processor_actor_io_sender.send(counted_message).await
                            {
    
                                return false;
    
                            }

                        }
                        Err(err) =>
                        {

                            if let Err(_err) = self.io_server.output_sender().send(WebSocketActorInputMessage::Disconnect).await
                            {
    
                                return false;
    
                            }

                            let message = ReadFrameProcessorActorInputMessage::ClientMessage(WebSocketActorOutputClientMessage::ConnectionError(SendableText::String(err.to_string())));
    
                            let counted_message = self.pipeline_message_counter.increment_with_message_mut(message);
        
                            if let Err(_err) = self.read_frame_processor_actor_io_sender.send(counted_message).await
                            {
    
                                return false;
    
                            }

                        }

                    }

                }
                SelectResult::Connection(input_opt) =>
                {

                    if let Some(input) = input_opt
                    {

                        match input
                        {

                            ReadWebSocketActorInputMessage::Connected(new_reader) =>
                            {

                                reader = new_reader;

                            }
                            ReadWebSocketActorInputMessage::Disconnect =>
                            {

                                return true;

                            }

                        }
                        
                    }

                }

            }

        }

    }

}

impl_mac_task_actor!(ReadWebSocketActor);
