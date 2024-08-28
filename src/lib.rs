//! The `tardy` crate is an asyncronous application that will do later what could be done now.
//!
//! ## Version 0.1.0 Baseline
//!
//! The inspiration for this code comes from Chapter 17 of the Rust book, which the authors have
//! kindly allowed me to review.  In this project, we will apply the concepts introduced in this
//! chapter to add async functionality to our app.  A disadvantage of embedding reviewer comments
//! into code docs is that much of documentation generated comes from traits like
//! [`winit::application::ApplicationHandler`] and third party crates like
//! [`derive_more`].  So that you the reviewer can easily add my 2 cents to Carol's 10, here is a
//! link list to documentation written explicity for your consumption:
//!
//! 1. Creating Windows with Tardy - [`App`]
//!     * [`App::new`]
//!     * [`App::create_window`]
//!     * [`App::load_config`]
//!     * [`App::load_cmds`]
//!     * [`App::keyboard_input`]
//!     * [`App::act`]
//!     * See also the doc comment above the impl for `ApplicationHandler`.
//! 2. Representing window state with `Lens` - [`Lens`]
//! 3. Reading commands from a configuration file with `Cmd` - [`Cmd`]
//!     * [`Cmd::act`]
//!     * See also the doc comment above the impl for `From<&config::Config>`.
//! 4. Dispatching actions with `Act` - [`Act`]
//!     * See also the doc comment above the impl for [`Act`].
//!
//! In the first installment, I create a simplified version of the
//! [window]("https://github.com/rust-windowing/winit/blob/master/examples/window.rs") example
//! from the `winit` crate.  The next step is to introduce async mechanics. Long term stretch goals
//! include rendering content to the window.  But don't rush me.
//!
//! The primary entry point for the program is the [`App`] struct.
//! The program can perform two functions, opening a window and closing a window.
//! You can map keyboard input to these actions by editing the `Tardy.toml` file.
//! The process of reading keyboard mappings from the `Tardy.toml` file goes in three stages.
//! The initial read of the config file happens using the [`config`] crate.
//! We translate the resulting [`config::Config`] into a program response using the following
//! progression: [`config::Config`] -> [`Cmd`] -> [`Act`].
//! The [`Lens`] struct holds application state associated with a window.
//!
//! To use the library, we create a [`winit::event_loop::EventLoop`] in the global application space, and an instance of
//! [`App`].  We read the configuration file into the struct using [`App::load_config`] and
//! [`App::load_cmds`], then pass a reference to our app into
//! [`winit::event_loop::EventLoop::run_app`].
//!
//! We decorate the main function with `#[tokio::main]`, using [`tokio`] for our runtime.  We will
//! be working with the `tokio` analogues to the `block_on`, `spawn_task`, `channel`, `join!` and
//! other building blocks from the `trpl` repository.
//!
//! ## Version 0.1.1 Update
//!
//! One of the first things I noticed when I started writing async closures like the listings in
//! Chapter 17, is that the main event loop in [`winit`] is a sync process.  This caused me to
//! suspect I had picked a poor soil in which to grow my async garden.  After reflecting a while, I
//! decided it was a good thing.  For one, `winit` was already the library I wanted to use.
//! I might provide more interesting feedback trying to use it with the crates I would use for
//! "real work".
//!
//! The second, and more important reason, is that a lot of libraries have sync processes or APIs,
//! and what am I going to do then?  Do I even understand Chapter 17?  I decided to forge ahead.
//!
//! My initial impulse was to use [`std::sync::mpsc::channel`] to pass messages between the sync
//! and async processes, following the advice of Tokio's [Bridging with sync
//! code]("https://tokio.rs/tokio/topics/bridging") on *Sending Messages*.  This works but it turns
//! out to be less ergonomic than using the API exposed by `winit`. An optional method of the
//! [`winit::application::ApplicationHandler`] trait is called `user_event`.  When implemented, we
//! can send messages from our async processes back to the sync event loop by passing an event loop
//! proxy to the async process and calling [`winit::event_loop::EventLoopProxy::send_event`].
//!
//! Here is a link list to the new doc content:
//!
//! 1. Making Hijinks with Imps - [`Imp`]
//!   * [`Hijinks`]
//!   * [`Meddle`]
//!   * [`Filch`]
//!   * [`Imp::pause`]
//!   * [`Imp::instigate`]
//!   * [`Imp::filch`]
//!   * [`Imp::spoil`]
//!   * [`Imp::vandalize`]
//!   * [`Imp::meddle`]
//! 2. The Reign of the Imp King - [`ImpKing`]
//!   * [`Quote`]
//!   * [`Quotes`]
//!   * [`ImpKing::summon`]
//!   * [`ImpKing::imps`]
//!   * [`ImpKing::spawn_imps`]
//!   * [`ImpKing::listen`]
//!   * [`ImpKing::reign`]
//!
//!
//! In terms of demonstrating concepts from the relevant chapter, this update does not go beyond
//! Section 17.3.  You might be thinking *"Only section 17.3?"*, but I am thinking *"Heck yeah I made it
//! to 17.3!"*.  I would like to continue to implement some of the concepts discussed in later
//! sections, in particular the composition of streams.  Before I get to this, I would like to
//! introduce something besides empty windows, so the next update will also feature some new
//! content.
mod act;
mod app;
mod arrive;
mod cmd;
mod imp;
mod lens;
mod utils;

// Since this is a small application, we lift all user-facing data types and functions to the parent namespace
// for ease of access.
pub use act::Act;
pub use app::{App, Frame, FRAMES, FRAME_POOL, IMPS, MIN_SPAN};
pub use arrive::{Arrive, Blame, Excuse};
pub use cmd::Cmd;
pub use imp::{Filch, Hijinks, Imp, ImpKing, Meddle, Quote, Quotes};
pub use lens::Lens;
pub use utils::trace_init;
