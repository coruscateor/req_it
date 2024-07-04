use act_rs::{impl_default_on_enter_and_exit_async, impl_default_on_enter_async, impl_default_on_exit_async,  tokio::interactors::mpsc::ActorIOInteractorClient, DroppedIndicator, HasInteractor};

use tokio::sync::mpsc::{Sender, Receiver, channel};

use super::{FrameProcessorActorMessage, WebSocketActorOutputMessage};

pub struct FrameProcessorActorState
{

    input_sender: Sender<FrameProcessorActorMessage>,
    input_receiver: Receiver<FrameProcessorActorMessage>, 
    client_sender_ouput: Sender<WebSocketActorOutputMessage>

}

impl FrameProcessorActorState
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

impl HasInteractor<Sender<FrameProcessorActorMessage>> for FrameProcessorActorState
{

    fn interactor(&self) -> &Sender<FrameProcessorActorMessage>
    {
       
       &self.input_sender

    }

}
