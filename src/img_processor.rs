#![feature(path_file_prefix)]
use exoquant::{convert_to_indexed, optimizer::KMeans, ditherer::FloydSteinberg, Color};
use image::{self, ImageBuffer, Pixel, DynamicImage, EncodableLayout, PixelWithColorType, Rgba};
use std::{env, path::Path, error::Error, io::{Write, Read}, borrow::BorrowMut, fs::OpenOptions};
use image::io::Reader as ImageReader;
use rayon::prelude::{IntoParallelIterator, ParallelIterator};

fn process_file(filename : impl AsRef<Path>, colors : usize) -> Result<(), Box<dyn Error>> {
    let img = ImageReader::open(&filename).unwrap().decode().unwrap();
    let pixels = img.to_rgba8();

    let pixels : Vec<_> = pixels
        .pixels()
        .map(|c| exoquant::Color::new(c[0], c[1], c[2], c[3]))
        .collect();


    let (colors, px) = convert_to_indexed(&pixels, img.width() as usize, colors, &KMeans, &FloydSteinberg::new());


    let brightness = |f : &Color| (0.299 * (f.r as f32).powf(2.0f32) + 0.587 * (f.g as f32).powf(2.0f32) + 0.114 * (f.b as f32).powf(2.0)).sqrt();

    let max_color = (0..colors.len()).max_by(|a, b|
            brightness(colors.get(*a).unwrap()).partial_cmp(&brightness(colors.get(*b).unwrap())).unwrap())
            .unwrap();
    {
        let c = colors.get(max_color).unwrap();
        println!("{max_color} => {{ r : {}, g : {}, b : {}, a : {} }}", c.r, c.g, c.b, c.a);
    }

    let mut path = AsRef::<Path>::as_ref(&filename).to_path_buf();

    path.set_file_name(path.file_prefix().map(|s| format!("{}.bin", s.to_str().unwrap())).unwrap());

    let mut f = OpenOptions::new()
        .write(true)
        .create(true)
        .open(path)?;

    let px = px.iter()
        .flat_map(|a| match a {
            a if (*a) == (max_color as u8) => [255, 255, 255, 255],
            _           => [0, 0, 0, 0]
        })
        .collect::<Vec<_>>();

    f.write_all(&px).unwrap();

    Ok(())
}

fn load_files() -> Result<(), Box<dyn Error>> {
    let colors = 8;
    let folder : Option<String> = env::args()
        .skip(1).next();

    let folder = env::current_dir()
        .ok().and_then(|p| folder.map(|f| p.join(f)))
        .expect("FEED IN A GODDAM ARGUMENT!");


    let v : Vec<_> = folder.read_dir()?
        .filter_map(Result::ok)
        .collect();

    let v = v
        .into_par_iter()
        .filter(|f| f.file_type().map(|s| s.is_file() && f.file_name().to_str().map(|st| !st.contains(".bin")).unwrap_or_default()).unwrap_or_default())
        .map(|f| process_file(f.path(), colors).unwrap())
        .collect::<Vec<_>>();

    println!("Processed {} files.", v.len());

    Ok(())
}