//! A recursive tree of terminal panes inside one tab. Splitting a pane
//! replaces its leaf with a horizontal `gtk::Paned` holding the old pane
//! and the new one.

use std::cell::RefCell;
use std::rc::Rc;

use gtk4 as gtk;
use gtk4::prelude::*;

use crate::pane::TerminalPane;

pub enum PaneTree {
    Leaf(Rc<RefCell<TerminalPane>>),
    Split {
        paned: gtk::Paned,
        a: Box<PaneTree>,
        b: Box<PaneTree>,
    },
}

impl PaneTree {
    pub fn leaf(pane: Rc<RefCell<TerminalPane>>) -> PaneTree {
        PaneTree::Leaf(pane)
    }

    pub fn widget(&self) -> gtk::Widget {
        match self {
            PaneTree::Leaf(pane) => pane.borrow().container.clone().upcast(),
            PaneTree::Split { paned, .. } => paned.clone().upcast(),
        }
    }

    pub fn contains(&self, id: u64) -> bool {
        match self {
            PaneTree::Leaf(pane) => pane.borrow().id == id,
            PaneTree::Split { a, b, .. } => a.contains(id) || b.contains(id),
        }
    }

    /// Id of the currently focused leaf, if any.
    pub fn focused_id(&self) -> Option<u64> {
        fn walk(tree: &PaneTree) -> Option<u64> {
            match tree {
                PaneTree::Leaf(pane) => {
                    let pane = pane.borrow();
                    if pane.term.is_focus() {
                        Some(pane.id)
                    } else {
                        None
                    }
                }
                PaneTree::Split { a, b, .. } => walk(a).or_else(|| walk(b)),
            }
        }
        walk(self)
    }

    /// Id of the first leaf (used as a fallback when nothing has focus).
    pub fn first_id(&self) -> u64 {
        match self {
            PaneTree::Leaf(pane) => pane.borrow().id,
            PaneTree::Split { a, .. } => a.first_id(),
        }
    }

    pub fn profile_of(&self, id: u64) -> Option<crate::config::Profile> {
        match self {
            PaneTree::Leaf(pane) => {
                let pane = pane.borrow();
                if pane.id == id {
                    Some(pane.profile.clone())
                } else {
                    None
                }
            }
            PaneTree::Split { a, b, .. } => a.profile_of(id).or_else(|| b.profile_of(id)),
        }
    }

    /// Split the leaf `target` side by side with `new_leaf`. Returns true on
    /// success. The caller must re-attach [`PaneTree::widget`] to the page.
    pub fn split(&mut self, target: u64, new_leaf: Rc<RefCell<TerminalPane>>) -> bool {
        match self {
            PaneTree::Leaf(pane) if pane.borrow().id == target => {
                let old_leaf = PaneTree::Leaf(pane.clone());
                let paned = gtk::Paned::new(gtk::Orientation::Horizontal);
                paned.set_hexpand(true);
                paned.set_vexpand(true);
                paned.set_start_child(Some(&old_leaf.widget()));
                paned.set_end_child(Some(&new_leaf.borrow().container));
                paned.set_shrink_start_child(false);
                paned.set_shrink_end_child(false);
                *self = PaneTree::Split {
                    paned,
                    a: Box::new(old_leaf),
                    b: Box::new(PaneTree::Leaf(new_leaf)),
                };
                true
            }
            PaneTree::Leaf(_) => false,
            PaneTree::Split { a, b, .. } => {
                if a.contains(target) {
                    a.split(target, new_leaf)
                } else {
                    b.split(target, new_leaf)
                }
            }
        }
    }

    /// Remove the leaf `id`, collapsing splits that become single-pane.
    /// Returns `None` when the tree becomes empty.
    pub fn remove(self, id: u64) -> Option<PaneTree> {
        match self {
            PaneTree::Leaf(pane) => {
                if pane.borrow().id == id {
                    None
                } else {
                    Some(PaneTree::Leaf(pane))
                }
            }
            PaneTree::Split { paned, a, b } => {
                if a.contains(id) {
                    match a.remove(id) {
                        Some(new_a) => {
                            paned.set_start_child(Some(&new_a.widget()));
                            Some(PaneTree::Split {
                                paned,
                                a: Box::new(new_a),
                                b,
                            })
                        }
                        None => Some(*b),
                    }
                } else {
                    match b.remove(id) {
                        Some(new_b) => {
                            paned.set_end_child(Some(&new_b.widget()));
                            Some(PaneTree::Split {
                                paned,
                                a,
                                b: Box::new(new_b),
                            })
                        }
                        None => Some(*a),
                    }
                }
            }
        }
    }
}
