//use act_rusty::{ActorFrontend, ActorState, HasInteractor, impl_mac_runtime_task_actor, DroppedIndicator, impl_default_on_enter_async, impl_default_on_exit_async, impl_default_on_enter_and_exit_async}; //impl_actor_frontend, //tokio_actors::RuntimeTaskFnActor,

use act_rs::{ActorFrontend, ActorState, HasInteractor, impl_mac_runtime_task_actor, DroppedIndicator, impl_default_on_enter_async, impl_default_on_exit_async, impl_default_on_enter_and_exit_async}; //impl_actor_frontend, //tokio_actors::RuntimeTaskFnActor,

//use act_rusty::tokio::{RuntimeTaskActor, interactors::mspc::{SenderInteractor, channel}};

use act_rs::tokio::{RuntimeTaskActor, interactors::mspc::{SenderInteractor, channel}};

//tokio_actors::RuntimeTaskActor,

//tokio_interactors::mspc::{SenderInteractor, channel}, 

use tokio::{sync::mpsc::Receiver, runtime::Handle};

use super::graphql_actor_message::*;

use paste::paste;

use std::{marker::PhantomData, sync::Arc};

//use async_trait::async_trait;

use tokio::runtime::Runtime;

//use act_rusty::ActorInteractor;

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

                        /*
                        let mut json_body = HashMap::new();

                        json_body.insert("query", params.query);

                        if params.query_variables.len() > 0
                        {

                            json_body.insert("variables", params.query_variables);

                        }
                        */

                        //let request_client = Client::new();

                        let mut json_body_map = Map::new();

                        json_body_map.insert("query".to_string(), Value::String(params.query));

                        if params.query_variables.len() > 0
                        {

                            //let variables_value;

                            //let variables_value_res = serde_json::to_value(params.query_variables);

                            let variables_from_str = serde_json::from_str(params.query_variables.as_str());
                            
                            match variables_from_str //variables_value_res
                            {
                                
                                Ok(res) => //variables_value = res,
                                {

                                    match &res
                                    {

                                        Value::Object(_) => { /*Success */ },
                                        _ =>
                                        {

                                            //Error - Not an object

                                            Self::check_send_error(sender.send(GraphQLRequestResult::new("Error: Parameters not provided in the format of an object".to_string(), Duration::default())));

                                            return di.has_not_dropped();

                                        }

                                    }

                                    json_body_map.insert("variables".to_string(), res);

                                }
                                Err(err) => 
                                {

                                    Self::check_send_error(sender.send(GraphQLRequestResult::new(err.to_string(), Duration::default())));

                                    return di.has_not_dropped();

                                }

                            }

                        }

                        let json_body = Value::Object(json_body_map);

                        let reqb = self.request_client.post(params.address).json(&json_body).headers(headers); //.body(params.query).header(key, value) //.data;

                        //reqb.headers(headers);

                        let req_res = reqb.build();

                        let req;

                        match req_res
                        {

                            Ok(res) => req = res,
                            Err(err) =>
                            {
                                
                                Self::check_send_error(sender.send(GraphQLRequestResult::new(err.to_string(), Duration::default())));

                                return di.has_not_dropped();

                            }
                            
                        }

                        //Get the current instant

                        let req_start = Instant::now();

                        //Send the request

                        //let res = reqb.send().await;

                        //Measure only the request and response

                        //println!("{}", req);

                        let res = self.request_client.execute(req).await;

                        //Get the elapsed time scince the start of sending the reqest to the reciipt of the response

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

                                                Self::check_send_error(sender.send(GraphQLRequestResult::new(full_err_string, elapsed))) //err.to_string(), elapsed)))

                                            }

                                        }

                                        //Self::check_send_error(sender.send(GraphQLRequestResult::new(text_ok_res, elapsed)));                                        

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

                        //res.

                        //let send_res = sender.send(GraphQLRequestResult::new("Success!".into(), 1.0));

                        //Self::check_send_error(sender.send(GraphQLRequestResult::new("Success!".into(), elapsed)));

                    },

                }

            }

        }

        //true

        di.has_not_dropped()

    }

    fn check_send_error(send_res: Result<(), GraphQLRequestResult>)
    {

        if let Err(_err) = send_res
        {

            println!("GraphQLMessage Send Error");
            
            //err.

        }

    }

}

impl HasInteractor<SenderInteractor<Option<GraphQLPostMessage>>> for GraphQLActorState
{

    fn get_interactor(&self) -> SenderInteractor<Option<GraphQLPostMessage>>
    {
       
       self.sender.clone()

    }

}

//impl_mac_runtime_task_actor!(SenderInteractor<Option<GraphQLPostMessage>>, GraphQLActorState, SenderInteractor_GraphQLActorState);

//pub type GraphQLRuntimeActor = MacRuntimeTaskActor_SenderInteractor_GraphQLActorState;

impl_mac_runtime_task_actor!(SenderInteractor<Option<GraphQLPostMessage>>, GraphQLActorState, GraphQLRuntimeActor);

/*
async fn default_fn_enter(_state: &mut GraphQLActorState, _di: &DroppedIndicator) -> bool
{

    true

}

async fn default_fn_exit(_state: &mut GraphQLActorState, _di: &DroppedIndicator)
{
}

async fn run(state: &mut GraphQLActorState, di: &DroppedIndicator) -> bool
{

    true

}
*/

/*
pub fn get_runtime_task_actor(handle: &Handle) -> GraphQLRuntimeActor
{

    let state = GraphQLActorState::new();

    GraphQLRuntimeActor::new(handle, state)

    /*

    Some value of type T.

    higher-ranked lifetime error
    could not prove `RuntimeTaskFnActor<GraphQLActorState, SenderInteractor<std::option::Option<graphql_actor_message::GraphQLMessage>>, impl Future<Output = bool>, for<'a, 'b> fn(&'a mut GraphQLActorState, &'b DroppedIndicator) -> impl Future<Output = bool> {default_fn_enter}, impl Future<Output = bool>, for<'a, 'b> fn(&'a mut GraphQLActorState, &'b DroppedIndicator) -> impl Future<Output = bool> {graphql_actor::run}, impl Future<Output = ()>, for<'a, 'b> fn(&'a mut GraphQLActorState, &'b DroppedIndicator) -> impl Future<Output = ()> {default_fn_exit}> well-formed`rustcClick for full compiler diagnostic

     */

    //let actor = RuntimeTaskFnActor::new(handle, state, default_fn_enter, run, default_fn_exit); //None, run, None);

    /*

    type annotations needed for `RuntimeTaskFnActor<GraphQLActorState, SenderInteractor<std::option::Option<graphql_actor_message::GraphQLMessage>>, FB, FNB, impl Future<Output = bool>, [closure@src/actors/graphql_actor.rs:54:62: 54:127], FE, FNE>`
    cannot satisfy `<_ as Future>::Output == bool`rustcClick for full compiler diagnostic
    graphql_actor.rs(54, 17): type must be known at this point
    runtime_task_fn_actor.rs(40, 16): required by a bound in `RuntimeTaskFnActor::<ST, IN, FB, FNB, FR, FNR, FE, FNE>::new`
    graphql_actor.rs(54, 14): consider giving `actor` an explicit type, where the type for type parameter `FB` is specified: `: RuntimeTaskFnActor<GraphQLActorState, SenderInteractor<std::option::Option<graphql_actor_message::GraphQLMessage>>, FB, FNB, impl Future<Output = bool>, [closure@src/actors/graphql_actor.rs:54:62: 54:127], FE, FNE>`

    */

    /*
    let actor = RuntimeTaskFnActor::new(handle, state, None, async move |state: &mut GraphQLActorState, di: &DroppedIndicator| {

        true

    }, None);
    */

}
*/

/*
error[E0195]: lifetime parameters or bounds on method `on_enter` do not match the trait declaration
  --> src/actors/graphql_actor.rs:48:14
   |
48 |     async fn on_enter(&mut self, _di: &act_rusty::DroppedIndicator) -> bool 
   |              ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ lifetimes do not match method in trait

error[E0195]: lifetime parameters or bounds on method `run` do not match the trait declaration
  --> src/actors/graphql_actor.rs:55:14
   |
55 |     async fn run(&mut self, di: &act_rusty::DroppedIndicator) -> bool
   |              ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ lifetimes do not match method in trait

error[E0195]: lifetime parameters or bounds on method `on_exit` do not match the trait declaration
  --> src/actors/graphql_actor.rs:69:14
   |
69 |     async fn on_exit(&mut self, _di: &act_rusty::DroppedIndicator)
   |              ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ lifetimes do not match method in trait
 */

/*
#[async_trait]
impl ActorState<SenderInteractor<Option<GraphQLMessage>>> for GraphQLActorState
{

    /*
    fn get_interactor(&self) -> SenderInteractor<Option<GraphQLMessage>>
    {
       
       self.sender.clone()

    }
    */

    //async fn on_enter(&mut self, _di: &act_rusty::DroppedIndicator) -> bool 
    //{
        
    //    true //proceed

    //}

    async fn run(&mut self, di: &act_rusty::DroppedIndicator) -> bool
    {

        if let Some(val) = self.reciver.recv().await
        {



        }
       
        di.has_not_dropped()

    }

    //async fn on_exit(&mut self, _di: &act_rusty::DroppedIndicator)
    //{
    //}

}
*/
