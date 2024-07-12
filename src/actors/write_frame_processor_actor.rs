use std::sync::{Arc, Mutex};

use act_rs::{impl_default_start_and_end_async, impl_default_start_async, impl_default_end_async, impl_mac_task_actor, tokio::io::mpsc::ActorIOClient, ActorFrontend};

use corlib::{impl_get_ref};

use fastwebsockets::{OpCode, Payload};

use tokio::sync::mpsc::{Sender, Receiver, channel};

use tokio::task::JoinHandle;

use paste::paste;

use super::{ReadFrameProcessorActorInputMessage, ReadFrameProcessorActorOutputMessage, ReadFrameProcessorActorState, WebSocketActor, WebSocketActorInputMessage, WebSocketActorOutputMessage, WebSocketActorState, WriteFrameProcessorActorInputMessage};

use super::OwnedFrame;

#[derive(Clone)]
pub struct WriteFrameProcessorActorIOClient
{

    sender: Sender<WriteFrameProcessorActorInputMessage>,
    web_socket_actor_io_client: ActorIOClient<WebSocketActorInputMessage, WebSocketActorOutputMessage>,
    //read_frame_proccessor_output_receiver: Arc<Mutex<Receiver<ReadFrameProcessorActorOutputMessage>>>
    read_frame_actor_io_client: ActorIOClient<ReadFrameProcessorActorInputMessage, ReadFrameProcessorActorOutputMessage>

}

impl WriteFrameProcessorActorIOClient
{

    pub fn new(sender: Sender<WriteFrameProcessorActorInputMessage>, web_socket_actor_io_client: ActorIOClient<WebSocketActorInputMessage, WebSocketActorOutputMessage>, read_frame_actor_io_client: ActorIOClient<ReadFrameProcessorActorInputMessage, ReadFrameProcessorActorOutputMessage>) -> Self //read_frame_proccessor_output_receiver: Arc<Mutex<Receiver<ReadFrameProcessorActorOutputMessage>>>) -> Self //Receiver<ReadFrameProcessorActorOutputMessage>) -> Self
    {

        Self
        {

            sender,
            web_socket_actor_io_client,
            //read_frame_proccessor_output_receiver //: Arc::new(Mutex::new(read_frame_proccessor_output_receiver))
            read_frame_actor_io_client

        }

    }

    impl_get_ref!(sender, Sender<WriteFrameProcessorActorInputMessage>);

    impl_get_ref!(web_socket_actor_io_client, ActorIOClient<WebSocketActorInputMessage, WebSocketActorOutputMessage>);

    //impl_get_ref!(read_frame_proccessor_output_receiver, Arc<Mutex<Receiver<ReadFrameProcessorActorOutputMessage>>>);

    impl_get_ref!(read_frame_actor_io_client, ActorIOClient<ReadFrameProcessorActorInputMessage, ReadFrameProcessorActorOutputMessage>);

    /*
    pub fn read_frame_proccessor_output_receiver(&self) -> &Mutex<Receiver<ReadFrameProcessorActorOutputMessage>>
    {

        &self.read_frame_proccessor_output_receiver

    }
    */


}

pub struct WriteFrameProcessorActorState
{

    //input_sender: Sender<WriteFrameProcessorActorInputMessage>,
    input_receiver: Receiver<WriteFrameProcessorActorInputMessage>, 
    //client_sender_ouput: Sender<WebSocketActorOutputMessage>
    web_socket_actor_io_client: ActorIOClient<WebSocketActorInputMessage, WebSocketActorOutputMessage> //WebSocketActor,
    //actor_io_client: WriteFrameProcessorActorIOClent

}

impl WriteFrameProcessorActorState
{

    pub fn new() -> (WriteFrameProcessorActorIOClient, Self) //client_sender_ouput: Sender<WebSocketActorOutputMessage>) -> Self
    {

        let (sender, receiver) = channel(1000);

        //let (read_frame_proccessor_input_sender, read_frame_proccessor_input_receiver) = channel(1000);

        //The ReadFrameProcessorActor ou

        //let (read_frame_proccessor_input_sender, read_frame_proccessor_input_receiver) = channel(1000);

        let read_frame_processor_actor_io_client = ReadFrameProcessorActorState::spawn(); //read_frame_proccessor_input_receiver));

        //let read_frame_proccessor_output_receiver = read_frame_processor_actor.interactor().clone();

        //let read_frame_processor_actor_interactor = read_frame_processor_actor.interactor().clone();

        let web_socket_actor_io_client = WebSocketActorState::spawn(read_frame_processor_actor_io_client.clone()); //WebSocketActor::new(WebSocketActorState::new(read_frame_processor_actor)); //, read_frame_proccessor_input_sender));

        let actor_io_client = WriteFrameProcessorActorIOClient::new(sender, web_socket_actor_io_client.clone(), read_frame_processor_actor_io_client); //read_frame_processor_actor_interactor); //read_frame_proccessor_output_receiver);

        (actor_io_client,
        Self
        {

            //input_sender: sender,
            input_receiver: receiver,
            //client_sender_ouput
            web_socket_actor_io_client,
            //actor_io_client

        })

    }

    pub fn spawn() -> WriteFrameProcessorActorIOClient
    {

        let (actor_io_client, state) = WriteFrameProcessorActorState::new();

        WriteFrameProcessorActor::spawn(state);

        actor_io_client

    }

    impl_default_start_and_end_async!();

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

                        //Set the payload of the OwnedFrame the right size.

                        let content_bytes = contents.as_bytes();

                        let payload = &mut of.payload;
                        
                        let cb_len = content_bytes.len();

                        if cb_len != payload.len()
                        {

                            payload.resize(cb_len, 0);

                        }

                        //Copy the bytes into the OwnedFrame payload. 

                        payload.copy_from_slice(content_bytes);

                        if let Err(_) = self.web_socket_actor_io_client.input_sender().send(WebSocketActorInputMessage::WriteFrame(of)).await
                        {

                            return false;

                        }

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

/*

future cannot be sent between threads safely
within `{async block@/run/media/paul/Main Stuff/SoftwareProjects/Rust/act_rs/src/tokio/mac_task_actor.rs:56:30: 60:18}`, the trait `Send` is not implemented for `std::rc::Rc<std::sync::Mutex<tokio::sync::mpsc::Receiver<web_socket_actor_messages::ReadFrameProcessorActorOutputMessage>>>`, which is required by `{async block@/run/media/paul/Main Stuff/SoftwareProjects/Rust/act_rs/src/tokio/mac_task_actor.rs:56:30: 60:18}: Send`

 */

impl_mac_task_actor!(WriteFrameProcessorActor);
