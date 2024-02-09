use std::{
    io::Error,
    pin::Pin,
    task::{Context, Poll},
};
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};

/// Utility to wrap a tuple (`T`, `U`),
/// where `T: AsyncRead` and `U: AsyncWrite`,
/// into a type implementing both of those traits.
#[pin_project::pin_project]
pub struct IoDuplex<T, U> {
    #[pin]
    reader: T,
    #[pin]
    writer: U,
}

impl<T, U> IoDuplex<T, U> {
    pub fn new(reader: T, writer: U) -> Self {
        Self { reader, writer }
    }
}

impl<T, U> AsyncRead for IoDuplex<T, U>
where
    T: AsyncRead,
{
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        AsyncRead::poll_read(self.project().reader, cx, buf)
    }
}

impl<T, U> AsyncWrite for IoDuplex<T, U>
where
    U: AsyncWrite,
{
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, Error>> {
        AsyncWrite::poll_write(self.project().writer, cx, buf)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Error>> {
        AsyncWrite::poll_flush(self.project().writer, cx)
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Error>> {
        AsyncWrite::poll_shutdown(self.project().writer, cx)
    }
}
