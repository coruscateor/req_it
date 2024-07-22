use std::sync::{atomic::AtomicUsize, Arc};

use act_rs::tokio::io::mpsc::ActorIOClient;

use tokio::sync::mpsc::{channel, Receiver, Sender};

use super::{ReadFrameProcessorActorInputMessage, ReadFrameProcessorActorOutputMessage, WebSocketActor, WebSocketActorInputMessage, WebSocketActorState};


pub struct WebSocketActorStateBuilder
{

    read_frame_processor_actor_io: ActorIOClient<ReadFrameProcessorActorInputMessage,ReadFrameProcessorActorOutputMessage>,
    in_the_read_pipeline_count: Arc<AtomicUsize>,
    input_receiver: Receiver<WebSocketActorInputMessage>

}

impl WebSocketActorStateBuilder
{

    pub fn new(read_frame_processor_actor_io: ActorIOClient<ReadFrameProcessorActorInputMessage, ReadFrameProcessorActorOutputMessage>, in_the_read_pipeline_count: &Arc<AtomicUsize>, input_receiver: Receiver<WebSocketActorInputMessage>) -> Self
    {

        Self
        {

            read_frame_processor_actor_io,
            in_the_read_pipeline_count: in_the_read_pipeline_count.clone(),
            input_receiver

        }


    }

    pub fn spawn(read_frame_processor_actor_io: ActorIOClient<ReadFrameProcessorActorInputMessage, ReadFrameProcessorActorOutputMessage>, in_the_read_pipeline_count: &Arc<AtomicUsize>) -> Sender<WebSocketActorInputMessage> //ActorIOClient<WebSocketActorInputMessage, WebSocketActorOutputMessage>
    {

        let (input_sender, input_receiver) = channel(50);

        let state_builder = WebSocketActorStateBuilder::new(read_frame_processor_actor_io, &in_the_read_pipeline_count, input_receiver);

        WebSocketActor::spawn(state_builder);

        input_sender

    }

    pub async fn build_async(self) -> (bool, WebSocketActorState)
    {

        (true, WebSocketActorState::new(self.read_frame_processor_actor_io, self.in_the_read_pipeline_count, self.input_receiver))

    }

}