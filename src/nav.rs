use crate::{Counter, Id};
use std::collections::BTreeMap;

/// Centralizes information related to the navigation tree.
///
/// # Fields
///
/// * focus - The current node in focus determines the active message delivered to the screen
/// reader.
/// * id - Owned [`Id`] used to generate unique ids for nodes.
/// * nodes - [`BTreeMap`] used to look up nodes by node id.
/// * tree - Contains the root tree, allowing us to change the app name delivered to the screen
/// reader.
#[derive(
    Debug, Clone, PartialEq, derive_new::new, derive_getters::Getters, derive_setters::Setters,
)]
#[setters(prefix = "with_")]
pub struct Nav {
    focus: accesskit::NodeId,
    id: Id<Counter, u64>,
    nodes: BTreeMap<accesskit::NodeId, accesskit::Node>,
    tree: accesskit::Tree,
}

impl Nav {
    /// Sets the app name.
    pub fn app_name(mut self, name: &str) -> Self {
        self.tree.app_name = Some(name.to_string());
        tracing::trace!("App name set to {name}");
        self
    }

    /// Returns a [`accesskit::TreeUpdate`] containing the full information for the tree.
    pub fn initial_tree(&self) -> accesskit::TreeUpdate {
        accesskit::TreeUpdate {
            nodes: self.into_nodes(),
            tree: Some(self.tree.clone()),
            focus: self.focus,
        }
    }

    pub fn intro() -> Self {
        // generate ids to track nodes
        let mut id = Id::counter();
        let msg_id = id.node_id();
        let win_id = id.node_id();

        // generate node content
        let desc = "Welcome to Tardy: the application for doing later what could be done today."
            .to_string();
        let msg = Self::message(&desc);
        let win = Self::window(vec![msg_id]);

        // insert into btreemap to track later
        let mut nodes = BTreeMap::new();
        nodes.insert(msg_id, msg);
        nodes.insert(win_id, win);

        // set focus to parent window and create tree
        let focus = win_id;
        let tree = accesskit::Tree::new(win_id);

        Self::new(focus, id, nodes, tree)
    }

    /// Converts the [`BTreeMap`] in the `nodes` field into a vector of tuples (key, value).
    pub fn into_nodes(&self) -> Vec<(accesskit::NodeId, accesskit::Node)> {
        self.nodes.clone().into_iter().collect()
    }

    /// Generates a message using the [`accesskit::Role::Label`] role.
    pub fn message(text: &str) -> accesskit::Node {
        let mut builder = accesskit::NodeBuilder::new(accesskit::Role::Label);
        builder.set_name(text);
        builder.set_live(accesskit::Live::Polite);
        builder.build()
    }

    /// Generates a node using the [`accesskit::Role::Window`] role.
    pub fn window(children: Vec<accesskit::NodeId>) -> accesskit::Node {
        let mut builder = accesskit::NodeBuilder::new(accesskit::Role::Window);
        builder.set_children(children);
        builder.set_name("Tardy");
        builder.build()
    }
}

impl Default for Nav {
    fn default() -> Self {
        // let desc = "The application for doing later what could be done today.".to_string();
        // let msg = Self::message(&desc);
        let win = Self::window(Vec::new());
        Self::from(win).app_name("Tardy")
    }
}

impl From<accesskit::Node> for Nav {
    fn from(node: accesskit::Node) -> Self {
        let mut id = Id::counter();
        let node_id = id.node_id();
        let focus = node_id;
        let mut nodes = BTreeMap::new();
        nodes.insert(node_id, node);
        let tree = accesskit::Tree::new(node_id);
        Self::new(focus, id, nodes, tree)
    }
}
// fn build_root(&mut self) -> Node {
//     let mut builder = NodeBuilder::new(Role::Window);
//     builder.set_children(vec![BUTTON_1_ID, BUTTON_2_ID]);
//     if self.announcement.is_some() {
//         builder.push_child(ANNOUNCEMENT_ID);
//     }
//     builder.set_name(WINDOW_TITLE);
//     builder.build()
// }
