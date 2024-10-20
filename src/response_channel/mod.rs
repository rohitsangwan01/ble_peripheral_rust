/// Original Author:  https://github.com/raunakab/response_channel/tree/master
pub mod response_error;
use std::ops::Deref;
use tokio::sync::mpsc;

pub fn channel<M, R>(
    buffer: usize,
    reverse_buffer: Option<usize>,
) -> (Sender<M, R>, mpsc::Receiver<(M, mpsc::Sender<R>)>) {
    let (tx, rx) = mpsc::channel(buffer);
    let (reverse_tx, reverse_rx) = mpsc::channel(reverse_buffer.unwrap_or(buffer));
    (
        Sender {
            tx,
            reverse_tx,
            reverse_rx,
        },
        rx,
    )
}

pub struct Sender<M, R> {
    pub(crate) tx: mpsc::Sender<(M, mpsc::Sender<R>)>,
    pub(crate) reverse_tx: mpsc::Sender<R>,
    pub(crate) reverse_rx: mpsc::Receiver<R>,
}

impl<M, R> Sender<M, R> {
    pub async fn send_await_automatic(
        &mut self,
        message: M,
    ) -> Result<Option<R>, response_error::Error<M>> {
        self.send_await(message).await?;
        let response = self.reverse_rx.recv().await;
        Ok(response)
    }

    pub async fn send_await(&self, message: M) -> Result<(), response_error::Error<M>> {
        self.tx
            .send((message, self.reverse_tx.clone()))
            .await
            .map_err(|mpsc::error::SendError((m, _))| mpsc::error::SendError(m))?;
        Ok(())
    }

    // pub async fn recv(&mut self) -> Option<R> {
    //     self.reverse_rx.recv().await
    // }
}

impl<M, R> Clone for Sender<M, R> {
    fn clone(&self) -> Self {
        let reverse_buffer = self.reverse_tx.max_capacity();
        let (reverse_tx, reverse_rx) = mpsc::channel(reverse_buffer);
        Self {
            tx: self.tx.clone(),
            reverse_tx,
            reverse_rx,
        }
    }
}

impl<M, R> Deref for Sender<M, R> {
    type Target = mpsc::Sender<(M, mpsc::Sender<R>)>;

    fn deref(&self) -> &Self::Target {
        &self.tx
    }
}
