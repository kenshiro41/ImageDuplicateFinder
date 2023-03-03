use image::{imageops, DynamicImage};
use rayon::prelude::*;
use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use walkdir::WalkDir;

fn main() {
    let start = Instant::now();

    let dir_path = "./images";

    let mut image_files = Vec::new();
    for entry in WalkDir::new(dir_path).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path().to_path_buf();
        if path.is_file() && is_image_file(&path) {
            image_files.push(path);
        }
    }
    println!("画像枚数: {:?}枚", image_files.len());

    let image_hash_map = Arc::new(Mutex::new(HashMap::new()));

    image_files.par_iter().for_each(|path| {
        let image = image::open(path).unwrap();
        let hash = get_image_hash(&image);

        // println!("画像: {:?}, hash: {:?}", path.display(), hash);

        image_hash_map
            .lock()
            .unwrap()
            .entry(hash)
            .or_insert(Vec::new())
            .push(path.to_path_buf());
    });

    println!("{:?}", image_hash_map);

    for paths in image_hash_map.lock().unwrap().values() {
        if paths.len() > 1 {
            println!("Similar images:");
            for path in paths {
                println!("{}", path.display());
            }
        }
    }

    let end = start.elapsed();
    println!("{:?}秒", end);
}

fn is_image_file(path: &Path) -> bool {
    let extension = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase());
    match extension.as_deref() {
        Some("jpg") | Some("jpeg") | Some("png") | Some("webp") => true,
        _ => false,
    }
}

fn get_image_hash(image: &DynamicImage) -> u64 {
    let gray_image = image.grayscale();
    let resized_image = imageops::resize(&gray_image, 8, 8, imageops::FilterType::Lanczos3);
    let avg = resized_image
        .enumerate_pixels()
        .fold(0, |sum, (_, _, pixel)| {
            let v = (u64::from(pixel[0]) << 48)
                | (u64::from(pixel[1]) << 32)
                | (u64::from(pixel[2]) << 16)
                | u64::from(pixel[3]);
            sum + v
        })
        / 64;
    let mut hash = 0;
    for (i, (_, _, pixel)) in resized_image.enumerate_pixels().enumerate() {
        let p = (u64::from(pixel[0]) << 48)
            | (u64::from(pixel[1]) << 32)
            | (u64::from(pixel[2]) << 16)
            | u64::from(pixel[3]);
        if p > avg {
            hash |= 1 << i;
        }
    }
    hash
}
