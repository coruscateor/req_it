use act_rs::{impl_default_start_and_end_async, impl_default_start_async, impl_default_end_async, impl_mac_task_actor, tokio::io::mpsc::{ActorIOClient, ActorIOServer, actor_io}};

use fastwebsockets::OpCode;

use tokio::sync::mpsc::{Sender, Receiver, channel};

use super::{ReadFrameProcessorActorInputMessage, ReadFrameProcessorActorOutputMessage, WebSocketActorOutputMessage};

use std::sync::{Arc, Mutex};

use act_rs::ActorFrontend;

use tokio::task::JoinHandle;

use paste::paste;

//super::OwnedFrame;

pub struct ReadFrameProcessorActorState
{

    //input_sender: Sender<ReadFrameProcessorActorInputMessage>,
    /*
    input_receiver: Receiver<ReadFrameProcessorActorInputMessage>, 
    ouput_sender: Sender<ReadFrameProcessorActorOutputMessage>,
    ouput_receiver: Arc<Mutex<Receiver<ReadFrameProcessorActorOutputMessage>>>
    */

    //io_client: ActorIOInteractorClient<ReadFrameProcessorActorInputMessage, ReadFrameProcessorActorOutputMessage>, //Should really only be on the "client side".
    io_server: ActorIOServer<ReadFrameProcessorActorInputMessage, ReadFrameProcessorActorOutputMessage>

}

impl ReadFrameProcessorActorState
{

    pub fn new() -> (ActorIOClient<ReadFrameProcessorActorInputMessage, ReadFrameProcessorActorOutputMessage>, Self) //input_receiver: Receiver<ReadFrameProcessorActorInputMessage>) -> Self
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

        let (io_client, io_server) = actor_io(1000, 1000);

        (io_client,
        Self
        {

            //io_client,
            io_server

        })

    }

    pub fn spawn() -> ActorIOClient<ReadFrameProcessorActorInputMessage, ReadFrameProcessorActorOutputMessage>
    {

        let (io_client, state) = ReadFrameProcessorActorState::new();

        ReadFrameProcessorActor::spawn(state);

        io_client

    }

    impl_default_start_and_end_async!();

    //impl_default_on_enter_async!();

    async fn run_async(&mut self) -> bool
    {

        if let Some(message) = self.io_server.input_receiver().recv().await //.input_receiver.recv().await
        {

            let output;

            match message
            {

                ReadFrameProcessorActorInputMessage::Frame(frame) =>
                {

                    if frame.opcode == OpCode::Text
                    {
        
                        //let opcode = format!("{}", frame.opcode);

                        let payload_as_utf8 = String::from_utf8_lossy(&frame.payload);

                        output = format!("{{fin: {fin},\nopcode: {opcode:?},\npayload_as_utf8: {payload_as_utf8}}}", fin = frame.fin, opcode = frame.opcode)
        
                    }
                    else if frame.opcode == OpCode::Binary || frame.opcode == OpCode::Continuation
                    {

                        output = format!("{{fin: {fin},\nopcode: {opcode:?},\npayload {payload:?}}}", fin = frame.fin, opcode = frame.opcode, payload = frame.payload)

                    }
                    else
                    {

                        output = format!("{{fin: {fin},\nopcode: {opcode:?},\npayload {payload:?}}}", fin = frame.fin, opcode = frame.opcode, payload = frame.payload)
                        
                    }

                }
                
            }

            let _ = self.io_server.output_sender().send(ReadFrameProcessorActorOutputMessage::Processed(output)).await; //.unwrap();

            return true;

        }

        false

    }

}

//impl_mac_task_actor!(ReadFrameProcessorActorState, ReadFrameProcessorActor);

impl_mac_task_actor!(ReadFrameProcessorActor);
