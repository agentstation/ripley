use std::path::Path;

use redb::{Database, ReadableDatabase, ReadableTable, TableDefinition};

use crate::feed::Advisory;
use crate::types::Ecosystem;

const ADVISORIES: TableDefinition<&str, &[u8]> = TableDefinition::new("advisories");
const META: TableDefinition<&str, u64> = TableDefinition::new("meta");

#[derive(Debug, thiserror::Error)]
#[allow(clippy::result_large_err)]
pub enum DbError {
    #[error("database error: {0}")]
    Redb(#[from] redb::Error),
    #[error("database error: {0}")]
    Database(#[from] redb::DatabaseError),
    #[error("database storage error: {0}")]
    Storage(#[from] redb::StorageError),
    #[error("database transaction error: {0}")]
    Transaction(#[from] redb::TransactionError),
    #[error("database commit error: {0}")]
    Commit(#[from] redb::CommitError),
    #[error("database table error: {0}")]
    Table(#[from] redb::TableError),
    #[error("serialization error: {0}")]
    Serde(#[from] serde_json::Error),
}

pub struct AdvisoryDb {
    db: Database,
}

#[allow(clippy::result_large_err)]
impl AdvisoryDb {
    pub fn open(path: &Path) -> Result<Self, DbError> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| DbError::Redb(redb::Error::Io(e)))?;
        }
        let db = match Database::create(path) {
            Ok(db) => db,
            Err(redb::DatabaseError::UpgradeRequired(_)) if path.exists() => {
                // Advisory cache from an older redb version — safe to discard
                std::fs::remove_file(path).map_err(|e| DbError::Redb(redb::Error::Io(e)))?;
                Database::create(path)?
            }
            Err(e) => return Err(e.into()),
        };

        let txn = db.begin_write()?;
        {
            let _ = txn.open_table(ADVISORIES)?;
            let _ = txn.open_table(META)?;
        }
        txn.commit()?;

        Ok(Self { db })
    }

    pub fn store_advisories(
        &self,
        ecosystem: Ecosystem,
        package: &str,
        advisories: &[Advisory],
    ) -> Result<(), DbError> {
        let key = format!("{ecosystem}:{package}");
        let value = serde_json::to_vec(advisories)?;

        let txn = self.db.begin_write()?;
        {
            let mut table = txn.open_table(ADVISORIES)?;
            table.insert(key.as_str(), value.as_slice())?;
        }
        txn.commit()?;

        Ok(())
    }

    pub fn get_advisories(
        &self,
        ecosystem: Ecosystem,
        package: &str,
    ) -> Result<Vec<Advisory>, DbError> {
        let key = format!("{ecosystem}:{package}");

        let txn = self.db.begin_read()?;
        let table = txn.open_table(ADVISORIES)?;

        match table.get(key.as_str())? {
            Some(value) => {
                let advisories: Vec<Advisory> = serde_json::from_slice(value.value())?;
                Ok(advisories)
            }
            None => Ok(Vec::new()),
        }
    }

    pub fn get_all_advisories(&self) -> Result<Vec<Advisory>, DbError> {
        let txn = self.db.begin_read()?;
        let table = txn.open_table(ADVISORIES)?;

        let mut all = Vec::new();
        let iter = table.iter()?;
        for entry in iter {
            let entry = entry.map_err(DbError::Storage)?;
            let advisories: Vec<Advisory> = serde_json::from_slice(entry.1.value())?;
            all.extend(advisories);
        }

        Ok(all)
    }

    pub fn get_last_poll(&self) -> Result<Option<u64>, DbError> {
        let txn = self.db.begin_read()?;
        let table = txn.open_table(META)?;

        match table.get("last_poll")? {
            Some(value) => Ok(Some(value.value())),
            None => Ok(None),
        }
    }

    pub fn set_last_poll(&self, timestamp: u64) -> Result<(), DbError> {
        let txn = self.db.begin_write()?;
        {
            let mut table = txn.open_table(META)?;
            table.insert("last_poll", timestamp)?;
        }
        txn.commit()?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::feed::{AffectedRange, FeedSource};
    use crate::types::Severity;

    fn sample_advisory() -> Advisory {
        Advisory {
            id: "GHSA-test-1234".to_string(),
            source: FeedSource::Osv,
            ecosystem: Ecosystem::Npm,
            package: "lodash".to_string(),
            affected_ranges: vec![AffectedRange {
                introduced: semver::Version::new(0, 0, 0),
                fixed: Some(semver::Version::new(4, 17, 21)),
            }],
            severity: Some(Severity::High),
            summary: "Prototype pollution".to_string(),
            references: vec!["https://example.com".to_string()],
        }
    }

    #[test]
    fn test_store_and_retrieve() {
        let dir = tempfile::tempdir().expect("create tempdir");
        let db = AdvisoryDb::open(&dir.path().join("test.redb")).expect("open db");

        let advisory = sample_advisory();
        db.store_advisories(Ecosystem::Npm, "lodash", std::slice::from_ref(&advisory))
            .expect("store");

        let retrieved = db.get_advisories(Ecosystem::Npm, "lodash").expect("get");
        assert_eq!(retrieved.len(), 1);
        assert_eq!(retrieved[0].id, "GHSA-test-1234");
        assert_eq!(retrieved[0].package, "lodash");
    }

    #[test]
    fn test_last_poll() {
        let dir = tempfile::tempdir().expect("create tempdir");
        let db = AdvisoryDb::open(&dir.path().join("test.redb")).expect("open db");

        assert_eq!(db.get_last_poll().expect("get"), None);

        db.set_last_poll(1700000000).expect("set");
        assert_eq!(db.get_last_poll().expect("get"), Some(1700000000));

        db.set_last_poll(1700001000).expect("set again");
        assert_eq!(db.get_last_poll().expect("get"), Some(1700001000));
    }

    #[test]
    fn test_empty_db_returns_empty() {
        let dir = tempfile::tempdir().expect("create tempdir");
        let db = AdvisoryDb::open(&dir.path().join("test.redb")).expect("open db");

        let advisories = db
            .get_advisories(Ecosystem::Npm, "nonexistent")
            .expect("get");
        assert!(advisories.is_empty());

        let all = db.get_all_advisories().expect("get all");
        assert!(all.is_empty());
    }

    #[test]
    fn test_get_all_advisories() {
        let dir = tempfile::tempdir().expect("create tempdir");
        let db = AdvisoryDb::open(&dir.path().join("test.redb")).expect("open db");

        let a1 = sample_advisory();
        let mut a2 = sample_advisory();
        a2.id = "GHSA-other-5678".to_string();
        a2.package = "express".to_string();

        db.store_advisories(Ecosystem::Npm, "lodash", &[a1])
            .expect("store");
        db.store_advisories(Ecosystem::Npm, "express", &[a2])
            .expect("store");

        let all = db.get_all_advisories().expect("get all");
        assert_eq!(all.len(), 2);
    }
}
