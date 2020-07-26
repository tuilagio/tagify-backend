extern crate reqwest;
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE /* USER_AGENT */};
use std::collections::HashMap;
// use std::fs::File;
// use std::io::prelude::*;
// use std::io::Read;
use crate::utils;
use bytes::Bytes;
use regex::Regex;

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
    pub google_storage_enable: String,
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

/* Retrieves object metadata */
// https://cloud.google.com/storage/docs/json_api/v1/objects/get?hl=en_US
// pub async fn get_object_from_bucket(
//     client: &reqwest::Client, bearer_string: &String,
//     bucket_name: &String, object_string: &String,
// ) -> Result<String, reqwest::Error> {
//     //TODO: object  not found (404, but now "error" in body: No such object: tagify_album_ss20_39/1.jpg)
//     let url = format!("https://storage.googleapis.com/storage/v1/b/{}/o/{}", &bucket_name, &object_string);
//     let res = client.get(&url)
//         .bearer_auth(&bearer_string)
//         .send()
//         .await?;
//     let body = res.text().await?;
//     Ok(body)
// }

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

/* Retrieves a list of objects matching the criteria */
// https://cloud.google.com/storage/docs/json_api/v1/objects/list?hl=en_US
// pub async fn get_all_objects_from_bucket(
//     client: &reqwest::Client, bearer_string: &String,
//     bucket_name: &String,
// ) -> Result<String, reqwest::Error> {

//     let url = format!("https://storage.googleapis.com/storage/v1/b/{}/o", &bucket_name);
//     let res = client.get(&url)
//         .bearer_auth(&bearer_string)
//         .send()
//         .await?;
//     let body = res.text().await?;
//     Ok(body)
// }

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
// pub async fn upload_file_to_bucket(
//     client: &reqwest::Client, bearer_string: &String,
//     bucket_name: &String, filepath: &String, object_name: &String,
// ) -> Result<String, reqwest::Error> {

//     let file = File::open(&filepath);
//     let mut f = match file {
//         Ok(file) => file,
//         Err(error) => panic!("Problem opening the file: {:?}", error),
//     };
//     let metadata = std::fs::metadata(&filepath).expect("unable to read metadata");
//     let mut buffer = vec![0; metadata.len() as usize];
//     f.read(&mut buffer).expect("buffer overflow");
//     let url = format!("https://storage.googleapis.com/upload/storage/v1/b/{}/o?uploadType=media&name={}", &bucket_name, &object_name);
//     let res = client.post(&url)
//         .bearer_auth(&bearer_string)
//         .headers(construct_headers_image(utils::get_file_ext(object_name)))
//         .body(buffer)
//         .send()
//         .await?;
//     let body = res.text().await?;
//     Ok(body)
// }

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

/* Download file. Mind the "?alt=media" URL parameter! */
// https://cloud.google.com/storage/docs/json_api/v1/objects/get?hl=en_US
// pub async fn download_file_from_bucket(
//     client: &reqwest::Client, bearer_string: &String,
//     bucket_name: &String, filepath: &String, object_name: &String,
// ) -> Result<String, reqwest::Error> {

//     let url = format!("https://storage.googleapis.com/storage/v1/b/{}/o/{}?alt=media", &bucket_name, &object_name);
//     let res = client.get(&url)
//         .bearer_auth(&bearer_string)
//         .headers(construct_headers_image(utils::get_file_ext(object_name)))
//         .send()
//         .await?;
//     let bytes = &res.bytes().await?;
//     // Write data
//     let buffer = File::create(filepath);
//     let mut b = match buffer {
//         Ok(buffer) => buffer,
//         Err(error) => panic!("Problem create the file: {:?}", error),
//     };

//     let mut pos = 0;
//     while pos < bytes.len() {
//         let bytes_written = b.write(&bytes[pos..]);
//         let bw = match bytes_written {
//             Ok(bytes_written) => bytes_written,
//             Err(error) => panic!("Problem writing the file: {:?}", error),
//         };
//         pos += bw;
//     }
//     Ok(filepath.to_string())
// }

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
