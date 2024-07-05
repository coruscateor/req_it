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

    pub fn copy_from_frame(&mut self, frame: &mut Frame<'_>)
    {

        self.fin = frame.fin;

        self.opcode = frame.opcode;

        frame.write(&mut self.payload);

    }

    pub fn reset(&mut self,)
    {

        self.fin = false;

        self.opcode = OpCode::Continuation;

        self.payload.clear();

    }

    pub fn copy_into_frame(&mut self, frame: &mut Frame<'_>)
    {

        frame.fin = self.fin;

        frame.opcode = self.opcode;

        //self.payload

        //let vec

        match &mut frame.payload
        {

            Payload::BorrowedMut(val) =>
            {

                frame.payload = Payload::Owned(val.to_vec());

            },
            Payload::Borrowed(val) =>
            {

                frame.payload = Payload::Owned(val.to_vec());

            },
            fastwebsockets::Payload::Owned(val) =>
            {

                //val.clear();

                let payload = &mut self.payload;

                //let val_capacity = val.capacity();

                //let payload_capacity = payload.capacity();

                let payload_len = payload.len();

                //if val.len() < payload_len //val.capacity() < payload_capacity
                //{

                    //val.resize(payload_len, 0); //payload_capacity, 0);

                //}

                val.resize(payload_len, 0);

                val.copy_from_slice(&payload);
                
                
            },
            fastwebsockets::Payload::Bytes(val) =>
            {

                frame.payload = Payload::Owned(val.to_vec());

            }

        }

    }

}