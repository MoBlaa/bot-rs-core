use futures::{Stream, Sink, SinkExt, StreamExt};
use crate::Message;
use std::fmt;
use std::error::Error;
use futures::channel::mpsc::{self, Sender};

#[derive(Debug)]
pub struct SendError(Box<dyn Error>);

impl fmt::Display for SendError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "failed to send message to platforms: {}", self)
    }
}

impl Error for SendError {}

impl From<Box<dyn Error>> for SendError {
    fn from(why: Box<dyn Error>) -> Self {
        SendError(why)
    }
}

impl<E: 'static + Error + NotSendError> From<E> for SendError {
    fn from(why: E) -> Self {
        SendError(Box::new(why))
    }
}

pub auto trait NotSendError {}
impl ! NotSendError for SendError {}

#[derive(Debug, Clone)]
pub struct RecvError;

impl fmt::Display for RecvError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "failed to receive message from any platform")
    }
}

impl Error for RecvError {}

pub struct PluginOutput(Sender<Message>);

pub auto trait NotMpscSender {}
impl !NotMpscSender for Sender<Message> {}

impl PluginOutput {
    async fn send(&mut self, message: Message) -> Result<(), SendError> {
        Ok(self.0.send(message).await?)
    }
}

impl From<Sender<Message>> for PluginOutput {
    fn from(sender: Sender<Message>) -> Self {
        PluginOutput(sender)
    }
}

impl<E: Error, S: 'static + Sink<Message, Error=E> + NotMpscSender + Unpin + Send + Sync> From<S> for PluginOutput {
    fn from(mut sink: S) -> Self {
        let (sender, mut receiver) = mpsc::channel::<Message>(1000);

        tokio::spawn(async move {
            while let Some(item) = receiver.next().await {
                if let Err(why) = sink.send(item).await {
                    error!("Failed to delegate message: {:?}", why);
                    break;
                }
            }
        });

        PluginOutput(sender)
    }
}

pub struct PluginInput(Box<dyn Stream<Item=Message> + Unpin>);

impl PluginInput {
    async fn next(&mut self) -> Result<Message, RecvError> {
        if let Some(item) = self.0.next().await {
            Ok(item)
        } else {
            Err(RecvError)
        }
    }
}

pub struct Ctx {
    output: PluginOutput,
    input: PluginInput
}

impl Ctx {
    pub fn new<O, OE>(sink: O, stream: PluginInput) -> Self
        where
            O: 'static + Sink<Message, Error=OE> + NotMpscSender + Unpin + Send + Sync,
            OE: Error {
        Ctx {
            output: PluginOutput::from(sink),
            input: stream
        }
    }

    pub async fn send(&mut self, message: Message) -> Result<(), SendError> {
        self.output.send(message).await
    }

    pub async fn next(&mut self) -> Result<Message, RecvError> {
        self.input.next().await
    }
}