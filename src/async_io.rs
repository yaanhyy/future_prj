use std::io;
use tokio::prelude::*;

// This is going to be our Future.
// In the common case, this is set to Some(Reading),
// but we'll set it to None when we return Async::Ready
// so that we can return the reader and the buffer.
struct ReadExact<R, T>(Option<Reading<R, T>>);

struct Reading<R, T> {
    // This is the stream we're reading from.
    reader: R,
    // This is the buffer we're reading into.
    buffer: T,
    // And this is how far into the buffer we've written.
    pos: usize,
}

// We want to be able to construct a ReadExact over anything
// that implements AsyncRead, and any buffer that can be
// thought of as a &mut [u8].
fn read_exact<R, T>(reader: R, buffer: T) -> ReadExact<R, T>
    where
        R: AsyncRead,
        T: AsMut<[u8]>,
{
    ReadExact(Some(Reading {
        reader,
        buffer,
        // Initially, we've read no bytes into buffer.
        pos: 0,
    }))
}

impl<R, T> Future for ReadExact<R, T>
    where
        R: AsyncRead,
        T: AsMut<[u8]>,
{
    // When we've filled up the buffer, we want to return both the buffer
    // with the data that we read and the reader itself.
    type Item = (R, T);
    type Error = io::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        match self.0 {
            Some(Reading {
                     ref mut reader,
                     ref mut buffer,
                     ref mut pos,
                 }) => {
                let buffer = buffer.as_mut();
                // Check that we haven't finished
                while *pos < buffer.len() {
                    // Try to read data into the remainder of the buffer.
                    // Just like read in std::io::Read, poll_read *can* read
                    // fewer bytes than the length of the buffer it is given,
                    // and we need to handle that by looking at its return
                    // value, which is the number of bytes actually read.
                    //
                    // Notice that we are using try_ready! here, so if poll_read
                    // returns NotReady (or an error), we will do the same!
                    // We uphold the contract that we have arranged to be
                    // notified later because poll_read follows that same
                    // contract, and _it_ returned NotReady.
                    let n = try_ready!(reader.poll_read(&mut buffer[*pos..]));
                    *pos += n;

                    // If no bytes were read, but there was no error, this
                    // generally implies that the reader will provide no more
                    // data (for example, because the TCP connection was closed
                    // by the other side).
                    if n == 0 {
                        return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "early eof"));
                    }
                }
            }
            None => panic!("poll a ReadExact after it's done"),
        }

// We need to return the reader and the buffer, which we can only
// do by moving them out of self. We do this by taking our state
// and leaving `None`. This _should_ be fine, because poll()
// requires callers to not call poll() again after Ready has been
// returned, so we should only ever see Some(Rea
        // is called.
        let reading = self.0.take().expect("must have seen Some above");
        Ok(Async::Ready((reading.reader, reading.buffer)))
    }
}

// This is going to be our Future.
// It'll seem awfully familiar to ReadExact above!
// In the common case, this is set to Some(Writing),
// but we'll set it to None when we return Async::Ready
// so that we can return the writer and the buffer.
struct WriteAll<W, T>(Option<Writing<W, T>>);

struct Writing<W, T> {
    // This is the stream we're writing into.
    writer: W,
    // This is the buffer we're writing from.
    buffer: T,
    // And this is much of the buffer we've written.
    pos: usize,
}

// We want to be able to construct a WriteAll over anything
// that implements AsyncWrite, and any buffer that can be
// thought of as a &[u8].
fn write_all<W, T>(writer: W, buffer: T) -> WriteAll<W, T>
    where
        W: AsyncWrite,
        T: AsRef<[u8]>,
{
    WriteAll(Some(Writing {
        writer,
        buffer,
        // Initially, we've written none of the bytes from buffer.
        pos: 0,
    }))
}

impl<W, T> Future for WriteAll<W, T>
    where
        W: AsyncWrite,
        T: AsRef<[u8]>,
{
    // When we've written out the entire buffer, we want to return
    // both the buffer and the writer so that the user can re-use them.
    type Item = (W, T);
    type Error = io::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        match self.0 {
            Some(Writing {
                     ref mut writer,
                     ref buffer,
                     ref mut pos,
                 }) => {
                let buffer = buffer.as_ref();
                // Check that we haven't finished
                while *pos < buffer.len() {
                    // Try to write the remainder of the buffer into the writer.
                    // Just like write in std::io::Write, poll_write *can* write
                    // fewer bytes than the length of the buffer it is given,
                    // and we need to handle that by looking at its return
                    // value, which is the number of bytes actually written.
                    //
                    // We are using try_ready! here, just like in poll_read in
                    // ReadExact, so that if poll_write returns NotReady (or an
                    // error), we will do the same! We uphold the contract that
                    // we have arranged to be notified later because poll_write
                    // follows that same contract, and _it_ returned NotReady.
                    let n = try_ready!(writer.poll_write(&buffer[*pos..]));
                    *pos += n;

                    // If no bytes were written, but there was no error, this
                    // generally implies that something weird happened under us.
                    // We make sure to turn this into an error for the caller to
                    // deal with.
                    if n == 0 {
                        return Err(io::Error::new(
                            io::ErrorKind::WriteZero,
                            "zero-length write",
                        ));
                    }
                }
            }
            None => panic!("poll a WriteAll after it's done"),
        }

        // We use the same trick as in ReadExact to ensure that we can return
        // the buffer and the writer once the entire buffer has been written out.
        let writing = self.0.take().expect("must have seen Some above");
        Ok(Async::Ready((writing.writer, writing.buffer)))
    }
}

fn async_write_read() {

}

#[test]
fn async_write_read_test() {
    async_write_read();
}