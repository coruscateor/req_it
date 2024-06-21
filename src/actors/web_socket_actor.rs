use act_rs::{ActorFrontend, ActorState, HasInteractor, impl_mac_task_actor, DroppedIndicator, impl_default_on_enter_async, impl_default_on_exit_async, impl_default_on_enter_and_exit_async};

//impl_mac_runtime_task_actor

use act_rs::tokio::{RuntimeTaskActor, interactors::mpsc::{SenderInteractor, channel}};

use fastwebsockets::{handshake, WebSocket};
use tokio::{sync::mpsc::Receiver, runtime::Handle};

use super::{graphql_actor_message::*, WebSocketActorInputMessage};

use paste::paste;

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

    sender_input: SenderInteractor<Option<WebSocketActorInputMessage>>,
    reciver_input: Receiver<Option<WebSocketActorInputMessage>>,
    //request_client: Client,
    //prettyer: PrettyEr

}

impl WebSocketActorState
{

    pub fn new() -> Self
    {

        let (sender_input, reciver_input) = channel(50);

        //let request_client = Client::new();

        Self
        {

            sender_input,
            reciver_input,
            //request_client,
            //prettyer: PrettyEr::new()


        }

    }

    //Default on_enter_async and on_exit_async implementations.

    impl_default_on_enter_and_exit_async!();

    async fn run_async(&mut self, di: &DroppedIndicator) -> bool
    {

        if let Some(opt) = self.reciver_input.recv().await
        {

            if let Some(val) = opt
            {

                match val
                {

                    WebSocketActorInputMessage::ConnectTo(url) => //,has_started) =>
                    {

                        match self.connect_to_server(url).await
                        {
                            Ok(res) => 
                            {

                                //res.read_frame()

                            },
                            Err(err) =>
                            {

                                let err_string = err.to_string();

                                //Send Error message to the actor-client

                            }
                        }


                    }

                }

            }

        }

        di.not_dropped()

    }

    async fn connect_to_server(&mut self, url: String) -> Result<WebSocket<TokioIo<Upgraded>>>
    {

        let url_str = url.as_str();

        let connection_stream = TcpStream::connect(url_str).await?;

        let request = Request::builder() //Request::new(Empty::<Bytes>::new());
            .method("GET")
            .uri(url_str)
            .header("Host", url_str)
            .header(UPGRADE, "websocket")
            .header(CONNECTION, "upgrade")
            .header("Sec-WebSocket-Verion", "13")
            .body(Empty::<Bytes>::new())?;
        /*
        let mut hm = request.headers_mut();

        hm.append("Host", url_str);

        hm.append(UPGRADE, "websocket");

        hm.append(CONNECTION, "upgrade");

        hm.append("Sec-WebSocket-Verion", "13");
        */

        let (ws, _) = handshake::client(&SpawnExecutor, request, connection_stream).await?;

        Ok(ws)

    }

    fn check_send_error(send_res: Result<(), GraphQLRequestResult>)
    {

        if let Err(_err) = send_res
        {

            println!("GraphQLMessage Send Error");

        }

    }

}

impl HasInteractor<SenderInteractor<Option<WebSocketActorInputMessage>>> for WebSocketActorState
{

    fn interactor(&self) -> &SenderInteractor<Option<WebSocketActorInputMessage>>
    {
       
       &self.sender_input

    }

}

//Setup the macro generated Task actor.

impl_mac_task_actor!(WebSocketActorState, SenderInteractor<Option<WebSocketActorInputMessage>>, WebSocketActor);
