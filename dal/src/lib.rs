mod user;
mod room;
mod mysql_dal;

pub use user::*;
pub use room::*;
pub use mysql_dal::*;

pub mod uuid {
    pub use ::uuid::Uuid;
    pub use ::uuid::Error;
}

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Mysql error: {0}")]
    Mysql(#[from] mysql::Error),
    #[error("Refinery error: {0}")]
    Refinery(#[from] refinery::Error),
}

pub type DalResult<T> = Result<T, Error>;

pub trait Datastore: Sized {}

pub trait Dal<T: Datastore, U>: Sized {
    fn get(dal: T, uuid: crate::uuid::Uuid) -> DalResult<Option<Self>>;
    fn delete(self) -> DalResult<()>;
    fn update(&mut self) -> DalResult<()>;
    fn create(dal: T, buildable: U) -> DalResult<Self>;
}