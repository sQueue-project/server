use mysql::prelude::Queryable;
use mysql_common::params;
use mysql_common::row::Row;
use uuid::Uuid;
use crate::{Dal, DalResult, Datastore, Error, Mysql};

#[derive(Clone)]
pub enum PretrackSourcePlatform {
    YouTube(String),
    Spotify(String)
}

impl PretrackSourcePlatform {
    pub fn new(platform: String, track_id: String) -> Option<Self> {
        match platform.as_str() {
            "YouTube" => Some(Self::YouTube(track_id)),
            "Spotify" => Some(Self::Spotify(track_id)),
            _ => None
        }
    }

    pub fn get_track_id(&self) -> String {
        match self {
            Self::YouTube(s) => s.clone(),
            Self::Spotify(s) => s.clone(),
        }
    }
}

impl ToString for PretrackSourcePlatform {
    fn to_string(&self) -> String {
        match self {
            Self::YouTube(_) => "YouTube".into(),
            Self::Spotify(_) => "Spotify".into(),
        }
    }
}

pub struct Pretrack<T: Datastore> {
    dal: T,
    pub uuid: Uuid,
    pub name: String,
    pub artist: String,
    pub duration: i64,
    pub thumbnail_url: String,
    pub platform: PretrackSourcePlatform
}

impl Dal<Mysql, PretrackBuildable> for Pretrack<Mysql> {
    fn get(dal: Mysql, uuid: Uuid) -> DalResult<Option<Self>> {
        let mut conn = dal.get_conn()?;
        let row: Row = match conn.exec_first("SELECT name,artist,duration,thumbnail_url,platform,platform_track_id FROM pretracks WHERE uuid = :uuid", params! {
            "uuid" => &uuid
        })? {
            Some(x) => x,
            None => return Ok(None)
        };

        let platform_name: String = row.get("platform").unwrap();
        let platform_track_id: String = row.get("platform_track_id").unwrap();
        let platform = PretrackSourcePlatform::new(platform_name.clone(), platform_track_id)
            .ok_or(Error::Other(format!("Unknown platform '{platform_name}'")))?;

        Ok(Some(Self {
            dal,
            uuid,
            name: row.get("name").unwrap(),
            artist: row.get("artist").unwrap(),
            duration: row.get("duration").unwrap(),
            thumbnail_url: row.get("thumbnail_url").unwrap(),
            platform,
        }))
    }

    fn delete(self) -> DalResult<()> {
        let mut conn = self.dal.get_conn()?;
        conn.exec_drop("DELETE FROM pretracks WHERE uuid = :uuid", params! {
            "uuid" => self.uuid
        })?;

        Ok(())
    }

    fn update(&mut self) -> DalResult<()> {
        todo!()
    }

    fn create(dal: Mysql, buildable: PretrackBuildable) -> DalResult<Self> {
        let mut conn = dal.get_conn()?;
        let uuid = Uuid::new_v4();
        conn.exec_drop("INSERT INTO pretracks (uuid, name, artist, duration, thumbnail_url, platform, platform_track_id) VALUES (:uuid, :name, :artist, :duration, :thumbnail_url, :platform, :platform_track_id)", params! {
            "uuid" => &uuid,
            "name" => &buildable.name,
            "artist" => &buildable.artist,
            "duration" => buildable.duration,
            "thumbnail_url" => &buildable.thumbnail_url,
            "platform" => buildable.platform.to_string(),
            "platform_track_id" => buildable.platform.get_track_id(),
        })?;

        Ok(Self {
            dal,
            uuid,
            name: buildable.name,
            artist: buildable.artist,
            duration: buildable.duration,
            thumbnail_url: buildable.thumbnail_url,
            platform: buildable.platform
        })
    }
}

pub struct PretrackBuildable {
    pub name: String,
    pub artist: String,
    pub duration: i64,
    pub thumbnail_url: String,
    pub platform: PretrackSourcePlatform,
}