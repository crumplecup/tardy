use crate::Act;
use std::collections::HashMap;
use strum::IntoEnumIterator;
use winit::{event, keyboard};

/// The `cmd` module maps keyboard input from the user to variants of the [`Act`] enum as the
/// mechanism for dispatching actions based on incoming keyboard events.
///
/// # Reading commands from a configuration file with `Cmd`
///
/// The `Cmd` struct is a wrapper around a [`HashMap<String, Act>`], where the [`String`] keys are
/// the logical key that will serve as the trigger for the [`Act`] contained in the value.  We use
/// the [`derive_more`] crate to implement [`derive_more::Deref`], and [`derive_more::DerefMut`],
/// so that we can easily access the methods of the underlying [`HashMap`].
///
/// We lean on the [`derive_new`] crate for a boilerplate implementation of the [`Cmd::new`]
/// method.
///
/// Maybe this should be named `Command`, but I do not feel like doing the extra typing today.
#[derive(
    Debug, Default, Clone, PartialEq, Eq, derive_new::new, derive_more::Deref, derive_more::DerefMut,
)]
pub struct Cmd(HashMap<String, Act>);

impl Cmd {
    /// Given an incoming [`event::KeyEvent`] from the [`winit`] crate, the `act` method checks the
    /// [`HashMap`] in `Self` to determine if the key event maps to an [`Act`] variant.
    pub fn act(&self, event: &event::KeyEvent) -> Option<Act> {
        match event.logical_key.as_ref() {
            keyboard::Key::Named(k) => {
                let key = format!("{k:?}");
                if let Some(act) = self.get(&key) {
                    tracing::trace!("Act detected: {act}");
                    Some(act.clone())
                } else {
                    None
                }
                // tracing::trace!("Named key not implemented: {k:?}");
                // None
            }
            keyboard::Key::Character(k) => {
                tracing::trace!("Character event: {k}");
                if let Some(value) = self.get(k) {
                    Some(value.clone())
                } else {
                    tracing::trace!("Command key not present {k}");
                    None
                }
            }
            other => {
                tracing::trace!("Unrecognized key event: {other:?}");
                None
            }
        }
    }
}

/// Here we rely on the [`strum`] and [`strum_macros`] crates to generate an iterator method over the
/// variants of [`Act`].  For each variant, we check to see if the [`config::Config`] passed in by the `config` argument  contains the snake
/// case version of the variant name as a key.  When a key is present, the method inserts a new
/// entry into [`HashMap`] in `Self` using the `config` value as a key, and the corresponding [`Act`] variant as the value.
///
/// The table in the [`config::Config`] has [`String`] representations of [`Act`] variants as keys
/// and keyboard characters as values.  We need the reverse, where the keyboard character enetered by the
/// user is the key, and the triggered [`Act`] is the value, so we create a new [`HashMap`] with
/// this inverse relationship, stored in the `Cmd` struct.
impl From<&config::Config> for Cmd {
    fn from(config: &config::Config) -> Self {
        let mut cmds = HashMap::new();
        let table = config.cache.clone().into_table().unwrap();
        Act::iter()
            .map(|a| {
                let key = a.snake();
                if let Some(entry) = table.get(&key) {
                    tracing::trace!("Command detected: {a}");
                    let value = entry.clone().into_string().unwrap();
                    cmds.insert(value, a.clone());
                }
            })
            .for_each(drop);
        if cmds.is_empty() {
            tracing::trace!("No valid commands detected!");
        }
        Self::new(cmds)
    }
}
