mod user;

use std::ops::Deref;
pub use user::*;

use uuid::Uuid;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Mysql error {0}")]
    Mysql(#[from] mysql::Error),
}

pub type DalResult<T> = Result<T, Error>;

pub trait Datastore: Sized {}

pub trait Dal<T: Datastore, U>: Sized {
    fn get(data: T, uuid: Uuid) -> DalResult<Option<Self>>;
    fn delete(self) -> DalResult<()>;
    fn update(&mut self) -> DalResult<()>;
    fn create(data: T, buildable: U) -> DalResult<Self>;
}

pub struct Mysql(mysql::Pool);

impl Datastore for Mysql {}

impl Deref for Mysql {
    type Target = mysql::Pool;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}