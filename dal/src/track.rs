use mysql::prelude::Queryable;
use mysql_common::params;
use mysql_common::row::Row;
use uuid::Uuid;
use crate::{Dal, DalResult, Datastore, Error, Mysql, Pretrack, PretrackSourcePlatform};

pub struct Track<T: Datastore> {
    dal: T,
    pub uuid: Uuid,
    pub room_uuid: Uuid,
    pub name: String,
    pub artist: String,
    pub duration: i64,
    pub thumbnail_url: String,
    pub platform: PretrackSourcePlatform,
}

pub struct TrackBuildable<T: Datastore> {
    pub pretrack: Pretrack<T>,
    pub room_uuid: Uuid
}

pub trait TrackExt<T: Datastore> {
    fn get_queue_idx(&self) -> DalResult<i64>;
}

impl Dal<Mysql, TrackBuildable<Mysql>> for Track<Mysql> {
    fn get(dal: Mysql, uuid: Uuid) -> DalResult<Option<Self>> {
        let mut conn = dal.get_conn()?;
        let row: Row = match conn.exec_first("SELECT room_uuid,name,artist,duration,thumbnail_url,platform,platform_video_id FROM tracks WHERE uuid = :uuid", params! {
            "uuid" => &uuid
        })? {
            Some(x) => x,
            None => return Ok(None)
        };

        let platform_name: String = row.get("platform").unwrap();
        let platform_track_id: String = row.get("platform_video_id").unwrap();
        let platform = PretrackSourcePlatform::new(platform_name.clone(), platform_track_id)
            .ok_or(Error::Other(format!("Unknown platform '{platform_name}'")))?;

        Ok(Some(Self {
            dal,
            uuid,
            room_uuid: row.get("room_uuid").unwrap(),
            name: row.get("name").unwrap(),
            artist: row.get("artist").unwrap(),
            duration: row.get("duration").unwrap(),
            thumbnail_url: row.get("thumbnail_url").unwrap(),
            platform
        }))
    }

    fn delete(self) -> DalResult<()> {
        let mut conn = self.dal.get_conn()?;
        conn.exec_drop("DELETE FROM tracks WHERE uuid = :uuid", params! {
            "uuid" => self.uuid
        })?;

        Ok(())
    }

    fn update(&mut self) -> DalResult<()> {
        todo!()
    }

    fn create(dal: Mysql, buildable: TrackBuildable<Mysql>) -> DalResult<Self> {
        let mut conn = dal.get_conn()?;
        let uuid = Uuid::new_v4();
        conn.exec_drop("INSERT INTO tracks (uuid, room_uuid, name, artist, duration, thumbnail_url, platform, platform_video_id) VALUES (:uuid, :room_uuid, :name, :artist, :duration, :thumbnail_url, :platform, :platform_video_id)", params! {
            "uuid" => &uuid,
            "room_uuid" => &buildable.room_uuid,
            "name" => &buildable.pretrack.name,
            "artist" => &buildable.pretrack.artist,
            "duration" => &buildable.pretrack.duration,
            "thumbnail_url" => &buildable.pretrack.thumbnail_url,
            "platform" => &buildable.pretrack.platform.to_string(),
            "platform_video_id" => &buildable.pretrack.platform.get_track_id(),
        })?;

        let this = Self {
            dal,
            uuid,
            room_uuid: buildable.room_uuid.clone(),
            name: buildable.pretrack.name.clone(),
            artist: buildable.pretrack.artist.clone(),
            duration: buildable.pretrack.duration,
            thumbnail_url: buildable.pretrack.thumbnail_url.clone(),
            platform: buildable.pretrack.platform.clone()
        };

        buildable.pretrack.delete()?;

        Ok(this)
    }
}

impl TrackExt<Mysql> for Track<Mysql> {
    fn get_queue_idx(&self) -> DalResult<i64> {
        let mut conn = self.dal.get_conn()?;
        let row: Row = match conn.exec_first("SELECT idx FROM queue WHERE track_uuid = :track_uuid", params! {
            "track_uuid" => &self.uuid
        })? {
            Some(x) => x,
            None => return Ok(-1)
        };

        let pos: i64 = row.get("idx").unwrap();
        Ok(pos)
    }
}