use crate::{Event, Hijinks};
use derive_more::Error;

/// The `arrive` module holds error handling types for the `tardy` crate.
///
/// With the announcement that [`derive_more`] has gone version 1.0.0, they added that the `Error`
/// implementation was capable of replacing `thiserror` in most cases.  As a long-time user of
/// `thiserror`, this caught my attention.  I also started seeing different errors when using the
/// question mark operator on new error types.  *Other libraries* were using [`derive_more`] for
/// their error types.  It appears I was witnessing a shift in the ecosystem.  In order to get with
/// the program, I am starting from scratch implementing error handling using [`derive_more`].
///
/// This process requires implementing [`derive_more::From`] for the error types I want to handle.
/// What is unclear in the documentation is that the [`Error`] type provides a [`Error::source`]
/// method.  Without calling this method and displaying the source, I lose the connecting thread to
/// the causal issue.  Here I have tried piping the error into the display, we will see if it
/// works.
///
/// The `Blame` enum is the primary [`Error`] type.  The purpose of `Blame` is primary to enable
/// conversion of errors from other libraries into a common type using the question mark operator.
/// Library-specific errors fall under the `Excuse` variant, which contains an [`Excuse`] enum with
/// variants for different internal error conditions.
#[derive(
    Debug,
    Copy,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    derive_more::Display,
    Error,
    derive_more::From,
)]
pub enum Blame {
    /// Triggered when the [`csv`] crate is unable to read the `.csv` file containing inspirational
    /// quotes.  Easy to find by feeding it a bogus path.
    #[from(csv::Error)]
    #[display("Csv: {:?}", self.source())]
    Csv,
    /// The `EventLoop` variant triggers on failure to create a new [`winit`] event loop.
    #[from(winit::error::EventLoopError)]
    #[display("EventLoop: {:?}", self.source())]
    EventLoop,
    /// The `EventLoopClosed` variant occurs when an async event tries to send a message to event
    /// loop after it has been closed.
    #[from(winit::event_loop::EventLoopClosed<Hijinks>)]
    #[from(winit::event_loop::EventLoopClosed<Event>)]
    #[display("EventLoopClosed: {:?}", self.source())]
    EventLoopClosed,
    /// The `Excuse` variant indicates an internal library error.  
    /// The variant contains an [`Excuse`] enum that describes the error condition.
    Excuse(Excuse),
    /// The `Io` variant indicates an error opening the file location where the csv quotes should
    /// be.
    #[from(std::io::Error)]
    #[display("Io: {:?}", self.source())]
    Io,
    /// The `Oneshot` variant indicates an error in the [`tokio`] oneshot channel used to call for
    /// [`crate::Frame`] instances from the [`crate::App`] and receive new frames.
    #[from(tokio::sync::oneshot::error::RecvError)]
    #[display("Oneshot: {:?}", self.source())]
    Oneshot,
    /// The `OsError` variant indicates an error from the [`winit`] crate.
    #[from(winit::error::OsError)]
    #[display("OsError: {:?}", self.source())]
    OsError,
    /// The `Tokio` variant indicates an error with the mpsc channel used to send [`Hijinks`] from
    /// [`crate::Imp`] types to the [`crate::ImpKing`].
    #[from(tokio::sync::mpsc::error::SendError<Hijinks>)]
    #[from(tokio::sync::mpsc::error::SendError<Event>)]
    #[display("Tokio: {:?}", self.source())]
    Tokio,
}

/// The `Arrive` type is an alias of the [`Result`] type, using the common error type [`Blame`].
/// The purpose of `Arrive` is to facilitate the use of the question mark operator for bubbling
/// errors up to the appropriate handler.
pub type Arrive<T> = Result<T, Blame>;

/// The `Excuse` enum describes internal library error states.
/// Currently the only variant is `NoFrames`, indicating the struct does not have a frame to pop
/// from the `frames` field.
#[derive(
    Debug,
    Copy,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    derive_more::Error,
    derive_more::Display,
)]
pub enum Excuse {
    /// The `NoFrames` variant indicates the struct does not have a frame to pop from the
    /// `frames` field.
    NoFrames,
    NavBuilder,
}
