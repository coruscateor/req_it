use std::future::Future;

use fastwebsockets::{FragmentCollectorRead, Frame, WebSocketError, WebSocketRead};

use hyper::upgrade::Upgraded;

use hyper_util::rt::TokioIo;


//#[derive(Debug)]
enum WebSocketReader
{

    WebSocketRead(WebSocketRead<TokioIo<Upgraded>>),
    FragmentCollectorRead(FragmentCollectorRead<TokioIo<Upgraded>>)

}

impl WebSocketReader
{

    pub fn is_websocket(&self) -> bool
    {

        if let WebSocketReader::WebSocketRead(_) = self
        {
            
            return true;

        }

        false

    }

    pub fn is_fragmentcollectorRead(&self) -> bool
    {

        if let WebSocketReader::FragmentCollectorRead(_) = self
        {
            
            return true;

        }

        false

    }

    pub async fn read_frame<R, E>(&mut self, send_fn: &mut impl FnMut(Frame<'_>) -> R) -> Result<Frame<'_>, WebSocketError>
        where E: Into<Box<dyn std::error::Error + Send + Sync + 'static>>,
              R: Future<Output = Result<(), E>>
    {

        match self
        {

            WebSocketReader::WebSocketRead(ws) =>
            {

                ws.read_frame(send_fn).await

            },
            WebSocketReader::FragmentCollectorRead(fc) =>
            {

                fc.read_frame(send_fn).await

            }

        }

    }

    /*
    pub async fn write_frame(&mut self, frame: Frame<'_>) -> Result<(), WebSocketError>
    {

        match self
        {

            WebSocketReader::WebSocket(ws) =>
            {

                ws.write_frame(frame).await

            },
            WebSocketReader::FragmentCollector(fc) =>
            {

                fc.write_frame(frame).await

            }

        }

    }

    pub fn into_inner(self) -> TokioIo<Upgraded>
    {

        match self
        {

            WebSocketReader::WebSocketRead(ws) =>
            {

                ws.into_inner()

            },
            WebSocketReader::FragmentCollectorRead(fc) =>
            {

                fc.into_inner()

            }

        }

    }

    pub async fn shutdown(self) -> Result<(), std::io::Error>
    {

        self.into_inner().shutdown().await

    }
    */
    
}