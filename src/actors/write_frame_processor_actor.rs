use act_rs::{impl_default_on_enter_and_exit_async, impl_default_on_enter_async, impl_default_on_exit_async, impl_mac_task_actor, tokio::interactors::mpsc::ActorIOInteractorClient, ActorFrontend, DroppedIndicator, HasInteractor};

use corlib::{impl_get_ref};

use tokio::sync::mpsc::{Sender, Receiver, channel};

use super::{WebSocketActor, WebSocketActorInputMessage, WebSocketActorOutputMessage, WebSocketActorState, WriteFrameProcessorActorInputMessage};

#[derive(Clone)]
pub struct WriteFrameProcessorActorInteractor
{

    sender: Sender<WriteFrameProcessorActorInputMessage>,
    web_socket_actor_interactor: ActorIOInteractorClient<WebSocketActorInputMessage, WebSocketActorOutputMessage>

}

impl WriteFrameProcessorActorInteractor
{

    pub fn new(sender: Sender<WriteFrameProcessorActorInputMessage>, web_socket_actor_interactor: ActorIOInteractorClient<WebSocketActorInputMessage, WebSocketActorOutputMessage>) -> Self
    {

        Self
        {

            sender,
            web_socket_actor_interactor

        }

    }

    impl_get_ref!(sender, Sender<WriteFrameProcessorActorInputMessage>);

    impl_get_ref!(web_socket_actor_interactor, ActorIOInteractorClient<WebSocketActorInputMessage, WebSocketActorOutputMessage>);

}

pub struct WriteFrameProcessorActorState
{

    //input_sender: Sender<WriteFrameProcessorActorInputMessage>,
    input_receiver: Receiver<WriteFrameProcessorActorInputMessage>, 
    //client_sender_ouput: Sender<WebSocketActorOutputMessage>
    web_socket_actor: WebSocketActor,
    interactor: WriteFrameProcessorActorInteractor
}

impl WriteFrameProcessorActorState
{

    pub fn new() -> Self //client_sender_ouput: Sender<WebSocketActorOutputMessage>) -> Self
    {

        let (sender, receiver) = channel(1000);

        let web_socket_actor = WebSocketActor::new(WebSocketActorState::new());

        let interactor = WriteFrameProcessorActorInteractor::new(sender, web_socket_actor.interactor().clone());

        Self
        {

            //input_sender: sender,
            input_receiver: receiver,
            //client_sender_ouput
            web_socket_actor,
            interactor

        }

    }

    impl_default_on_enter_and_exit_async!();

    //impl_default_on_enter_async!();

    async fn run_async(&mut self) -> bool
    {

        match self.input_receiver.recv().await
        {

            Some(message) =>
            {

                match message
                {

                    WriteFrameProcessorActorInputMessage::Process(contents) =>
                    {

                        

                    },

                }

            },
            None =>
            {

                return false;

            }
        }

        true

    }

}

impl HasInteractor<WriteFrameProcessorActorInteractor> for WriteFrameProcessorActorState
{

    fn interactor(&self) -> &WriteFrameProcessorActorInteractor
    {
       
       &self.interactor //input_sender

    }

}

impl_mac_task_actor!(WriteFrameProcessorActorState, WriteFrameProcessorActorInteractor, WriteFrameProcessorActor);
