use std::ops::Deref;
use mysql::{OptsBuilder, Pool};
use crate::{DalResult, Datastore};

mod migrations {
    use refinery::embed_migrations;
    embed_migrations!("./mysql_migrations");
}

#[derive(Clone, Debug)]
pub struct Mysql(Pool);

impl Mysql {
    pub fn new<S: AsRef<str>>(host: S, db: S, username: S, password: S) -> DalResult<Self> {
        let opts = OptsBuilder::new()
            .ip_or_hostname(Some(host.as_ref()))
            .db_name(Some(db.as_ref()))
            .user(Some(username.as_ref()))
            .pass(Some(password.as_ref()));
        let pool = Pool::new(opts)?;
        Ok(Self(pool))
    }

    pub fn migrate(&self) -> DalResult<()> {
        let mut conn = self.get_conn()?;
        migrations::migrations::runner().run(&mut conn)?;
        Ok(())
    }
}

impl Datastore for Mysql {}

impl Deref for Mysql {
    type Target = mysql::Pool;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}