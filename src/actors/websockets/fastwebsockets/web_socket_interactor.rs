use std::rc::Rc;

use tokio::sync::mpsc::{channel, Receiver, Sender};

use super::{WebSocketActorInputMessage, WebSocketActorOutputMessage};


pub struct WebSocketInteractorClient
{

    actor_input_sender: Sender<WebSocketActorInputMessage>,
    actor_output_receiver: Rc<Receiver<WebSocketActorOutputMessage>>,

}

impl WebSocketInteractorClient
{

    pub fn new(actor_input_sender: Sender<WebSocketActorInputMessage>, actor_output_receiver: Receiver<WebSocketActorOutputMessage>) -> Self
    {

        Self
        {

            actor_input_sender,
            actor_output_receiver: Rc::new(actor_output_receiver)

        }

    }

    pub fn input_sender(&self) -> &Sender<WebSocketActorInputMessage>
    {

        &self.actor_input_sender

    }

    pub fn output_receiver(&self) -> &Receiver<WebSocketActorOutputMessage>
    {

        &self.actor_output_receiver

    }

}

pub struct WebSocketInteractorServer
{

    actor_input_receiver: Receiver<WebSocketActorInputMessage>,
    actor_output_sender: Sender<WebSocketActorOutputMessage>,

}

impl WebSocketInteractorServer
{

    pub fn new(actor_input_receiver: Receiver<WebSocketActorInputMessage>, actor_output_sender: Sender<WebSocketActorOutputMessage>) -> Self
    {

        Self
        {

            actor_input_receiver,
            actor_output_sender

        }

    }

    pub fn input_receiver(&self) -> &Receiver<WebSocketActorInputMessage>
    {

        &self.actor_input_receiver

    }

    pub fn output_sender(&self) -> &Sender<WebSocketActorOutputMessage>
    {

        &self.actor_output_sender

    }

}


pub fn web_socket_interactor(input_buffer_size: usize, output_buffer_size: usize) -> (WebSocketInteractorClient, WebSocketInteractorServer)
{

    let (actor_input_sender,actor_input_receiver) = channel(input_buffer_size);

    let (actor_output_sender,actor_output_receiver) = channel(output_buffer_size);

    (WebSocketInteractorClient::new(actor_input_sender, actor_output_receiver), WebSocketInteractorServer::new(actor_input_receiver, actor_output_sender))

}
