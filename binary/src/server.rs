use std::error::Error;
use std::io;
use config::VERSION;
use database::{SqliteDatabase, Bookmark, BookmarkId};
use serde_json;
use chrome_native_messaging::{event_loop, write_output, errors};
use utils::empty_result_err;

type JSON = serde_json::Value;

#[derive(Deserialize)]
struct RequestData {
    bookmark: Option<Bookmark>,
    bookmark_id: Option<BookmarkId>,
}

#[derive(Deserialize)]
struct Request {
    method: String,
    data: Option<RequestData>,
}

pub struct Server {
    db: SqliteDatabase,
}

impl Server {
    pub fn new(db: SqliteDatabase) -> Self {
        Self { db }
    }

    // Listen for native message from WebExtension in a loop
    pub fn listen(&self) {
        event_loop(|req_json: JSON| -> Result<(), errors::Error> {
            let req: Request = serde_json::from_value(req_json)?;

            let res: JSON = self.router(&req.method, req.data);

            write_output(io::stdout(), &res)
        });
    }

    // Route requests per the method
    fn router(&self, method: &str, data: Option<RequestData>) -> JSON {
        match method {
            "GET" => self.get(),
            "OPTIONS" => Server::option(),
            "POST" => match data {
                Some(data) => match data.bookmark {
                    Some(bookmark) => self.post(&bookmark),
                    _ => Server::unknown(),
                },
                _ => Server::unknown(),
            },
            "PUT" => match data {
                Some(data) => match data.bookmark {
                    Some(bookmark) => self.put(&bookmark),
                    _ => Server::unknown(),
                },
                _ => Server::unknown(),
            },
            "DELETE" => match data {
                Some(data) => match data.bookmark_id {
                    Some(bookmark_id) => self.delete(&bookmark_id),
                    _ => Server::unknown(),
                },
                _ => Server::unknown(),
            },
            _ => Server::unknown(),
        }
    }

    fn get(&self) -> JSON {
        let bookmarks = self.db.get_bookmarks();

        match bookmarks {
            Ok(bm) => {
                json!({
                    "success": true,
                    "bookmarks": bm,
                })
            }
            Err(err) => {
                json!({
                    "success": false,
                    "message": err.description(),
                })
            }
        }
    }

    fn option() -> JSON {
        json!({
            "success": true,
            "binaryVersion": VERSION,
        })
    }

    fn post(&self, bm: &Bookmark) -> JSON {
        let added = self.db.add_bookmark(&bm);

        json!({ "success": added.is_ok() })
    }

    fn put(&self, bm: &Bookmark) -> JSON {
        let updated: Result<(), ()> = if bm.id.is_some() {
            empty_result_err(self.db.update_bookmark(&bm))
        } else { Err(()) };

        json!({ "success": updated.is_ok() })
    }

    fn delete(&self, bm_id: &BookmarkId) -> JSON {
        let deleted = self.db.delete_bookmark(&bm_id);

        json!({ "success": deleted.is_ok() })
    }

    fn unknown() -> JSON {
        json!({
            "success": false,
            "message": "Unrecognised request type or bad request payload.",
        })
    }
}
