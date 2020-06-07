use serde::{Deserialize, Serialize};
use tokio_pg_mapper_derive::PostgresMapper;

#[derive(Debug, Serialize, Deserialize, PostgresMapper)]
#[pg_mapper(table = "todos")]
pub struct Todo {
    pub id: Option<i32>,
    pub description: String,
    pub date: String,
    pub progress: i32,
}
