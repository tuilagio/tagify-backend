use serde::{Deserialize, Serialize};
use tokio_pg_mapper_derive::PostgresMapper;

use chrono::offset::Utc;
use chrono::{DateTime, TimeZone, NaiveDateTime};

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

#[derive(Debug, Clone, Serialize, Deserialize, PostgresMapper)]
#[pg_mapper(table = "albums")]
pub struct AlbumPreview {
    pub id: i32,
    pub title: String,
    pub description: String,
    pub first_photo: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlbumsPreview {
    pub  albums: Vec<AlbumPreview> ,
}

#[derive(Debug, Clone, Serialize, Deserialize, PostgresMapper)]
#[pg_mapper(table = "image_metas")]
pub struct PhotoPreview {
    pub id: i32,
    pub file_path: String
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagPhoto {
    pub tag: String,
    pub coordinates: String
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyPhoto {
    pub verified: bool
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhotoToTag {
    pub id: i32,
    pub file_path: String,
    pub tagged: bool,
   // pub timestamp: DateTime<Utc>
}