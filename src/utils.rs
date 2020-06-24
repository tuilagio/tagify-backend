// use crate::models::Roles;
use std::io::Write;

use actix_multipart::Multipart;
use actix_web::{middleware, web, App, Error, HttpResponse, HttpServer};
use futures::{StreamExt, TryStreamExt};
use std::convert::TryFrom;
use std::path::Path;
use std::fs;
use log::{error, info};

// pub fn validate_role(role: &str) -> bool {
//     let mut is_role = false;
//     for curr in Roles.iter() {
//         if **curr == user.role {
//             is_role = true;
//         }
//     }
//     is_role
// }

pub fn get_filenames_in_folder(folder_path: &str) -> Vec<String> {

    let mut filenames_folder = Vec::new();
    
    let paths = fs::read_dir(folder_path).unwrap();
    let mut names =
    paths.filter_map(|entry| {
        entry.ok().and_then(|e|
            e.path().file_name()
            .and_then(|n| n.to_str().map(|s| String::from(s)))
        )
    }).collect::<Vec<String>>();
    
    if names.len() != 0 {
        names.sort();
    
        for name in names.iter().rev() {
            filenames_folder.push(name.clone());
        }

    }
    return filenames_folder;
}

pub fn calculate_next_filename_image (
    filenames_folder: &Vec<String>, filenames_db: &Vec<String>
) -> String {
    let mut max_i: u32 = 0;
    
    let mut fn_folder_num: Vec<u32> = Vec::new();
    for filename in filenames_folder {
        let vec: Vec<&str> = filename.split(".").collect();
        let fname: &str = vec[0];
        if fname.parse::<u32>().is_ok() {
            let num = fname.parse().unwrap();
            fn_folder_num.push(num);
            if max_i < num {
                max_i = num;
            }
        }
    }
    fn_folder_num.sort();

    let mut fn_db_num: Vec<u32> = Vec::new(); 
    for filename in filenames_db {
        let vec: Vec<&str> = filename.split(".").collect();
        let fname: &str = vec[0];
        if fname.parse::<u32>().is_ok() {
            let num = fname.parse().unwrap();
            fn_folder_num.push(num);
            if max_i < num {
                max_i = num;
            }
        }
    }
    fn_db_num.sort();

    let mut next_filename = "0".to_string();
    let mut free_slots: Vec<u32> = vec![0; usize::try_from(max_i).unwrap() + 1];
 
    for num in fn_folder_num {
        free_slots[ usize::try_from(num).unwrap()] = 1;
    }
    for num in fn_db_num {
        free_slots[ usize::try_from(num).unwrap()] = 1;
    }

    for i in 1..max_i {
        if free_slots[usize::try_from(i).unwrap()] == 0 {
            // filename can be this number
            next_filename = format!("{}", i);
            return next_filename;
        }
    }

    next_filename = format!("{}", (max_i + 1));
    return next_filename;
}