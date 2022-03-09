use mysql::prelude::Queryable;
use mysql_common::params;
use mysql_common::params::Params;
use mysql_common::row::Row;
use uuid::Uuid;
use crate::{DalResult, Datastore, Mysql, Track, Dal};

pub struct Queue<T: Datastore> {
    pub(crate) dal: T,
    pub room_uuid: Uuid,
}

struct IndexedTrack<T: Datastore> {
    track: Track<T>,
    idx: i64,
}

impl Queue<Mysql> {
    pub fn get_enqueued(&self) -> DalResult<Vec<Track<Mysql>>> {
        let mut conn = self.dal.get_conn()?;
        let rows: Vec<Row> = conn.exec("SELECT track_uuid,idx FROM queue WHERE room_uuid = :room_uuid", params! {
            "room_uuid" => &self.room_uuid
        })?;

        let mut tracks = rows.into_iter()
            .map(|x| {
                (x.get::<Uuid, &str>("track_uuid").unwrap(), x.get::<i64, &str>("idx").unwrap())
            })
            .map(|(uuid, idx)| Ok(IndexedTrack {
                track: Track::get(self.dal.clone(), uuid)?.unwrap(),
                idx
            }))
            .collect::<DalResult<Vec<_>>>()?;
        tracks.sort_by(|a, b| a.idx.cmp(&b.idx));

        let tracks = tracks.into_iter()
            .map(|x| x.track)
            .collect::<Vec<_>>();

        Ok(tracks)
    }

    pub fn enqueue(&self, track: &Track<Mysql>, added_by: &Uuid) -> DalResult<i64> {
        let mut conn = self.dal.get_conn()?;
        let idx = match conn.exec_first::<Row, &str, Params>("SELECT idx FROM queue WHERE room_uuid = :room_uuid ORDER BY idx DESC", params! {
            "room_uuid" => &self.room_uuid
        })? {
            Some(x) => x.get("idx").unwrap(),
            None => 0_i64
        };

        conn.exec_drop("INSERT INTO queue (track_uuid, room_uuid, idx, added_by) VALUES (:track_uuid, :room_uuid, :idx, :added_by)", params! {
            "track_uuid" => &track.uuid,
            "room_uuid" => &self.room_uuid,
            "idx" => idx + 1,
            "added_by" => added_by
        })?;

        Ok(idx + 1)
    }

    pub fn dequeue(&self, track: &Track<Mysql>) -> DalResult<()> {
        let mut conn = self.dal.get_conn()?;
        conn.exec_drop("DELETE FROM queue WHERE track_uuid = :track_uuid", params! {
            "track_uuid" => &track.uuid
        })?;
        Ok(())
    }
}