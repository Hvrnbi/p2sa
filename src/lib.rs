use image::{GrayImage, Luma, imageops::FilterType::Nearest};
use svg::{Document, node::element::Path, node::element::path::Data, node::element::Rectangle};

/** Import a picture, resize it, and convert it to Greyscale */
fn import_picture(path: &String, new_size: [u32; 2], edge_detection: bool) -> GrayImage {
    let src_img = image::open(path)
        .expect("failed to read image")
        .resize(new_size[0], new_size[1], Nearest)
        .to_luma8();

    let img: GrayImage = if edge_detection {
        let detection = edge_detection::canny(src_img, 1.8, 0.1, 0.01);
        detection.as_image()
            .to_luma8()
    } else {
        src_img
    };

    img
}

/** Create a white image in Greyscale */
fn create_blank_image(width: u32, height: u32) -> GrayImage {
    GrayImage::from_pixel(width, height, Luma([255]))
}

/** Reduce the value of the given pixel by 51 (cause it's 255 / 5, so it makes 6 levels of grey) */
fn darken_pixel(mut img: GrayImage, x: u32, y: u32) -> GrayImage {
    let px_val: u8 = img.get_pixel(x, y).0[0];

    if 55 <= px_val {
        img.put_pixel(x, y, Luma([px_val - 55]));
        img
    } else {
        img
    }
}

/** Places the nails randomly on the image, with a higher probability as the pixel is white */
fn create_nails_list(img: GrayImage, nails_count: u16, width: u32) -> Vec<[u32; 2]> {
    let mut vec_res: Vec<[u32; 2]> = vec![];
    let cumulative_pixel_values: Vec<u32> = image_pixels_cumsum(img.clone());

    for _ in 0..nails_count {
        let rd: u32 = fastrand::u32(0..cumulative_pixel_values[cumulative_pixel_values.len() - 1]);
        let px_ind: u32 = find_pixel(&cumulative_pixel_values, rd);
        let nail: [u32; 2] = [px_ind % width, px_ind / width];
        vec_res.push(nail);
    }

    vec_res
}

/** Takes an image and return the cumulative sum of its pixels in a vector */
fn image_pixels_cumsum(img: GrayImage) -> Vec<u32> {
    img.into_vec()
        .into_iter()
        .scan(0, |acc, x| {
            *acc += u32::from(x);
            Some(*acc)
        })
        .collect()
}

/** Takes a cumulative sum of a vector's valeus and a number, and return the index where the number is bigger than the sum */
fn find_pixel(cumsum: &Vec<u32>, nb: u32) -> u32 {
    let mut i: usize = 0;
    
    while cumsum[i] < nb {
        i += 1;
    }

    i as u32
}

/** Draws a line on the given image between the two given points */
fn draw_line(mut img: GrayImage, nail1: [u32; 2], nail2: [u32; 2]) -> GrayImage {
    let px_arr: Vec<[u32; 2]> = pixels_on_the_line(nail1, nail2);

    for i in 0..px_arr.len() {
        img = darken_pixel(img, px_arr[i][0], px_arr[i][1]);
    }

    img
}

/** Returns a vector with all the points through which the line passes */
fn pixels_on_the_line(start: [u32; 2], end: [u32; 2]) -> Vec<[u32; 2]> {
    let mut vec_res: Vec<[u32; 2]> = vec![];

    let eq_vals: [f32; 2] = line_equation_values(start, end);
    let a: f32 = eq_vals[0];
    let b: f32 = eq_vals[1];

    let mut x: f32;
    let mut y: f32;

    if -1.0 <= a && a <= 1.0 {
        let x2: f32;

        if start[0] < end[0] {
            x = (start[0] + 1) as f32;
            x2 = end[0] as f32;
            vec_res.push([start[0], start[1]]);
        } else {
            x = (end[0] + 1) as f32;
            x2 = start[0] as f32;
            vec_res.push([end[0], end[1]]);
        }

        while x <= x2 {
            y = a * x + b;
            vec_res.push([x as u32, y as u32]);
            x += 1.;
        }
    } else {
        let y2: f32;

        if start[1] < end[1] {
            x = start[0] as f32;
            y2 = end[1] as f32;
            y = (start[1] + 1) as f32;
            vec_res.push([start[0], start[1]]);
        } else {
            x = end[0] as f32;
            y2 = start[1] as f32;
            y = (end[1] + 1) as f32;
            vec_res.push([end[0], end[1]]);
        }

        if a != 999999999. && b != 999999999. {
            while y <= y2 {
                x = (y - b) / a;
                vec_res.push([x as u32, y as u32]);
                y += 1.;
            }
        } else {
            for i in y as u32..(y2 + 1.) as u32 {
                vec_res.push([x as u32, i]);
            }
        }
    }

    vec_res
}

/** Calculates the variablees of the line's equation */
fn line_equation_values(start: [u32; 2], end: [u32; 2]) -> [f32; 2] {
    let a: f32;
    let b: f32;
    
    if start[0] != end[0] {
        a = (i64::from(end[1]) - i64::from(start[1])) as f32 / (i64::from(end[0]) - i64::from(start[0])) as f32;
        b = start[1] as f32 - i64::from(start[0]) as f32 * a;
    } else {
        a = 999999999.;
        b = 999999999.;
    }

    [a, b]
}

/** Calculates the reduction of the error if the line is drawn */
fn line_error_reduction(start: [u32; 2], end: [u32; 2], img_src: &GrayImage, img: &GrayImage) -> f32 {
    if start == end {
        return 999.
    }
    let px_arr = pixels_on_the_line(start, end);
    let mut err: f32 = 0.;
    let px_cnt: f32 = px_arr.len() as f32;

    for i in 0..px_cnt as usize {
        err += pixel_error(px_arr[i], img_src, img, false);
    }

    let mut new_err: f32 = 0.;

    for i in 0..px_cnt as usize {
        new_err += pixel_error(px_arr[i], img_src, img, true);
    }

    (new_err - err) / px_cnt
}

/** Returns the absolute value of the difference between the same pixel on the two images */
fn pixel_error(px: [u32; 2], img_src: &GrayImage, img: &GrayImage, darkened: bool) -> f32 {
    if darkened {
        if img.get_pixel(px[0], px[1]).0[0] >= 51 {
            (i32::from(img_src.get_pixel(px[0], px[1]).0[0]) - i32::from(img.get_pixel(px[0], px[1]).0[0]) + 51).pow(2) as f32
        } else {
            pixel_error(px, img_src, img, false)
        }
    } else {
        (i32::from(img_src.get_pixel(px[0], px[1]).0[0]) - i32::from(img.get_pixel(px[0], px[1]).0[0])).pow(2) as f32
    }
}

/** Finds the line that causes the greatest error reduction between the images */
fn find_best_line(img_src: &GrayImage, img: &GrayImage, start: [u32; 2], nails: Vec<[u32; 2]>) -> ([u32; 2], f32) {
    let half_nails_cnt: usize = nails.len() / 2;
    let half_nails: Vec<[u32; 2]> = fastrand::choose_multiple(nails, half_nails_cnt);
    let mut err_red_vec: Vec<f32> = vec![];

    for i in 0..half_nails.len() {
        err_red_vec.push(line_error_reduction(start, half_nails[i], &img_src, &img));
    }

    let val_and_ind_best_err_red: (f32, usize) = minimum_value_and_index_f32_vector(&err_red_vec);
    let best_err_red: f32 = val_and_ind_best_err_red.0;
    let ind_best_err_red: usize = val_and_ind_best_err_red.1;
    let best_nail: [u32; 2] = half_nails[ind_best_err_red];

    (best_nail, best_err_red)
}

/** Returns the index of the minimum element in the given vector */
fn minimum_value_and_index_f32_vector(vec: &Vec<f32>) -> (f32, usize) {
    let mut min: f32 = vec[0];
    let mut min_ind: usize = 0;

    for i in 1..vec.len() {
        if vec[i] < min {
            min = vec[i];
            min_ind = i;
        }
    }

    (min, min_ind)
}

/** Main function */
pub fn p2sa(src_path: String, output_path: String, new_size: [u32; 2], nails_count: u16) {
    let src_img: GrayImage = import_picture(&src_path, new_size, false);

    let mut new_img: GrayImage = create_blank_image(new_size[0], new_size[1]);

    let edges_img: GrayImage = import_picture(&src_path, new_size, true);

    let nails: Vec<[u32; 2]> = create_nails_list(edges_img, nails_count, new_size[0]);

    let mut start: [u32; 2] = nails[0];
    let mut err_rising_cnt: u8 = 0;
    let mut lines_cnt: u16 = 0;

    let mut first_err_red: f32 = 0.;

    let is_svg: bool;
    let mut svg_data: Data = Data::new();

    if &output_path[output_path.len() - 4..output_path.len()] == ".svg" {
        is_svg = true;
        svg_data = svg_data
            .move_to((start[0], start[1]));
    } else {
        is_svg = false;
    }

    while err_rising_cnt < 3 {
        let best_nail_and_err_red: ([u32; 2], f32) = find_best_line(&src_img, &new_img, start, nails.clone());
        let best_nail: [u32; 2] = best_nail_and_err_red.0;

        if lines_cnt == 0 {
            first_err_red = best_nail_and_err_red.1;
        }
        
        println!("Progress estimation : {} %", (best_nail_and_err_red.1 - first_err_red) * 100. / - first_err_red);

        if best_nail_and_err_red.1 >= 0. {
            err_rising_cnt += 1;
        } else {
            err_rising_cnt = 0;
        }

        new_img = draw_line(new_img, start, best_nail);

        if is_svg {
            svg_data = svg_data.line_to((start[0], start[1]));
        }

        start = best_nail;
        lines_cnt += 1;
    }

    svg_data = svg_data.close();

    if is_svg {

        let svg_bg = Rectangle::new()
            .set("width", new_size[0])
            .set("height", new_size[1])
            .set("x", 0)
            .set("y", 0)
            .set("fill", "white")
            .set("stroke", "none");

        let svg_path = Path::new()
            .set("fill", "none")
            .set("stroke", "black")
            .set("stroke-width", 1)
            .set("stroke-opacity", 0.2)
            .set("d", svg_data);

        let svg_document = Document::new()
            .set("viewbox", (0, 0, new_size[0], new_size[1]))
            .set("width", new_size[0])
            .set("height", new_size[1])
            .add(svg_bg)
            .add(svg_path);

        let _ = svg::save(&output_path, &svg_document);
    } else {
        let _ = new_img.save(output_path);
    }

    println!("{} lines drawn", lines_cnt);
}
