pub mod db;
pub mod models;

pub use db::connection::{AppDatabase, Database};
pub use db::error::DbError;
