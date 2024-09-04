use act_rs::{impl_default_start_and_end_async, impl_default_start_async, impl_default_end_async, impl_mac_task_actor, tokio::io::mpsc::{ActorIOClient, ActorIOServer, actor_io}};

use fastwebsockets::OpCode;

use tokio::sync::mpsc::{Sender, Receiver, channel};

use super::{OwnedFrame, ReadFrameProcessorActorInputMessage, ReadFrameProcessorActorOutputMessage, WebSocketActorOutputClientMessage}; //pipeline_message_counter::Decrementor,  //, WebSocketActorOutputMessage};

use std::{borrow::Cow, rc::Rc, sync::{atomic::{AtomicUsize, Ordering}, Arc, Mutex}};

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
    io_server: ActorIOServer<ReadFrameProcessorActorInputMessage, ReadFrameProcessorActorOutputMessage>,
    //in_the_read_pipeline_count: Decrementor //Arc<AtomicUsize>

}

impl ReadFrameProcessorActorState
{

    pub fn new() -> (ActorIOClient<ReadFrameProcessorActorInputMessage, ReadFrameProcessorActorOutputMessage>, Self) //in_the_read_pipeline_count: Decrementor //&Arc<AtomicUsize> //input_receiver: Receiver<ReadFrameProcessorActorInputMessage>) -> Self
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
            io_server,
            //in_the_read_pipeline_count //: in_the_read_pipeline_count.clone()

        })

    }

    pub fn spawn() -> ActorIOClient<ReadFrameProcessorActorInputMessage, ReadFrameProcessorActorOutputMessage> //in_the_read_pipeline_count: Decrementor //&Arc<AtomicUsize>
    {

        let (io_client, state) = ReadFrameProcessorActorState::new(); //in_the_read_pipeline_count);

        ReadFrameProcessorActor::spawn(state);

        io_client

    }

    impl_default_start_and_end_async!();

    //impl_default_on_enter_async!();

    async fn run_async(&mut self) -> bool
    {

        if let Some(message) = self.io_server.input_receiver().recv().await //.input_receiver.recv().await
        {

            //let output;

            match message
            {

                ReadFrameProcessorActorInputMessage::Frame(frame) =>
                {

                    let output = Self::format_output(frame);

                    if let Err(_err) = self.io_server.output_sender().send(ReadFrameProcessorActorOutputMessage::Processed(output)).await
                    {

                        return false;

                    }

                    //.fetch_sub(1, Ordering::SeqCst);

                }
                ReadFrameProcessorActorInputMessage::ClientMessage(message) =>
                {

                    //Client messages get passed through to the UI.
                    
                    if let Err(_err) = self.io_server.output_sender().send(ReadFrameProcessorActorOutputMessage::ClientMessage(message)).await
                    {

                        return false;

                    }

                }
                ReadFrameProcessorActorInputMessage::FrameAndMessage(frame, message) =>
                {

                    if let Err(_err) = self.io_server.output_sender().send(ReadFrameProcessorActorOutputMessage::ClientMessage(message)).await
                    {

                        return false;

                    }

                    let output = Self::format_output(frame);

                    if let Err(_err) = self.io_server.output_sender().send(ReadFrameProcessorActorOutputMessage::Processed(output)).await
                    {

                        return false;

                    }

                    //self.in_the_read_pipeline_count.fetch_sub(1, Ordering::SeqCst);

                }
                
            }

            //self.in_the_read_pipeline_count.dec();

            return true;

        }

        false

    }

    fn format_output(frame: OwnedFrame) -> String
    {

        #[derive(Debug)]
        enum PayloadOutput<'a>
        {

            Text(Cow<'a, str>),
            Binary(&'a Vec<u8>)

        }

        let opt_payload_output;

        if frame.payload.is_empty()
        {

            opt_payload_output = None;

        }
        else
        {

            match frame.opcode
            {
                
                OpCode::Text =>
                {

                    //Output u8 only option?
                    
                    let payload_as_utf8 = String::from_utf8_lossy(&frame.payload);
    
                    opt_payload_output = Some(PayloadOutput::Text(payload_as_utf8));
    
                }
                OpCode::Binary =>
                {
    
                    //Format as a particular type (JSON)?
                    
                    opt_payload_output = Some(PayloadOutput::Binary(&frame.payload));
    
                }
                OpCode::Continuation =>
                {
    
                    opt_payload_output = Some(PayloadOutput::Binary(&frame.payload));
    
                }
                OpCode::Close | OpCode::Ping | OpCode::Pong =>
                {

                    //Also interpret in other ways?

                    let payload_as_utf8 = String::from_utf8_lossy(&frame.payload);
    
                    opt_payload_output = Some(PayloadOutput::Text(payload_as_utf8));

                }
                /*
                OpCode::Close => todo!(),
                OpCode::Ping => todo!(),
                OpCode::Pong => todo!(),
                */
            }
            
        }

        match opt_payload_output
        {
            Some(payload_output) =>
            {

                match payload_output
                {

                    PayloadOutput::Text(text) =>
                    {

                        format!("{{\n\tfin: {fin},\n\topcode: {opcode:?},\n\tpayload: \"{text}\"\n}}", fin = frame.fin, opcode = frame.opcode)

                    }
                    PayloadOutput::Binary(binary) =>
                    {

                        format!("{{\n\tfin: {fin},\n\topcode: {opcode:?},\n\tpayload: {binary:?}\n}}", fin = frame.fin, opcode = frame.opcode)

                    }

                }

            }
            None =>
            {

                format!("{{\n\tfin: {fin},\n\topcode: {opcode:?}\n}}", fin = frame.fin, opcode = frame.opcode)

                //format!("{{\n\tfin: {fin},\n\topcode: {opcode:?},\n\tpayload: None\n}}", fin = frame.fin, opcode = frame.opcode)

            }

        }

        //format!("{{fin: {fin},\nopcode: {opcode:?},\npayload {payload:?}}}", fin = frame.fin, opcode = frame.opcode, payload = payload_output)

    }

}

//impl_mac_task_actor!(ReadFrameProcessorActorState, ReadFrameProcessorActor);

impl_mac_task_actor!(ReadFrameProcessorActor);
