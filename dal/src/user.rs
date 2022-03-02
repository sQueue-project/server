use mysql::prelude::Queryable;
use mysql::{params, TxOpts};
use mysql_common::row::Row;
use crate::{Uuid, Dal, DalResult, Datastore, Mysql};

pub struct User<T: Datastore> {
    dal: T,
    pub uuid: Uuid,
    pub name: String,
}

pub struct UserBuildable {
    pub name: String,
}

impl Dal<Mysql, UserBuildable> for User<Mysql> {
    fn get(dal: Mysql, uuid: Uuid) -> DalResult<Option<Self>> {
        let mut tx = dal.start_transaction(TxOpts::default())?;
        let row: Row = match tx.exec_first("SELECT name FROM users WHERE uuid = :uuid", params! {
            "uuid" => &uuid
        })? {
            Some(x) => x,
            None => return Ok(None)
        };
        tx.commit()?;

        Ok(Some(Self {
            dal,
            uuid,
            name: row.get("name").unwrap(),
        }))
    }

    fn delete(self) -> DalResult<()> {
        let mut tx = self.dal.start_transaction(TxOpts::default())?;
        tx.exec_drop("DELETE FROM users WHERE uuid = :uuid", params! {
            "uuid" => &self.uuid
        })?;
        tx.commit()?;
        Ok(())
    }

    fn update(&mut self) -> DalResult<()> {
        let mut tx = self.dal.start_transaction(TxOpts::default())?;

        tx.exec_drop("UPDATE users SET name = :name WHERE uuid = :uuid", params! {
            "name" => &self.name,
            "uuid" => &self.uuid
        })?;
        tx.commit()?;
        Ok(())
    }

    fn create(dal: Mysql, buildable: UserBuildable) -> DalResult<Self> {
        let mut tx = dal.start_transaction(TxOpts::default())?;
        let uuid = Uuid::new_v4();

        tx.exec_drop("INSERT INTO users (uuid, name) VALUES (:uuid, :name)", params! {
            "uuid" => &uuid,
            "name" => &buildable.name
        })?;
        tx.commit()?;
        Ok(Self {
            dal,
            uuid,
            name: buildable.name
        })
    }
}