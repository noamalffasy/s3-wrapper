use chrono::{DateTime, Utc};

pub struct Bucket {
    pub name: String,
    pub region: String,
    pub creation_date: DateTime<Utc>,
}
