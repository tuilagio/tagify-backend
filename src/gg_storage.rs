extern crate reqwest;
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE /* USER_AGENT */};
use std::collections::HashMap;
// use std::fs::File;
// use std::io::prelude::*;
// use std::io::Read;
use crate::errors::HandlerError;
use crate::utils;
use bytes::Bytes;
use regex::Regex;

pub fn create_error(response: &str) -> HandlerError {
    let json: serde_json::Value = match serde_json::from_str(&response) {
        Ok(i) => i,
        Err(_) => return HandlerError::InternalError,
    };
    let err_msg = json["error"]["errors"][0]["message"].clone();

    return HandlerError::StorageError {
        err: err_msg.to_string(),
    };
}

fn construct_headers_image(ext: String) -> HeaderMap {
    let mut headers = HeaderMap::new();
    // TODO: this is stupid, but I couldn't findd a way to convert string to 'static string'
    let mut l_ext = ext;
    l_ext.make_ascii_lowercase();
    if l_ext == "png" {
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("image/png"));
    } else if l_ext == "bmp" {
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("image/bmp"));
    } else if l_ext == "jpg" || l_ext == "jpeg" {
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("image/jpeg"));
    } else if l_ext == "gif" {
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("image/gif"));
    } else if l_ext == "ico" {
        headers.insert(
            CONTENT_TYPE,
            HeaderValue::from_static("image/vnd.microsoft.icon"),
        );
    } else if l_ext == "svg" {
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("image/svg+xml"));
    } else if l_ext == "tif" || l_ext == "tiff" {
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("image/tiff"));
    } else if l_ext == "webp" {
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("image/webp"));
    } else {
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("image/png"));
    }
    headers
}

pub static PREFIX_BUCKET: &str = "tagify_album_ss20_";

#[derive(Clone)]
pub struct GoogleStorage {
    pub bearer_string: String,
    pub project_number: String,
    pub google_storage_enable: bool,
}

pub async fn get_bucket(
    client: &reqwest::Client,
    bearer_string: &String,
    bucket_name: &String,
) -> Result<String, reqwest::Error> {
    let url = format!(
        "https://storage.googleapis.com/storage/v1/b/{}",
        bucket_name
    );
    let res = client.get(&url).bearer_auth(&bearer_string).send().await?;
    let body = res.text().await?;
    return Ok(body);
}

// https://cloud.google.com/storage/docs/json_api/v1/buckets/delete?hl=en_US
pub async fn delete_bucket(
    client: &reqwest::Client,
    bearer_string: &String,
    bucket_name: &String,
) -> Result<String, reqwest::Error> {
    let url = format!(
        "https://storage.googleapis.com/storage/v1/b/{}",
        bucket_name
    );
    let res = client
        .delete(&url)
        .bearer_auth(&bearer_string)
        .send()
        .await?;
    let body = res.text().await?;
    return Ok(body);
}

/* https://cloud.google.com/storage/docs/json_api/v1/buckets/insert?hl=en_US */
pub async fn create_bucket(
    client: &reqwest::Client,
    bearer_string: &String,
    project_number: &String,
    bucket_name: &String,
) -> Result<String, reqwest::Error> {
    let loc = "EUROPE-WEST3".to_string();
    let mut map = HashMap::new();
    map.insert("name", bucket_name);
    map.insert("location", &loc);
    let url = format!(
        "https://storage.googleapis.com/storage/v1/b?project={}&predefinedAcl=private&predefinedDefaultObjectAcl=bucketOwnerFullControl&projection=full", 
        &project_number);
    let res = client
        .post(&url)
        .bearer_auth(&bearer_string)
        .json(&map)
        .send()
        .await?;
    let body = res.text().await?;
    return Ok(body);
}

/* Delete object (file) */
// https://cloud.google.com/storage/docs/json_api/v1/objects/delete?hl=en_US
pub async fn delete_object_from_bucket(
    client: &reqwest::Client,
    bearer_string: &String,
    bucket_name: &String,
    object_name: &String,
) -> Result<String, reqwest::Error> {
    let url = format!(
        "https://storage.googleapis.com/storage/v1/b/{}/o/{}",
        &bucket_name, &object_name
    );
    let res = client
        .delete(&url)
        .bearer_auth(&bearer_string)
        .send()
        .await?;
    let body = res.text().await?;
    Ok(body)
}

/* Retrieves a list of object names matching the criteria */
// https://cloud.google.com/storage/docs/json_api/v1/objects/list?hl=en_US
pub async fn get_all_object_names_from_bucket(
    client: &reqwest::Client,
    bearer_string: &String,
    bucket_name: &String,
) -> Result<Vec<String>, reqwest::Error> {
    let url = format!(
        "https://storage.googleapis.com/storage/v1/b/{}/o",
        &bucket_name
    );
    let res = client.get(&url).bearer_auth(&bearer_string).send().await?;
    let body = res.text().await?;
    let mut objectnames_bucket: Vec<String> = Vec::new();
    if !body.contains("error") {
        let re = Regex::new(r###""name":\s"([\w]+\.[\w]+)","###).unwrap();
        for cap in re.captures_iter(&body) {
            objectnames_bucket.push(cap[1].to_string());
        }
    }
    Ok(objectnames_bucket)
}

// https://cloud.google.com/storage/docs/json_api/v1/objects/insert?hl=en_US
pub async fn upload_buffer_with_name_to_bucket(
    client: &reqwest::Client,
    bearer_string: &String,
    bucket_name: &String,
    object_name: &String,
    buffer: Bytes,
) -> Result<String, reqwest::Error> {
    let url = format!(
        "https://storage.googleapis.com/upload/storage/v1/b/{}/o?uploadType=media&name={}",
        &bucket_name, &object_name
    );
    let res = client
        .post(&url)
        .bearer_auth(&bearer_string)
        .headers(construct_headers_image(utils::get_file_ext(object_name)))
        .body(buffer)
        .send()
        .await?;
    let body = res.text().await?;
    Ok(body)
}

/* Retrieves object bytes. Mind the "?alt=media" URL parameter! */
// https://cloud.google.com/storage/docs/json_api/v1/objects/get?hl=en_US
pub async fn download_object_bytes_from_bucket(
    client: &reqwest::Client,
    bearer_string: &String,
    bucket_name: &String,
    object_name: &String,
) -> Result<Bytes, reqwest::Error> {
    let url = format!(
        "https://storage.googleapis.com/storage/v1/b/{}/o/{}?alt=media",
        &bucket_name, &object_name
    );
    let res = client
        .get(&url)
        .bearer_auth(&bearer_string)
        .headers(construct_headers_image(utils::get_file_ext(object_name)))
        .send()
        .await?;
    let bytes = res.bytes().await?;
    Ok(bytes)
}
