use std::convert::AsRef;
use std::path::{Path, PathBuf};

use actix_web::http::header;
use bytes::Bytes;
use rand::prelude::*;
use tokio::prelude::*;
use tokio::stream::{Stream, StreamExt};
use failure::Fallible;
use failure_derive::Fail;

/// Image processing related stuff
pub mod imagetools;

/// Some basic server configuration
/// * host:port to listem socket on
/// * where to put uploaded/downloaded pictures
#[derive(Clone)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub uploads_dir: PathBuf,
}

/// Info about successfully uploaded/downloaded picture
pub struct UploadedFile {
    /// Unique ID
    pub id: String,
    /// Location of the file
    pub path: PathBuf,
    /// Location of the thumbnail (if succeed to generate)
    pub thumbnail_path: Option<PathBuf>,
}

/// Error describing problem writing streamed file to filesystem
#[derive(Debug, Fail)]
pub enum UploadError {
    /// Error while reading stream
    /// * Stream from client when we handle file upload
    /// * Stream from server when we download picture from given URI
    #[fail(display = "Client error: {}", 0)]
    Client(failure::Error),
    /// Error while creating temporary file or writing stream into that file
    #[fail(display = "Server error: {}", 0)]
    Server(failure::Error),
}

#[derive(Debug, Fail)]
pub enum FetchError {
    #[fail(display = "Server returned non-success code")]
    ServerReturnedError,
    #[fail(display = "Server returned unsupported media type")]
    UnsupportedMediaType,
    #[fail(display = "Fetch failed: {}", 0)]
    FetchError(reqwest::Error),
}

/// Guess file extension based on MIME type or return None if we don't handle that type
pub fn mime_type_to_extension(mime_type: &str) -> Option<&'static str> {
    match mime_type {
        "image/bmp" => Some("bmp"),
        "image/jpeg" => Some("jpg"),
        "image/png" => Some("png"),
        _ => None,
    }
}

/// Generate random alphanumeric string of given length
pub fn gen_rand_id(len: usize) -> String {
    let mut rng = thread_rng();

    (0..len)
        .map(|_| rng.sample(rand::distributions::Alphanumeric))
        .take(len)
        .collect()
}

pub async fn fetch_image(config: &Config, uri: &str) -> Fallible<UploadedFile> {
    let client = reqwest::Client::new();

    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(header::ACCEPT, "image/jpeg, image/png, image/bmp".parse().unwrap());

    let response = client
        .get(uri)
        .headers(headers)
        .send()
        .await?;

    dbg!(&response);

    if !response.status().is_success() {
        return Err(FetchError::ServerReturnedError.into());
    }

    let headers = response.headers();

    let extension = match headers.get(header::CONTENT_TYPE) {
        Some(mime_type) => {
            let mime_type_str = mime_type
                .to_str()
                .map_err(|_| FetchError::UnsupportedMediaType)?;

            match mime_type_to_extension(mime_type_str) {
                Some(ext) => ext,
                None => return Err(FetchError::UnsupportedMediaType.into()),
            }
        }
        None => return Err(FetchError::UnsupportedMediaType.into()),
    };

    let stream = response.bytes_stream();

    upload_image(stream, &config.uploads_dir, extension).await
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

    log::debug!("Uploading to {}", tmp_path.to_str().unwrap_or("?"));

    let res = stream_to_file(stream, &tmp_path).await;
    if let Err(err) = res {
        // log::error!("Upload error: {}", err);
        return Err(err);
    }

    let mut upload_path = tmp_path.clone();
    upload_path.set_extension(extension);

    log::debug!(
        "Renaming {} -> {}",
        tmp_path.to_str().unwrap_or("?"),
        upload_path.to_str().unwrap_or("?")
    );
    tokio::fs::rename(&tmp_path, &upload_path).await.unwrap();

    let mut thumbnail_path = upload_path.clone();
    thumbnail_path.set_file_name(format!("{}_thumbnail.{}", id, extension));

    log::debug!(
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
        log::warn!("Error creating thumbnail: {}", err);
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
