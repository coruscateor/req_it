use act_rs::{impl_default_on_enter_and_exit_async, impl_default_on_enter_async, impl_default_on_exit_async,  tokio::interactors::mpsc::ActorIOInteractorClient, DroppedIndicator, HasInteractor};

use tokio::sync::mpsc::{Sender, Receiver, channel};

use super::{WriteFrameProcessorActorInputMessage, WebSocketActorOutputMessage};

pub struct WriteFrameProcessorActorState
{

    input_sender: Sender<WriteFrameProcessorActorInputMessage>,
    input_receiver: Receiver<WriteFrameProcessorActorInputMessage>, 
    client_sender_ouput: Sender<WebSocketActorOutputMessage>

}

impl WriteFrameProcessorActorState
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

impl HasInteractor<Sender<WriteFrameProcessorActorInputMessage>> for WriteFrameProcessorActorState
{

    fn interactor(&self) -> &Sender<WriteFrameProcessorActorInputMessage>
    {
       
       &self.input_sender

    }

}
