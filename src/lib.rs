use bytes::Bytes;
use rand::prelude::*;
use tokio::prelude::*;
use tokio::stream::{ Stream, StreamExt };

pub mod imagetools;

pub fn gen_rand_id(len: usize) -> String {
    let mut rng = thread_rng();

    (0..len)
        .map(|_| rng.sample(rand::distributions::Alphanumeric))
        .take(len)
        .collect()
}

pub async fn stream_to_writer<S, W, E>(mut stream: S, mut writer: W) -> Result<(), Box<dyn std::error::Error>>
where
    S: Stream<Item = Result<Bytes, E>> + std::marker::Unpin,
    W: AsyncWrite + std::marker::Unpin,
    E: Into<Box<dyn std::error::Error>>,
{
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| e.into())?;
        writer.write_all(&chunk).await?;
    }

    writer.flush().await?;

    Ok(())
}
