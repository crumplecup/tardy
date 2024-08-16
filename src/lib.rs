//! The `tardy` crate is an asyncronous application that will do later what could be done now.
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
mod act;
mod app;
mod cmd;
mod lens;
mod utils;

/// Since this is a small application, we lift all user-facing data types and functions to the parent namespace
/// for ease of access.
pub use act::Act;
pub use app::App;
pub use cmd::Cmd;
pub use lens::Lens;
pub use utils::trace_init;
