//use act_rusty::tokio_oneshot::Sender;

//use act_rusty::tokio::oneshot::Sender;

use tokio::sync::oneshot::Sender;

use std::time::{Duration, Instant};

use paste::paste;

//use corlib::{impl_get_ref, impl_get};

use gtk_estate::corlib::{impl_get_ref, impl_get};

pub struct GraphQLPostRequestParams
{

    pub address: String,
    pub query: String,
    pub query_variables: String,
    pub http_headers: String

}

impl GraphQLPostRequestParams
{

    pub fn new(address: String, query: String, query_variables: String, http_headers: String) -> Self
    {

        Self
        {

            address,
            query,
            query_variables,
            http_headers

        }

    }

}

pub struct GraphQLRequestResult
{

    result: String,
    duration: Duration

}

impl GraphQLRequestResult
{

    pub fn new(result: String, duration: Duration) -> Self
    {

        Self
        {

            result,
            duration

        }

    }

    impl_get_ref!(result, String);

    impl_get!(duration, Duration);

}

pub enum GraphQLPostMessage
{

    Request(GraphQLPostRequestParams, Sender<GraphQLRequestResult>)

}



