use std::convert::AsRef;
use std::path::{Path, PathBuf};

use bytes::Bytes;
use rand::prelude::*;
use tokio::prelude::*;
use tokio::stream::{Stream, StreamExt};
use failure::Fallible;
use failure_derive::Fail;

pub mod imagetools;

pub struct UploadedFile {
    pub id: String,
    pub path: PathBuf,
    pub thumbnail_path: Option<PathBuf>,
}

#[derive(Debug, Fail)]
pub enum UploadError {
    #[fail(display = "Client error: {}", 0)]
    Client(failure::Error),
    #[fail(display = "Server error: {}", 0)]
    Server(failure::Error),
}

pub fn mime_type_to_extension(mime_type: &str) -> Option<&'static str> {
    match mime_type {
        "image/bmp" => Some("bmp"),
        "image/jpeg" => Some("jpg"),
        "image/png" => Some("png"),
        _ => None,
    }
}

pub fn gen_rand_id(len: usize) -> String {
    let mut rng = thread_rng();

    (0..len)
        .map(|_| rng.sample(rand::distributions::Alphanumeric))
        .take(len)
        .collect()
}

pub async fn upload_image<S, P, E>(
    stream: S,
    uploads_dir: P,
    extension: &str,
) -> Fallible<UploadedFile>
where
    S: Stream<Item = Result<Bytes, E>> + std::marker::Unpin,
    P: AsRef<Path>,
    E: Into<failure::Error>,
{
    let id = gen_rand_id(12);

    let mut tmp_path = PathBuf::with_capacity(64);
    tmp_path.push(&uploads_dir);
    tmp_path.push(&id);
    tmp_path.set_extension("tmp");

    eprintln!("Uploading to {}", tmp_path.to_str().unwrap_or("?"));

    let res = stream_to_file(stream, &tmp_path).await;
    if let Err(err) = res {
        // eprintln!("Upload error: {}", err);
        return Err(err);
    }

    let mut upload_path = tmp_path.clone();
    upload_path.set_extension(extension);

    eprintln!(
        "Renaming {} -> {}",
        tmp_path.to_str().unwrap_or("?"),
        upload_path.to_str().unwrap_or("?")
    );
    tokio::fs::rename(&tmp_path, &upload_path).await.unwrap();

    let mut thumbnail_path = upload_path.clone();
    thumbnail_path.set_file_name(format!("{}_thumbnail.{}", id, extension));

    eprintln!(
        "Thumbnail {} -> {}",
        upload_path.to_str().unwrap_or("?"),
        thumbnail_path.to_str().unwrap_or("?")
    );

    let (upload_path_clone, thumbnail_path_clone) = (upload_path.clone(), thumbnail_path.clone());
    // Processing of a big image may be a hard task,
    // let's do it on a dedicated thread
    let res = tokio::task::spawn_blocking(move || {
        imagetools::create_thumbnail(&upload_path_clone, &thumbnail_path_clone, (100, 100))
    })
    .await
    .unwrap();

    let thumbnail_path = if let Err(err) = res {
        eprintln!("Error creating thumbnail: {}", err);
        None
    } else {
        Some(thumbnail_path)
    };

    Ok(UploadedFile {
        id,
        path: upload_path,
        thumbnail_path,
    })
}

pub async fn stream_to_file<S, P, E>(stream: S, filename: P) -> Fallible<()>
where
    S: Stream<Item = Result<Bytes, E>> + std::marker::Unpin,
    P: AsRef<Path>,
    E: Into<failure::Error>,
{
    let file = tokio::fs::File::create(&filename)
        .await
        .map_err(|e| UploadError::Server(e.into()))?;
    let writer = tokio::io::BufWriter::new(file);

    let res = stream_to_writer(stream, writer).await;
    if res.is_err() {
        tokio::fs::remove_file(&filename).await.unwrap();
    }
    res
}

pub async fn stream_to_writer<S, W, E>(mut stream: S, mut writer: W) -> Fallible<()>
where
    S: Stream<Item = Result<Bytes, E>> + std::marker::Unpin,
    W: AsyncWrite + std::marker::Unpin,
    E: Into<failure::Error>,
{
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| UploadError::Client(e.into()))?;
        writer
            .write_all(&chunk)
            .await
            .map_err(|e| UploadError::Server(e.into()))?;
    }

    writer
        .flush()
        .await
        .map_err(|e| UploadError::Server(e.into()))?;

    Ok(())
}
