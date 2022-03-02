use mysql::prelude::Queryable;
use mysql::TxOpts;
use mysql_common::params;
use mysql_common::row::Row;
use rand::Rng;
use crate::{uuid::Uuid, Dal, DalResult, Datastore, Mysql};

pub struct Member {
    pub uuid: Uuid,
    pub joined_at: i64,
}

pub enum RemoveStatus {
    Ok {
        new_owner: Uuid,
    },
    LastMember
}

pub trait RoomExt<T: Datastore, U>: Dal<T, U> {
    fn get_by_join_code<S: AsRef<str>>(dal: T, code: S) -> DalResult<Option<Self>>;
    fn add_user(&mut self, user: &Uuid) -> DalResult<()>;
    fn remove_user(&mut self, user: &Uuid) -> DalResult<RemoveStatus>;
    fn list_members(&self) -> DalResult<Vec<Member>>;
}

pub struct Room<T: Datastore> {
    dal: T,
    pub uuid: Uuid,
    pub owner: Uuid,
    pub name: String,
    pub join_code: String,
}

pub struct RoomBuildable {
    pub user_owner: Uuid,
    pub name: String,
}

const JOIN_CODE_LENGTH: usize = 6;

pub fn generate_join_code() -> String {
    rand::thread_rng().sample_iter(rand::distributions::Alphanumeric).take(JOIN_CODE_LENGTH).map(char::from).collect::<String>().to_uppercase()
}

impl Dal<Mysql, RoomBuildable> for Room<Mysql> {
    fn get(dal: Mysql, uuid: Uuid) -> DalResult<Option<Self>> {
        let mut tx = dal.start_transaction(TxOpts::default())?;
        let row: Row = match tx.exec_first("SELECT name,owner,join_code FROM rooms WHERE uuid = :uuid", params! {
            "uuid" => &uuid
        })? {
            Some(x) => x,
            None => return Ok(None)
        };

        let name: String = row.get("name").unwrap();
        let owner: Uuid = row.get("owner").unwrap();
        let join_code: String = row.get("join_code").unwrap();

        Ok(Some(Self {
            dal,
            uuid,
            owner,
            name,
            join_code,
        }))
    }

    fn delete(self) -> DalResult<()> {
        let mut tx = self.dal.start_transaction(TxOpts::default())?;
        tx.exec_drop("DELETE FROM rooms WHERE uuid = :uuid", params! {
            "uuid" => &self.uuid
        })?;
        tx.commit()?;

        Ok(())
    }

    fn update(&mut self) -> DalResult<()> {
        todo!()
    }

    fn create(dal: Mysql, buildable: RoomBuildable) -> DalResult<Self> {
        let mut tx = dal.start_transaction(TxOpts::default())?;

        let uuid = Uuid::new_v4();

        let mut join_code = generate_join_code();
        while let Some(_) = Self::get_by_join_code(dal.clone(), &join_code)? {
            join_code = generate_join_code();
        }

        tx.exec_drop("INSERT INTO rooms (name,uuid,owner,join_code) VALUES (:name, :uuid, :owner,:join_code)", params! {
            "name" => &buildable.name,
            "uuid" => &uuid,
            "owner" => &buildable.user_owner,
            "join_code" => &join_code,
        })?;
        tx.commit()?;
        Ok(Self {
            dal,
            uuid,
            name: buildable.name,
            owner: buildable.user_owner,
            join_code,
        })
    }
}

impl RoomExt<Mysql, RoomBuildable> for Room<Mysql> {
    fn get_by_join_code<S: AsRef<str>>(dal: Mysql, code: S) -> DalResult<Option<Self>> {
        let mut conn = dal.get_conn()?;
        let row: Row = match conn.exec_first("SELECT uuid FROM rooms WHERE join_code = :join_code", params! {
            "join_code" => code.as_ref()
        })? {
            Some(x) => x,
            None => return Ok(None)
        };

        let uuid: Uuid = row.get("uuid").unwrap();
        Self::get(dal, uuid)
    }

    fn add_user(&mut self, user: &Uuid) -> DalResult<()> {
        let mut conn = self.dal.get_conn()?;
        conn.exec_drop("INSERT INTO room_members (room_uuid, user_uuid, joined_at) VALUES (:room_uuid, :user_uuid, :joined_at)", params! {
            "room_uuid" => &self.uuid,
            "user_uuid" => user,
            "joined_at" => time::OffsetDateTime::now_utc().unix_timestamp()
        })?;
        Ok(())
    }

    fn remove_user(&mut self, user: &Uuid) -> DalResult<RemoveStatus> {
        let mut tx = self.dal.start_transaction(TxOpts::default())?;

        tx.exec_drop("DELETE FROM room_members WHERE room_uuid = :room_uuid AND user_uuid = :user_uuid", params! {
            "room_uuid" => &self.uuid,
            "user_uuid" => user
        })?;

        // Select a new owner
        let mut remaining = self.list_members()?
            .into_iter()
            .filter(|x| x.uuid.ne(user))
            .collect::<Vec<_>>();
        remaining.sort_by(|a, b| a.joined_at.cmp(&b.joined_at));
        let remove_status = if let Some(first) = remaining.first() {
            tx.exec_drop("UPDATE rooms SET owner = :owner WHERE uuid = :uuid", params! {
                "owner" => first.uuid,
                "uuid" => self.uuid
            })?;

            RemoveStatus::Ok {
                new_owner: first.uuid
            }
        } else {
            RemoveStatus::LastMember
        };

        tx.commit()?;

        Ok(remove_status)
    }

    fn list_members(&self) -> DalResult<Vec<Member>> {
        let mut conn = self.dal.get_conn()?;
        let rows: Vec<Row> = conn.exec("SELECT user_uuid,joined_at FROM room_members WHERE room_uuid = :room_uuid", params! {
            "room_uuid" => &self.uuid
        })?;

        let members = rows.into_iter()
            .map(|x| Member {
                uuid: x.get("user_uuid").unwrap(),
                joined_at: x.get("joined_at").unwrap(),
            })
            .collect::<Vec<_>>();
        Ok(members)
    }
}