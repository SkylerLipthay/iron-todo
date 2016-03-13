use iron::prelude::*;
use iron::{Handler, AroundMiddleware};
use time;

/// A middleware around an Iron request handler that logs to `stdout` information about the request
/// including how long it took to process, its HTTP status code, and its error (if any).
///
/// Examples of logged lines:
///
/// ```plaintext
/// POST http://localhost:3000/items → 200 OK in 0.028068ms
/// DELETE http://localhost:3000/items/0 → 200 OK in 0.037205ms
/// DELETE http://localhost:3000/items/text → 400 Bad Request in 0.031282ms (BadRequest)
/// GET http://localhost:3000/items → 200 OK in 0.041985ms
/// ```
pub struct Logger;

struct LoggerHandler<H: Handler>(H);

impl<H: Handler> Handler for LoggerHandler<H> {
    fn handle(&self, req: &mut Request) -> IronResult<Response> {
        let entry = time::precise_time_ns();
        let res = self.0.handle(req);
        let time_ns = (time::precise_time_ns() - entry) as f64;
        let time_ms = time_ns / 1.0e6;

        match res {
            Ok(res) => {
                println!("{} {} → {} in {}ms", req.method, req.url, format_status(&res), time_ms);
                Ok(res)
            },
            Err(IronError { error, response: res }) => {
                println!("{} {} → {} in {}ms ({:?})", req.method, req.url, format_status(&res),
                         time_ms, error);
                Ok(res)
            },
        }
    }
}

fn format_status(res: &Response) -> String {
    res.status.map_or(String::from("???"), |s| s.to_string())
}

impl AroundMiddleware for Logger {
    fn around(self, handler: Box<Handler>) -> Box<Handler> {
        Box::new(LoggerHandler(handler))
    }
}
