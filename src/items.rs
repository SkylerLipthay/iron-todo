use iron::prelude::*;
use iron::BeforeMiddleware;
use iron::typemap::Key;
use serde::ser::{Serialize, Serializer};
use serde::ser::impls::MapIteratorVisitor;
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

/// A list of todo items where each item corresponds to a unique numerical identifier.
pub struct TodoItems {
    items: BTreeMap<u64, String>,
    next_id: u64,
}

impl TodoItems {
    /// Creates a blank list of todo items.
    pub fn new() -> TodoItems {
        TodoItems { items: BTreeMap::new(), next_id: 0 }
    }

    /// Inserts a new todo item with a title, returning its automatically issued identifier.
    pub fn insert(&mut self, title: String) -> u64 {
        let id = self.next_id;
        self.items.insert(id, title);
        self.next_id += 1;
        id
    }

    /// Removes and returns a todo item by its identifier. If the identifier does not exist, `None`
    /// is returned
    pub fn remove(&mut self, id: u64) -> Option<String> {
        self.items.remove(&id)
    }
}

/// Allows a `TodoItems` instance to be generically serialized by Serde.
///
/// The items are serialized as a key-value map between todo identifiers and respective titles. If,
/// for instance, the instance is serialized as JSON, the resultant string may look like:
///
/// ```json
/// {
///     "4": "Do the dishes",
///     "6": "Clean the shower",
///     "10": "Read a book"
/// }
/// ```
impl Serialize for TodoItems {
    fn serialize<S: Serializer>(&self, serializer: &mut S) -> Result<(), S::Error> {
        // JSON requires that object keys are strings, so we do that conversion here.
        serializer.serialize_map(MapIteratorVisitor::new(
            self.items.iter().map(|(k, v)| (k.to_string(), v)),
            Some(self.items.len())
        ))
    }
}

/// Allows a `TodoItems` instance to be persisted and shared across an Iron server's request
/// handlers.
impl Key for TodoItems {
    type Value = Arc<Mutex<TodoItems>>;
}

/// A middleware that makes available a shared `TodoItems` instance to all Iron request handlers.
pub struct TodoItemsMiddleware(Arc<Mutex<TodoItems>>);

impl TodoItemsMiddleware {
    pub fn new(items: TodoItems) -> TodoItemsMiddleware {
        TodoItemsMiddleware(Arc::new(Mutex::new(items)))
    }
}

impl BeforeMiddleware for TodoItemsMiddleware {
    fn before(&self, req: &mut Request) -> IronResult<()> {
        req.extensions.insert::<TodoItems>(self.0.clone());
        Ok(())
    }
}
