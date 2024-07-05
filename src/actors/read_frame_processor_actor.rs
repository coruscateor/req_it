use act_rs::{impl_default_on_enter_and_exit_async, impl_default_on_enter_async, impl_default_on_exit_async, impl_mac_task_actor, tokio::interactors::mpsc::ActorIOInteractorClient, DroppedIndicator, HasInteractor};

use tokio::sync::mpsc::{Sender, Receiver, channel};

use super::{ReadFrameProcessorActorInputMessage, WebSocketActorOutputMessage};

use std::sync::Arc;

use act_rs::ActorFrontend;

pub struct ReadFrameProcessorActorState
{

    input_sender: Sender<ReadFrameProcessorActorInputMessage>,
    input_receiver: Receiver<ReadFrameProcessorActorInputMessage>, 
    client_sender_ouput: Sender<WebSocketActorOutputMessage>

}

impl ReadFrameProcessorActorState
{

    pub fn new(client_sender_ouput: Sender<WebSocketActorOutputMessage>) -> Self
    {

        let (sender, receiver) = channel(1000);

        Self
        {

            input_sender: sender,
            input_receiver: receiver,
            client_sender_ouput

        }

    }

    impl_default_on_enter_and_exit_async!();

    //impl_default_on_enter_async!();

}

impl HasInteractor<Sender<ReadFrameProcessorActorInputMessage>> for ReadFrameProcessorActorState
{

    fn interactor(&self) -> &Sender<ReadFrameProcessorActorInputMessage>
    {
       
       &self.input_sender

    }

}

impl_mac_task_actor!(ReadFrameProcessorActorState, ActorIOInteractorClient<ReadFrameProcessorActorInputMessage, ReadFrameProcessorActorOutputMessage>, ReadFrameProcessorActor);