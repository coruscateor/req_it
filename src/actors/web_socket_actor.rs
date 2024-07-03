use act_rs::{ActorFrontend, ActorState, HasInteractor, impl_mac_task_actor, DroppedIndicator, impl_default_on_enter_async, impl_default_on_exit_async, impl_default_on_enter_and_exit_async};

//impl_mac_runtime_task_actor

use act_rs::tokio::interactors::mpsc::{ActorIOInteractorClient, ActorIOInteractorServer, actor_io_interactors};

use fastwebsockets::{handshake, WebSocket};

//use gtk_estate::corlib::MovableText;

use corlib::text::MovableText;

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

static EMPTY_URL_PROVIDED: &str = "Empty URL provided";

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
    web_socket: Option<WebSocket<TokioIo<Upgraded>>>, //Option<Arc<WebSocket<TokioIo<Upgraded>>>>,
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

            self.process_received_actor_input_message(val).await;
            
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

    async fn process_received_actor_input_message(&mut self, message: WebSocketActorInputMessage)
    {

        match message
        {

            WebSocketActorInputMessage::ConnectTo(url) =>
            {

                //Chjeck if a zero lenght string has been provided for the connection URL.

                if url.is_empty()
                {

                    self.receiver_input.output_sender().send(WebSocketActorOutputMessage::ClientMessage(WebSocketActorOutputClientMessage::ConnectionResult(MovableText::Str(EMPTY_URL_PROVIDED)))).await.unwrap();

                    return;

                }

                //Disconnet from current server.

                match self.web_socket.take()
                {

                    Some(ws) =>
                    {

                        //Make sure the stream gets shutdown correctly.

                        //let _ = ws.into_inner().shutdown().await;

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

                        //Do something with the connection response,

                        self.url = Some(url);

                        //Connected!

                        self.receiver_input.output_sender().send(WebSocketActorOutputMessage::ClientMessage(WebSocketActorOutputClientMessage::ConnectionResult(MovableText::Str(CONNECTION_SUCCEEDED)))).await.unwrap();



                    },
                    Err(err) =>
                    {

                        let err_string = err.to_string();

                        //Send Error message to the actor-client

                        self.receiver_input.output_sender().send(WebSocketActorOutputMessage::ClientMessage(WebSocketActorOutputClientMessage::ConnectionResult(MovableText::String(err_string)))).await.unwrap();

                    }

                }


            }

        }


    }

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

