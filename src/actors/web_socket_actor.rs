use act_rs::{ActorFrontend, ActorState, HasInteractor, impl_mac_task_actor, DroppedIndicator, impl_default_on_enter_async, impl_default_on_exit_async, impl_default_on_enter_and_exit_async};

//impl_mac_runtime_task_actor

use act_rs::tokio::{RuntimeTaskActor, interactors::mpsc::{SenderInteractor, channel}};

use tokio::{sync::mpsc::Receiver, runtime::Handle};

use super::{graphql_actor_message::*, WebSocketActorInputMessage};

use paste::paste;

use std::{marker::PhantomData, sync::Arc};

use tokio::runtime::Runtime;

use act_rs::ActorInteractor;

//use reqwest::{self, Client, header::{HeaderMap, HeaderValue}};

use std::collections::HashMap;

use std::time::{Duration, Instant};

use pretty_goodness::json::PrettyEr;

use serde_json::{Value, Map};



pub struct WebSocketActorState
{

    sender_input: SenderInteractor<Option<WebSocketActorInputMessage>>,
    reciver_input: Receiver<Option<WebSocketActorInputMessage>>,
    //request_client: Client,
    prettyer: PrettyEr

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
            prettyer: PrettyEr::new()


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

                    WebSocketActorInputMessage::ConnectTo(url,has_started) =>
                    {



                    }

                }

            }

        }

        di.not_dropped()

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
