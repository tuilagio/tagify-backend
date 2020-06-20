use serde::{Deserialize, Serialize};
use tokio_pg_mapper_derive::PostgresMapper;

#[derive(Debug, Clone, Serialize, Deserialize, PostgresMapper)]
#[pg_mapper(table = "albums")]
pub struct Album {
    pub id: i32,
    pub title: String,
    pub description: String,
    pub tags: Vec<String>,
    pub image_number: i32,
    pub tagged_number: i32,
    pub users_id: i32,
    pub first_photo: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreateAlbum {
    pub title: String,
    pub description: String,
    pub tags: Vec<String>,
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UpdateAlbum {
    pub title: String,
    pub description: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AlbumTag {
    pub tags_id: i32,
    pub albums_id: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Tag {
    pub tag: String,
}
