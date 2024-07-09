use std::rc::Rc;
use std::sync::Mutex;

use act_rs::{impl_default_on_enter_and_exit_async, impl_default_on_enter_async, impl_default_on_exit_async, impl_mac_task_actor, tokio::interactors::mpsc::ActorIOInteractorClient, ActorFrontend, DroppedIndicator, HasInteractor};

use corlib::{impl_get_ref};

use fastwebsockets::{OpCode, Payload};
use tokio::sync::mpsc::{Sender, Receiver, channel};

use super::{ReadFrameProcessorActor, ReadFrameProcessorActorOutputMessage, ReadFrameProcessorActorState, WebSocketActor, WebSocketActorInputMessage, WebSocketActorOutputMessage, WebSocketActorState, WriteFrameProcessorActorInputMessage};

use super::OwnedFrame;

#[derive(Clone)]
pub struct WriteFrameProcessorActorInteractor
{

    sender: Sender<WriteFrameProcessorActorInputMessage>,
    web_socket_actor_interactor: ActorIOInteractorClient<WebSocketActorInputMessage, WebSocketActorOutputMessage>,
    read_frame_proccessor_output_receiver: Rc<Mutex<Receiver<ReadFrameProcessorActorOutputMessage>>>

}

impl WriteFrameProcessorActorInteractor
{

    pub fn new(sender: Sender<WriteFrameProcessorActorInputMessage>, web_socket_actor_interactor: ActorIOInteractorClient<WebSocketActorInputMessage, WebSocketActorOutputMessage>, read_frame_proccessor_output_receiver: Receiver<ReadFrameProcessorActorOutputMessage>) -> Self
    {

        Self
        {

            sender,
            web_socket_actor_interactor,
            read_frame_proccessor_output_receiver: Rc::new(Mutex::new(read_frame_proccessor_output_receiver))

        }

    }

    impl_get_ref!(sender, Sender<WriteFrameProcessorActorInputMessage>);

    impl_get_ref!(web_socket_actor_interactor, ActorIOInteractorClient<WebSocketActorInputMessage, WebSocketActorOutputMessage>);

    //impl_get_ref!(read_frame_proccessor_output_receiver, Rc<Mutex<Receiver<ReadFrameProcessorActorOutputMessage>>>);

    pub fn read_frame_proccessor_output_receiver(&self) -> &Mutex<Receiver<ReadFrameProcessorActorOutputMessage>>
    {

        &self.read_frame_proccessor_output_receiver

    }


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

        //let (read_frame_proccessor_input_sender, read_frame_proccessor_input_receiver) = channel(1000);

        let (read_frame_proccessor_output_sender, read_frame_proccessor_output_receiver) = channel(1000);

        let read_frame_processor_actor = ReadFrameProcessorActor::new(ReadFrameProcessorActorState::new(read_frame_proccessor_output_sender));

        let web_socket_actor = WebSocketActor::new(WebSocketActorState::new(read_frame_processor_actor));

        let interactor = WriteFrameProcessorActorInteractor::new(sender, web_socket_actor.interactor().clone(), read_frame_proccessor_output_receiver);

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

                        //Get from cache...

                        let mut of = OwnedFrame::new();

                        of.opcode = OpCode::Text;

                        let content_bytes = contents.as_bytes();

                        let payload = &mut of.payload;
                        
                        let cb_len = content_bytes.len();

                        if cb_len != payload.len()
                        {

                            payload.resize(cb_len, 0);

                        }

                        payload.copy_from_slice(content_bytes);

                        self.web_socket_actor.interactor().input_sender().send(WebSocketActorInputMessage::WriteFrame(of));

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
