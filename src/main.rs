extern crate iron;
extern crate router;
extern crate serde;
extern crate serde_json;
extern crate time;

mod error;
mod items;
mod logger;

use error::Error;
use iron::prelude::*;
use iron::{headers, status, AroundMiddleware};
use iron::modifiers::Header;
use items::{TodoItems, TodoItemsMiddleware};
use logger::Logger;
use router::Router;
use serde::ser::{Serialize, Serializer};
use serde_json::ser::Serializer as JsonSerializer;
use std::io::Read;
use std::path::Path;

fn main() {
    let mut router = Router::new();
    router.get("/", index);
    router.get("/items", all);
    router.post("/items", create);
    router.delete("/items/:id", delete);

    let mut chain = Chain::new(router);
    chain.link_before(TodoItemsMiddleware::new(TodoItems::new()));

    let handler = logger::Logger.around(Box::new(chain));

    Iron::new(handler).http("localhost:3000").unwrap();
}

/// Serves the client application.
fn index(_: &mut Request) -> IronResult<Response> {
    Ok(Response::with((status::Ok, Path::new("data/index.html"))))
}

/// Responds with a JSON list of all existing todo items.
fn all(req: &mut Request) -> IronResult<Response> {
    let items = &*req.extensions.get::<TodoItems>().unwrap().lock().unwrap();
    json_response(&*items)
}

/// Creates a new todo item by title and responds with its issued unique identifier as JSON.
fn create(req: &mut Request) -> IronResult<Response> {
    let mut title = String::new();
    req.body.read_to_string(&mut title).unwrap();
    let mut items = req.extensions.get::<TodoItems>().unwrap().lock().unwrap();
    json_response(items.insert(title))
}

/// Deletes a todo item by its unique identifier.
fn delete(req: &mut Request) -> IronResult<Response> {
    let router = req.extensions.get::<Router>().unwrap();

    let id: u64 = match router.find("id").unwrap().parse() {
        Ok(id) => id,
        Err(_) => return Error::BadRequest.into_iron_result(),
    };

    let mut items = req.extensions.get::<TodoItems>().unwrap().lock().unwrap();

    if let None = items.remove(id) {
        return Error::NotFound.into_iron_result();
    }

    Ok(Response::with(status::Ok))
}

/// A helper function that generates a "200 OK" `Response` with a serialized item, encoded in JSON,
/// as its body.
fn json_response<T: Serialize>(t: T) -> IronResult<Response> {
    let mut serializer = JsonSerializer::new(vec![]);
    try!(serializer.serialize_some(t).map_err(|err| Error::JsonError(err)));
    let response = Response::with((status::Ok, Header(headers::ContentType::json())));
    Ok(response.set(serializer.into_inner()))
}
