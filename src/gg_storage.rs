extern crate reqwest;
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE, /* USER_AGENT */};
use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::io::Read;

fn construct_headers() -> HeaderMap {
    let mut headers = HeaderMap::new();
    // headers.insert(USER_AGENT, HeaderValue::from_static("reqwest"));
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("image/png"));
    headers
}

// pub let prefix_bucket: String = "tagify_album_ss20_".to_string();
// pub PREFIX_BUCKET: &'static String = &"tagify_album_ss20_".to_string();
pub static PREFIX_BUCKET: &str = "tagify_album_ss20_";

#[derive(Clone)]
pub struct GoogleStorage {
    pub key_refresh_token: String,
    pub bearer_string: String,
    pub project_number: String,
    pub google_storage_enable: String,
}

pub async fn get_bucket(
    client: &reqwest::Client,
    bearer_string: &String, bucket_name: &String, 
) -> Result<String, reqwest::Error> {

    let url = format!("https://storage.googleapis.com/storage/v1/b/{}", bucket_name);
    let res = client
        .get(&url)
        .bearer_auth(&bearer_string)
        .send()
        .await?;
    let body = res.text().await?;
    return Ok(body);
}

// https://cloud.google.com/storage/docs/json_api/v1/buckets/delete?hl=en_US
pub async fn delete_bucket(
    client: &reqwest::Client,
    bearer_string: &String, bucket_name: &String, 
) -> Result<String, reqwest::Error> {

    let url = format!("https://storage.googleapis.com/storage/v1/b/{}", bucket_name);
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
    client: &reqwest::Client, bearer_string: &String, key_refresh_token: &String,
    project_number: &String, bucket_name: &String, 
) -> Result<String, reqwest::Error> {
    
    let loc = "EUROPE-WEST3".to_string();
    let mut map = HashMap::new();
    map.insert("name", bucket_name);
    map.insert("location", &loc);
    let url = format!(
        "https://storage.googleapis.com/storage/v1/b?project={}&predefinedAcl=private&predefinedDefaultObjectAcl=bucketOwnerFullControl&projection=full&key={}", 
        &project_number, &key_refresh_token);
    let res = client
        .post(&url)
        .bearer_auth(&bearer_string)
        .json(&map)
        .send()
        .await?;
    let body = res.text().await?;
    // print!("{:?}", body);
    return Ok(body);
}

/* Retrieves object metadata */
// https://cloud.google.com/storage/docs/json_api/v1/objects/get?hl=en_US
pub async fn get_object_from_bucket(
    client: &reqwest::Client, bearer_string: &String,
    object_string: &String, bucket_name: &String, 
) -> Result<String, reqwest::Error> {

    let url = format!("https://storage.googleapis.com/storage/v1/b/{}/o/{}", &bucket_name, &object_string);
    let res = client.get(&url)
        .bearer_auth(&bearer_string)
        .send()
        .await?;
    let body = res.text().await?;
    // print!("{:?}", body3);
    Ok(body)
}

/* Retrieves a list of objects matching the criteria */
// https://cloud.google.com/storage/docs/json_api/v1/objects/list?hl=en_US
pub async fn get_all_object_from_bucket(
    client: &reqwest::Client, bearer_string: &String,
    bucket_name: &String, 
) -> Result<String, reqwest::Error> {

    let url = format!("https://storage.googleapis.com/storage/v1/b/{}/o", &bucket_name);
    let res = client.get(&url)
        .bearer_auth(&bearer_string)
        .send()
        .await?;
    let body = res.text().await?;
    Ok(body)
}

// https://cloud.google.com/storage/docs/json_api/v1/objects/insert?hl=en_US
pub async fn upload_file_to_bucket(
    client: &reqwest::Client, bearer_string: &String,
    bucket_name: &String, filepath: &String, object_name: &String, 
) -> Result<String, reqwest::Error> {

    let file = File::open(&filepath);
    let mut f = match file {
        Ok(file) => file,
        Err(error) => panic!("Problem opening the file: {:?}", error),
    };
    let metadata = std::fs::metadata(&filepath).expect("unable to read metadata");
    let mut buffer = vec![0; metadata.len() as usize];
    f.read(&mut buffer).expect("buffer overflow");
    let url = format!("https://storage.googleapis.com/upload/storage/v1/b/{}/o?uploadType=media&name={}", &bucket_name, &object_name);
    let res = client.post(&url)
        .bearer_auth(&bearer_string)
        .headers(construct_headers())
        .body(buffer)
        .send()
        .await?;
    // print!("\n##############\n{:?}\n", &res);
    let body = res.text().await?;
    // print!("\n##############\n{:?}\n", &body);
    Ok(body)
}

/* Retrieves object metadata AND download file. Mind the "?alt=media" URL parameter! */
// https://cloud.google.com/storage/docs/json_api/v1/objects/get?hl=en_US
pub async fn download_file_from_bucket(
    client: &reqwest::Client, bearer_string: &String,
    bucket_name: &String, filepath: &String, object_name: &String, 
) -> Result<String, reqwest::Error> {

    // let object_name = "salt3.png";
    let url = format!("https://storage.googleapis.com/storage/v1/b/{}/o/{}?alt=media", &bucket_name, &object_name);
    // print!("{}\n", url);
    let res = client.get(&url)
        .bearer_auth(&bearer_string)
        .headers(construct_headers())
        .send()
        .await?;
    print!("\n##############\n{:?}\n", &res);
    let bytes = &res.bytes().await?;
    // print!("{:}", bytes.len());
    // Write data
    let buffer = File::create(filepath);
    let mut b = match buffer {
        Ok(buffer) => buffer,
        Err(error) => panic!("Problem create the file: {:?}", error),
    };

    let mut pos = 0;
    while pos < bytes.len() {
        let bytes_written = b.write(&bytes[pos..]);
        let bw = match bytes_written {
            Ok(bytes_written) => bytes_written,
            Err(error) => panic!("Problem writing the file: {:?}", error),
        };
        pos += bw;
    }
    Ok(filepath.to_string())
}



// #[tokio::main]
// pub async fn main() -> Result<(), reqwest::Error> {
//     /* How to create project: */
//     // - This link _may_ work: https://console.cloud.google.com/projectcreate?previousPage=%2Fhome%2Fdashboard%3Fproject%3Ddt-project-01&folder=&organizationId=0
//     //  - After create project under "Project infor" board you can get all information. Copy "Project number" replace to "project_number" variable.

//     /* How to get token: */
//     // - Go to  https://developers.google.com/oauthplayground/
//     // - Select "Cloud Storage JSON API v1" ... 
//     // - ... and then select "https://www.googleapis.com/auth/devstorage.full_control".
//     // - Authorize APIs. Allow OAuth 2.0 Playground to have control.
//     // - There will be 3 fields: "Authorization code", "Refresh token" and "Access token".
//     // - Click "Exchange authorization code for tokens"
//     // - Copy  "Refresh token" and "Access token" and replace value of variables "key_refresh_token"  and  "bearer_string"

//     // Should look like "1//041_LvYC-DOorCgYIARAAGAQSNwF-L9srgfrdh5reu4w6hgerdg3ez6riz8uktzhjztuthrtzhftgewraA8BTTLZ9Kfzjheg5egdGw"
//     let key_refresh_token = "".to_string();
//     // ... "ya29.a0AfH6SMDfLmtz9GdzAhN2xkeTwgQpYnhfhrsdg4wet-kPTeE8txEGgdv6bYb4oqdFXPmxQer5dtrhgrtzrth6CSWx3NGYrzj7tu4z5437z56uthe5z345tewrferzh65uget34t3eiRZYWFIH0"
//     let bearer_string = "".to_string();
//     // ... 726421344513
//     let project_number  = "".to_string();

//     /* What do those code do? */
//     // - Create, get bucket. Then upload file to this bucket, and download it.
//     // - object_name Name  that file will be saved under in the cloud.
//     // - filepath Path to  local file to be upload.
//     // - filepath_download Path to download file to.
//     // The path should work out of the box.

//     let bucket_name = "qwertz013".to_string();
//     let object_name = "salt_object.png".to_string();
//     let filepath = "./salt.png".to_string();
//     let filepath_download = "./salt_download.png".to_string();

//     let client = reqwest::Client::new();

//     print!("\n#########\nCreate bucket\n#########\n");
//     print!("{:?}", create_bucket(&client, &bearer_string, &key_refresh_token, &project_number, &bucket_name).await?);
//     print!("\n#########\nGet bucket\n#########\n");
//     print!("{:?}", get_bucket(&client, &bearer_string, &bucket_name).await?);
//     print!("\n#########\nupload_file_to_bucket\n#########\n");
//     print!("{:?}", upload_file_to_bucket(&client, &bearer_string, &bucket_name, &filepath, &object_name).await?);
//     print!("\n#########\ndownload_file_from_bucket\n#########\n");
//     print!("{:?}", download_file_from_bucket(&client, &bearer_string, &bucket_name, &filepath_download, &object_name).await?);

//     Ok(())
// }
