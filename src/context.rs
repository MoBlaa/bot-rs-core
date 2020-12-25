use futures::{Sink, SinkExt, Future};
use crate::Message;
use futures::channel::mpsc;

#[derive(Clone)]
pub struct Ctx {
    output: mpsc::Sender<Message>
}

impl Ctx {
    pub fn new(output: mpsc::Sender<Message>) -> Self {
        Ctx {
            output
        }
    }

    pub fn spawn<F>(&self, future: F) where F: Future<Output=()> + Send + 'static {
        tokio::spawn(future);
    }

    pub async fn send(&mut self, message: Message) {
        self.output.send(message).await.expect("failed to send message to output");
    }

    pub fn output(&mut self) -> &mut impl Sink<Message> {
        &mut self.output
    }
}