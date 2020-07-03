use std::path::Path;

use opencv::core::{ Mat, CV_8UC3, Size_, Vector };
use opencv::imgcodecs::{ imread, imwrite, IMREAD_COLOR };
use opencv::imgproc::{ resize, INTER_AREA };

pub fn create_thumbnail<P>(src: P, dest: P, (w, h): (u16, u16)) -> opencv::Result<()>
where
    P: AsRef<Path>,
{
    let src = src.as_ref().to_str().unwrap();
    let dest = dest.as_ref().to_str().unwrap();

    let src_image = imread(src, IMREAD_COLOR)?;

    let size = Size_::new(w as i32, h as i32);

    let mut dest_image = unsafe { Mat::new_size(size, CV_8UC3)}?;

    resize(
        &src_image,
        &mut dest_image,
        size,
        0.0, 0.0,
        INTER_AREA
    )?;

    let params = Vector::new();
    imwrite(dest, &dest_image, &params)?;

    Ok(())
}
