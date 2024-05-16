use act_rs::{ActorFrontend, ActorState, HasInteractor, impl_mac_runtime_task_actor, DroppedIndicator, impl_default_on_enter_async, impl_default_on_exit_async, impl_default_on_enter_and_exit_async};

use act_rs::tokio::{RuntimeTaskActor, interactors::mpsc::{SenderInteractor, channel}};

use tokio::{sync::mpsc::Receiver, runtime::Handle};

use super::graphql_actor_message::*;

use paste::paste;

use std::{marker::PhantomData, sync::Arc};

use tokio::runtime::Runtime;

use act_rs::ActorInteractor;

use reqwest::{self, Client, header::{HeaderMap, HeaderValue}};

use std::collections::HashMap;

use std::time::{Duration, Instant};

use pretty_goodness::json::PrettyEr;

use serde_json::{Value, Map};

pub struct GraphQLActorState
{

    sender: SenderInteractor<Option<GraphQLPostMessage>>,
    reciver: Receiver<Option<GraphQLPostMessage>>,
    request_client: Client,
    prettyer: PrettyEr

}

impl GraphQLActorState
{

    pub fn new() -> Self
    {

        let (sender, reciver) = channel(5);

        let request_client = Client::new();

        Self
        {

            sender,
            reciver,
            request_client,
            prettyer: PrettyEr::new()


        }

    }

    //Default on_enter_async and on_exit_async implementations.

    impl_default_on_enter_and_exit_async!();

    async fn run_async(&mut self, di: &DroppedIndicator) -> bool
    {

        if let Some(opt) = self.reciver.recv().await
        {

            if let Some(val) = opt
            {

                match val
                {

                    GraphQLPostMessage::Request(params, sender) => 
                    {

                        //Setup Request
                        
                        //Headers

                        let mut headers = HeaderMap::new();

                        //'Content-Type': 'application/json',
                        //'Accept': 'application/json',

                        let application_json = HeaderValue::from_str("application/json").unwrap();

                        headers.append("Content-Type", application_json.clone());

                        headers.append("Accept", application_json);

                        //Accept-Encoding: gzip

                        //https://graphql.org/learn/best-practices/#json-with-gzip

                        //Body

                        let mut json_body_map = Map::new();

                        json_body_map.insert("query".to_string(), Value::String(params.query));

                        if params.query_variables.len() > 0
                        {

                            let variables_from_str = serde_json::from_str(params.query_variables.as_str());
                            
                            match variables_from_str
                            {
                                
                                Ok(res) =>
                                {

                                    match &res
                                    {

                                        Value::Object(_) => { /*Success */ },
                                        _ =>
                                        {

                                            //Error - Not an object

                                            Self::check_send_error(sender.send(GraphQLRequestResult::new("Error: Parameters not provided in the format of an object".to_string(), Duration::default())));

                                            return di.not_dropped();

                                        }

                                    }

                                    json_body_map.insert("variables".to_string(), res);

                                }
                                Err(err) => 
                                {

                                    Self::check_send_error(sender.send(GraphQLRequestResult::new(err.to_string(), Duration::default())));

                                    return di.not_dropped();

                                }

                            }

                        }

                        let json_body = Value::Object(json_body_map);

                        let reqb = self.request_client.post(params.address).json(&json_body).headers(headers);

                        let req_res = reqb.build();

                        let req;

                        match req_res
                        {

                            Ok(res) => req = res,
                            Err(err) =>
                            {
                                
                                Self::check_send_error(sender.send(GraphQLRequestResult::new(err.to_string(), Duration::default())));

                                return di.not_dropped();

                            }
                            
                        }

                        //Get the current instant

                        let req_start = Instant::now();

                        //Send the request

                        //Measuring only the request and response times.

                        let res = self.request_client.execute(req).await;

                        //Get the elapsed time since the start of sending the request to the receipt of the response.

                        let elapsed = req_start.elapsed();

                        match res
                        {

                            Ok(ok_res) =>
                            {

                                let text_res = ok_res.text().await;

                                match text_res
                                {

                                    Ok(text_ok_res) =>
                                    {

                                        //Prettafy the produced text

                                        let pretty_res = self.prettyer.prettyafy(&text_ok_res);

                                        match pretty_res
                                        {

                                            Ok(res) => 
                                            {

                                                Self::check_send_error(sender.send(GraphQLRequestResult::new(res, elapsed)))

                                            },
                                            Err(err) =>
                                            {

                                                //Include source on error? - Yes

                                                let ugly_len = text_ok_res.len();

                                                let err_string = err.to_string();

                                                let mut full_err_string = String::with_capacity(ugly_len + err_string.len() + 2);

                                                full_err_string.push_str(text_ok_res.as_str());

                                                full_err_string.push_str("\n\n");

                                                full_err_string.push_str(err_string.as_str());

                                                Self::check_send_error(sender.send(GraphQLRequestResult::new(full_err_string, elapsed)))

                                            }

                                        }                                        

                                    },
                                    Err(text_err_res) =>
                                    {

                                        Self::check_send_error(sender.send(GraphQLRequestResult::new(text_err_res.to_string(), elapsed)));

                                    }
                                }

                            },
                            Err(err_res) =>
                            {

                                Self::check_send_error(sender.send(GraphQLRequestResult::new(err_res.to_string(), elapsed)));

                            }

                        }

                    },

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

impl HasInteractor<SenderInteractor<Option<GraphQLPostMessage>>> for GraphQLActorState
{

    fn interactor(&self) -> &SenderInteractor<Option<GraphQLPostMessage>>
    {
       
       &self.sender

    }

}

//Setup the macro generated runtime task actor.  

impl_mac_runtime_task_actor!(GraphQLActorState, SenderInteractor<Option<GraphQLPostMessage>>, GraphQLRuntimeActor);
