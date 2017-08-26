use std::str::FromStr;

use futures::{Future, Stream, future};

use hyper::{Method, Request, Uri};
use hyper::header::Authorization;

use serde_json::from_slice;

use error::{Error, DatabaseError};
use types::{DatabaseResponse as Response, User, Client};

#[derive(Clone)]
pub struct Database {
  uri: Uri,
  token: String,
  client: Client,
}

impl Database {
  pub fn new(token: String, client: Client) -> Database {
    Database {
      token: token,
      client: client,
      uri: Uri::from_str(
        "https://us-central1-sagiri-izumi.cloudfunctions.net/api/kitsu/users.json",
      ).unwrap(),
    }
  }

  pub fn fetch(&self) -> Box<Future<Item = Vec<User>, Error = Error>> {
    let mut req = Request::new(Method::Get, self.uri.clone());
    req.headers_mut().set(Authorization(self.token.clone()));

    Box::new(self.client.request(req).from_err::<Error>().and_then(
      |res| {
        res
          .body()
          .from_err::<Error>()
          .concat2()
          .and_then(|chunks| {
            future::result::<Response, Error>(from_slice(&chunks).map_err(|e| e.into()))
          })
          .and_then(|response| match response {
            Response::Ok { data } => Ok(data),

            Response::Error { error } => {
              return Err(Error::Database(DatabaseError { description: error }));
            }
          })
      },
    ))
  }
}