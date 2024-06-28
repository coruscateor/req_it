use act_rs::{ActorFrontend, ActorState, HasInteractor, impl_mac_task_actor, DroppedIndicator, impl_default_on_enter_async, impl_default_on_exit_async, impl_default_on_enter_and_exit_async};

//impl_mac_runtime_task_actor

use act_rs::tokio::interactors::mpsc::{ActorIOInteractorClient, ActorIOInteractorServer, actor_io_interactors};

use fastwebsockets::{handshake, WebSocket};

use gtk_estate::corlib::MovableText;
use hyper::body::Incoming;
use hyper::Response;
use tokio::io::AsyncWriteExt;
use tokio::sync::mpsc::Sender;
use tokio::{sync::mpsc::Receiver, runtime::Handle};

use std::future::Future;
use std::{marker::PhantomData, sync::Arc};

use tokio::runtime::Runtime;

use act_rs::ActorInteractor;

//use reqwest::{self, Client, header::{HeaderMap, HeaderValue}};

use std::collections::HashMap;

use std::time::{Duration, Instant};

//use pretty_goodness::json::PrettyEr;

use serde_json::{Value, Map};

use tokio::net::TcpStream;

use hyper::{Request, body::Bytes, upgrade::Upgraded, header::{UPGRADE, CONNECTION}};

use http_body_util::Empty;

use anyhow::Result;

use hyper_util::rt::TokioIo;

use super::{WebSocketActorInputMessage, WebSocketActorOutputClientMessage, WebSocketActorOutputMessage};

static CONNECTION_SUCCEEDED: &str = "Connection Succeeded!";

//static CONNECTION_FAILED: &str = "Connection Faild!";

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

//https://docs.rs/tokio/1.38.0/tokio/io/trait.AsyncWriteExt.html#method.shutdown

pub struct WebSocketActorState
{

    sender_input: ActorIOInteractorClient<WebSocketActorInputMessage, WebSocketActorOutputMessage>, //Sender<WebSocketActorInputMessage>,
    receiver_input: ActorIOInteractorServer<WebSocketActorInputMessage, WebSocketActorOutputMessage>, //Receiver<WebSocketActorInputMessage>,
    //connection_stream: Option<TcpStream>
    web_socket: Option<WebSocket<TokioIo<Upgraded>>>,
    url: Option<String>

}

impl WebSocketActorState
{

    pub fn new() -> Self
    {

        //let (sender_input, reciver_input) = channel(50);

        //let (sender_input, reciver_input) = tokio::sync::mpsc::channel(50);

        let (sender_input, receiver_input) = actor_io_interactors(10, 1000);

        Self
        {

            sender_input,
            receiver_input,
            web_socket: None,
            url: None

        }

    }

    //Default on_enter_async and on_exit_async implementations.

    impl_default_on_enter_and_exit_async!();

    async fn run_async(&mut self, di: &DroppedIndicator) -> bool
    {

        if let Some(val) = self.receiver_input.input_receiver().recv().await
        {

            match val
            {

                WebSocketActorInputMessage::ConnectTo(url) =>
                {

                    /*
                    if let Some(connection) = &mut self.connection_stream
                    {

                        connection.shutdown();

                    }
                    */

                    /*
                    if let Some(ws) = &mut self.web_socket
                    {

                        ws.

                    }
                    */

                    match self.web_socket.take()
                    {

                        Some(ws) =>
                        {

                            //Make syre the stream gets shutdown correctly.

                            let _ = ws.into_inner().shutdown().await;

                            //Send error or other message.

                            //Make sure to get rid of the URL as well.

                            self.url = None;

                        }
                        None => {}

                    }

                    match self.connect_to_server(&url).await
                    {

                        Ok(res) => 
                        {

                            self.web_socket = Some(res.0);

                            //Do something with the connection response,

                            self.url = Some(url);

                            //Connected!

                            /*
                                `web_socket_actor_message::WebSocketActorOutputMessage` doesn't implement `std::fmt::Debug`
                                the trait `std::fmt::Debug` is not implemented for `web_socket_actor_message::WebSocketActorOutputMessage`, which is required by `tokio::sync::mpsc::error::SendError<web_socket_actor_message::WebSocketActorOutputMessage>: std::fmt::Debug`
                                add `#[derive(Debug)]` to `web_socket_actor_message::WebSocketActorOutputMessage` or manually `impl std::fmt::Debug for web_socket_actor_message::WebSocketActorOutputMessage`
                                the trait `std::fmt::Debug` is implemented for `tokio::sync::mpsc::error::SendError<T>`
                                required for `tokio::sync::mpsc::error::SendError<web_socket_actor_message::WebSocketActorOutputMessage>` to implement `std::fmt::Debug`rustcClick for full compiler diagnostic
                                result.rs(1073, 12): required by a bound in `Result::<T, E>::unwrap`
                            */

                            if let Err(_) = self.receiver_input.output_sender().send(WebSocketActorOutputMessage::ClientMessage(WebSocketActorOutputClientMessage::ConnectionResult(MovableText::Str(CONNECTION_SUCCEEDED)))).await //.unwrap();
                            {

                                panic!("This should've sent");

                            }

                            //res.read_frame()

                        },
                        Err(err) =>
                        {

                            let err_string = err.to_string();

                            //Send Error message to the actor-client

                            if let Err(_) = self.receiver_input.output_sender().send(WebSocketActorOutputMessage::ClientMessage(WebSocketActorOutputClientMessage::ConnectionResult(MovableText::String(err_string)))).await //.unwrap();
                            {

                                panic!("This should've sent");

                            }

                        }

                    }


                }

            }

        }

        di.not_dropped()

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

    /*
    fn check_send_error(send_res: Result<(), GraphQLRequestResult>)
    {

        if let Err(_err) = send_res
        {

            println!("GraphQLMessage Send Error");

        }

    }
    */

}

impl HasInteractor<ActorIOInteractorClient<WebSocketActorInputMessage, WebSocketActorOutputMessage>> for WebSocketActorState
{

    fn interactor(&self) -> &ActorIOInteractorClient<WebSocketActorInputMessage, WebSocketActorOutputMessage>
    {
       
       &self.sender_input

    }

}

//Setup the macro generated Task actor.

impl_mac_task_actor!(WebSocketActorState, ActorIOInteractorClient<WebSocketActorInputMessage, WebSocketActorOutputMessage>, WebSocketActor);

