extern crate clap;
extern crate image;
extern crate rusttype;

use clap::{App, Arg};
use image::imageops::crop;
use image::{DynamicImage, GenericImageView, Rgba};
use rusttype::{point, FontCollection, Scale};

static FONT_SIZE: f32 = 12.0;
static THUMBNAIL_W: u32 = 40;

fn main() {
    let matches = args();
    let image_path = matches.value_of("image").unwrap();

    // open specified image, turn it into grayscale and resize to size: (THUMBNAIL_W, whatever)
    let img = image::open(image_path)
        .expect("Not a valid image file.")
        .thumbnail(THUMBNAIL_W, std::u32::MAX)
        .grayscale();

    // chars that will be used on output image
    let letters = if let Some(letters) = matches.value_of("letters") {
        // use specified chars
        letters.chars().collect()
    } else {
        // default chars
        vec![' ', '·', '>', 'X']
    };
    let letters_len = letters.len() as f32;

    let mut data: Vec<String> = vec![];

    let mut count_row = 0;
    for p in img.raw_pixels() {
        if count_row % THUMBNAIL_W == 0 {
            data.push(String::from(""));
        }
        let row = data.len() - 1;
        data[row].push(letters[((p as f32 / 255.0) * letters_len) as usize]);
        count_row += 1;
    }

    let font = Vec::from(include_bytes!("../fonts/Roboto/RobotoMono-Regular.ttf") as &[u8]);
    let font = FontCollection::from_bytes(font)
        .unwrap()
        .into_font()
        .unwrap();
    let font_size = if let Some(s) = matches.value_of("font_size") {
        s.parse::<f32>().unwrap()
    } else {
        FONT_SIZE
    };
    let line_height = font_size.floor() as i32;
    let scale = Scale {
        x: font_size * 2.0, // monospace font
        y: font_size,
    };
    let v_metrics = font.v_metrics(scale);
    let offset = point(0.0, v_metrics.ascent);

    let (w, h) = img.dimensions();
    let (image_width, image_height) = (w * 99, h * 99); // Big enough to render all content
    let mut image = DynamicImage::new_rgba8(image_width, image_height).to_rgba();

    let (mut valid_width, mut valid_height) = (0, 0);
    for (row, s) in data.iter().enumerate() {
        let glyphs: Vec<_> = font.layout(s, scale, offset).collect();
        let width = glyphs
            .iter()
            .rev()
            .map(|g| g.position().x as f32 + g.unpositioned().h_metrics().advance_width)
            .next()
            .unwrap_or(0.0)
            .ceil() as i32;
        valid_width = std::cmp::max(valid_width, width);
        for g in glyphs {
            if let Some(bb) = g.pixel_bounding_box() {
                g.draw(|x, y, v| {
                    let image_x = x as i32 + bb.min.x;
                    let glyph_height = bb.max.y - bb.min.y;
                    let image_y = (row + 1) as i32 * line_height - glyph_height + y as i32 - 2;
                    valid_height = image_y + 8;

                    if image_x >= 0
                        && image_x < image_width as i32
                        && image_y >= 0
                        && image_y < image_height as i32
                    {
                        let color = Rgba {
                            data: [0, (255.0 * v) as u8, 0, 255],
                        };
                        image.put_pixel(image_x as u32, image_y as u32, color);
                    }
                })
            }
        }
    }

    image = crop(&mut image, 0, 0, valid_width as u32, valid_height as u32).to_image();

    let output_filename = if let Some(o) = matches.value_of("out") {
        o.to_owned() + ".jpg"
    } else {
        "out.jpg".to_owned()
    };
    image.save(&output_filename).unwrap();

    println!("Generated {}", &output_filename);
}

fn args() -> clap::ArgMatches<'static> {
    App::new("aart")
        .version("1.0")
        .author("Chuang Yu <cyu9960@gmail.com>")
        .about("Convert image to ascii art.")
        .arg(
            Arg::with_name("out")
                .short("o")
                .long("out")
                .value_name("FILE")
                .help("Sets the output file(default: out.png)"),
        )
        .arg(
            Arg::with_name("letters")
                .short("l")
                .long("letters")
                .value_name("STRING")
                .help("Sets the letters used on output file."),
        )
        .arg(
            Arg::with_name("font_size")
                .short("s")
                .long("size")
                .value_name("FONT SIZE")
                .help("Sets the font size(default: 18)."),
        )
        .arg(
            Arg::with_name("image")
                .help("Sets the image file to convert")
                .required(true)
                .index(1),
        )
        .get_matches()
}
