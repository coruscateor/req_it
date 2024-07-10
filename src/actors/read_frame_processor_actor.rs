use act_rs::{impl_default_on_enter_and_exit_async, impl_default_on_enter_async, impl_default_on_exit_async, impl_mac_task_actor, tokio::interactors::mpsc::{ActorIOInteractorClient, ActorIOInteractorServer, actor_io_interactors}, DroppedIndicator, HasInteractor};

use tokio::sync::mpsc::{Sender, Receiver, channel};

use super::{ReadFrameProcessorActorInputMessage, ReadFrameProcessorActorOutputMessage, WebSocketActorOutputMessage};

use std::sync::{Arc, Mutex};

use act_rs::ActorFrontend;

pub struct ReadFrameProcessorActorState
{

    //input_sender: Sender<ReadFrameProcessorActorInputMessage>,
    /*
    input_receiver: Receiver<ReadFrameProcessorActorInputMessage>, 
    ouput_sender: Sender<ReadFrameProcessorActorOutputMessage>,
    ouput_receiver: Arc<Mutex<Receiver<ReadFrameProcessorActorOutputMessage>>>
    */

    io_client: ActorIOInteractorClient<ReadFrameProcessorActorInputMessage, ReadFrameProcessorActorOutputMessage>, //Should really only be on the "client side".
    io_server: ActorIOInteractorServer<ReadFrameProcessorActorInputMessage, ReadFrameProcessorActorOutputMessage>

}

impl ReadFrameProcessorActorState
{

    pub fn new() -> Self //input_receiver: Receiver<ReadFrameProcessorActorInputMessage>) -> Self
    {

        /*
        let (ouput_sender, ouput_receiver) = channel(1000);

        Self
        {

            input_receiver,
            ouput_sender,
            ouput_receiver: Arc::new(Mutex::new(ouput_receiver))

        }
        */

        let (io_client, io_server) = actor_io_interactors(1000, 1000);

        Self
        {

            io_client,
            io_server

        }

    }

    impl_default_on_enter_and_exit_async!();

    //impl_default_on_enter_async!();

    async fn run_async(&mut self) -> bool
    {

        if let Some(message) = self.io_server.input_receiver().recv().await //.input_receiver.recv().await
        {



            return true;

        }

        false

    }

}

impl HasInteractor<ActorIOInteractorClient<ReadFrameProcessorActorInputMessage, ReadFrameProcessorActorOutputMessage>> for ReadFrameProcessorActorState
{

    fn interactor(&self) -> &ActorIOInteractorClient<ReadFrameProcessorActorInputMessage, ReadFrameProcessorActorOutputMessage> //&Arc<Mutex<Receiver<ReadFrameProcessorActorOutputMessage>>>
    {
       
       &self.io_client //ouput_receiver

    }

}

impl_mac_task_actor!(ReadFrameProcessorActorState, ActorIOInteractorClient<ReadFrameProcessorActorInputMessage, ReadFrameProcessorActorOutputMessage>, ReadFrameProcessorActor);
