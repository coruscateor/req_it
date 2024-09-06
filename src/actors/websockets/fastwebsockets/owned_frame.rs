use std::default;

use bytes::BytesMut;

use fastwebsockets::{Frame, OpCode, Payload};

/*
pub enum OwnedPayload
{

    Vec(Vec<u8>),
    BytesMut(BytesMut)

}
*/

#[derive(Debug)]
pub struct OwnedFrame
{

    pub fin: bool,
    pub opcode: OpCode,
    pub payload: Vec<u8> //OwnedPayload

}

impl OwnedFrame
{

    pub fn new() -> Self
    {

        Self
        {

            fin: Default::default(),
            opcode: OpCode::Continuation,
            payload: Default::default()

        }
        
    }

    pub fn with_capacity(capacity: usize) -> Self
    {

        Self
        {

            fin: Default::default(),
            opcode: OpCode::Continuation,
            payload: Vec::with_capacity(capacity)

        }
        
    }

    pub fn from_frame(frame: &mut Frame<'_>) -> Self
    {

        let mut payload = Vec::new(); //= Default::default();

        frame.write(&mut payload);

        Self
        {

            fin: frame.fin,
            opcode: frame.opcode,
            payload

        }

    }

    pub fn copy_all_from_read_frame(&mut self, frame: &Frame<'_>)
    {

        self.fin = frame.fin;

        self.opcode = frame.opcode;

        self.copy_payload_from_read_frame(frame);

        //frame.write(&mut self.payload);

        /*
        let payload = &mut self.payload;

        //let payload_len = payload.len();

        let frame_payload_len = frame.payload.len();

        if payload.len() != frame_payload_len
        {

            payload.resize(frame_payload_len, 0)

        }

        payload.copy_from_slice(&frame.payload);
        
        */

    }

    pub fn copy_payload_from_read_frame(&mut self, frame: &Frame<'_>)
    {

        let payload = &mut self.payload;

        let frame_payload_len = frame.payload.len();

        if payload.len() != frame_payload_len
        {

            payload.resize(frame_payload_len, 0)

        }

        payload.copy_from_slice(&frame.payload);
        
    }


    pub fn reset(&mut self,)
    {

        self.fin = false;

        self.opcode = OpCode::Continuation;

        self.payload.clear();

    }

    pub fn setup_frame_to_be_written<'f>(&'f mut self, frame: &mut Frame<'f>)
    {

        frame.fin = self.fin;

        frame.opcode = self.opcode;

        frame.payload = Payload::BorrowedMut(&mut self.payload);

    }

    pub fn new_frame_to_be_written(&mut self) -> Frame
    {

        let mut frame = Frame::new(false, OpCode::Continuation, None, vec![].into());

        self.setup_frame_to_be_written(&mut frame);

        frame

    }

    //Valid opcode and fin field arrangements.

    pub fn continuation_setup(&mut self)
    {

        self.opcode = OpCode::Continuation;

        self.fin = false;

    }

    pub fn final_continuation_setup(&mut self)
    {

        self.opcode = OpCode::Continuation;

        self.fin = true;

    }

    pub fn text_setup(&mut self)
    {

        self.opcode = OpCode::Text;

        self.fin = true;

    }

    pub fn text_continuation_stater_setup(&mut self)
    {

        self.opcode = OpCode::Text;

        self.fin = false;

    }

    pub fn binary_setup(&mut self)
    {

        self.opcode = OpCode::Binary;

        self.fin = true;

    }

    pub fn binary_continuation_stater_setup(&mut self)
    {

        self.opcode = OpCode::Binary;

        self.fin = false;

    }

    pub fn close_setup(&mut self)
    {

        self.opcode = OpCode::Close;

        self.fin = true;

    }

    pub fn ping_setup(&mut self)
    {

        self.opcode = OpCode::Ping;

        self.fin = true;

    }

    pub fn pong_setup(&mut self)
    {

        self.opcode = OpCode::Pong;

        self.fin = true;

    }

    pub fn pong_setup_with_payload(&mut self, frame: &Frame<'_>)
    {

        self.pong_setup();

        self.copy_payload_from_read_frame(frame);

    }

    pub fn set_payload_from_str(&mut self, contents: &str)
    {
        
        //Set the payload of the OwnedFrame the right size.

        let content_bytes = contents.as_bytes();

        let payload = &mut self.payload;
        
        let cb_len = content_bytes.len();

        if cb_len != payload.len()
        {

            payload.resize(cb_len, 0);

        }

        //Copy the bytes into the OwnedFrame payload. 

        payload.copy_from_slice(content_bytes);

    }

    pub fn clear_payload(&mut self)
    {

        self.payload.clear();

    }

}