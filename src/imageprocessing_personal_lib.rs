use std::cmp;

use eframe::egui;
use image::*;
use imageproc::point::Point;

pub(crate) struct CropRect {
    pub x0: f32,
    pub y0: f32,
    pub x1: f32,
    pub y1: f32,
}

impl CropRect {
    pub fn new(x0: f32, y0: f32, x1: f32, y1: f32) -> Self {
        return CropRect { x0, y0, x1, y1 };
    }
}

impl Default for CropRect {
    fn default() -> Self {
        return CropRect { x0: -1.0, y0: -1.0, x1: -1.0, y1: -1.0 };
    }
}

pub(crate) enum DrawingType {
    None,
    Arrow,
    Circle,
    Rectangle,
    Line,
}

pub(crate) enum Corner {
    None,
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

pub(crate) fn draw_arrow(image: &DynamicImage, x0: f32, y0: f32, x1: f32, y1: f32, color: Rgba<u8>) -> DynamicImage {
    // Draw the main arrow line
    if (x0 - x1).abs() < 1.0 || (y0 - y1).abs() < 1.0 {
        return image.clone();
    }


    let img = DynamicImage::ImageRgba8(imageproc::drawing::draw_line_segment(image, (x0, y0), (x1, y1), color));


    // Calculate arrowhead points
    let arrow_length = 15.0;
    let arrow_angle: f64 = 20.0;
    let dx = f64::from(x1 - x0);
    let dy = f64::from(y1 - y0);
    let angle = (dy).atan2(dx);
    let arrowhead_size = (dx * dx + dy * dy).sqrt().min(arrow_length);

    // Calculate arrowhead vertices
    let angle1 = angle + arrow_angle.to_radians();
    let angle2 = angle - arrow_angle.to_radians();

    let x2 = (x1 as f64 - arrowhead_size * angle1.cos()) as f32;
    let y2 = (y1 as f64 - arrowhead_size * angle1.sin()) as f32;
    let x3 = (x1 as f64 - arrowhead_size * angle2.cos()) as f32;
    let y3 = (y1 as f64 - arrowhead_size * angle2.sin()) as f32;

    let arrowhead_points: &[Point<i32>] = &[Point::new(x1 as i32, y1 as i32), Point::new(x2 as i32, y2 as i32), Point::new(x3 as i32, y3 as i32)];

    // Draw arrowhead polygon
    return DynamicImage::ImageRgba8(imageproc::drawing::draw_polygon(&img, arrowhead_points, color));
}

pub(crate) fn draw_rect(image: &DynamicImage, x0: f32, y0: f32, x1: f32, y1: f32, color: Rgba<u8>) -> DynamicImage {
    let mut startx = cmp::min(x0 as i32, x1 as i32);
    let mut endx = cmp::max(x0 as i32, x1 as i32);
    let mut starty = cmp::min(y0 as i32, y1 as i32);
    let mut endy = cmp::max(y0 as i32, y1 as i32);

    startx = cmp::max(startx, 0);
    starty = cmp::max(starty, 0);
    endx = cmp::max(endx, 0);
    endy = cmp::max(endy, 0);

    if endx as u32 - startx as u32 == 0 || endy as u32 - starty as u32 == 0 {
        return DynamicImage::ImageRgba8(imageproc::drawing::draw_line_segment(image, (startx as f32, starty as f32), (endx as f32, endy as f32), color));
    }
    return DynamicImage::ImageRgba8(imageproc::drawing::draw_hollow_rect(image, imageproc::rect::Rect::at(startx, starty).of_size(endx as u32 - startx as u32, endy as u32 - starty as u32), color));
}

pub(crate) struct ImageProcSetting{
    pub drawing: bool,
    pub drawing_type: DrawingType,
    pub initial_pos: egui::Pos2,
    pub crop: bool,
    pub crop_point: CropRect,
    pub current_crop_point: Corner,
    pub texting: bool,
    pub text_string: String,
}

impl ImageProcSetting{
    pub fn setup_arrow() -> Self{
        return Self{
            drawing : true,
            crop : false,
            drawing_type : DrawingType::Arrow,
            initial_pos : egui::pos2(-1.0, -1.0),
            crop_point : CropRect::default(),
            current_crop_point : Corner::None,
            texting : false,
            text_string : "".to_string(),
        }
    }

    pub fn setup_circle() -> Self{
        return Self{
            drawing : true,
            crop : false,
            drawing_type : DrawingType::Circle,
            initial_pos : egui::pos2(-1.0, -1.0),
            crop_point : CropRect::default(),
            current_crop_point : Corner::None,
            texting : false,
            text_string : "".to_string(),
        }
    }
    pub fn setup_line() -> Self{
        return Self{
            drawing : true,
            crop : false,
            drawing_type : DrawingType::Line,
            initial_pos : egui::pos2(-1.0, -1.0),
            crop_point : CropRect::default(),
            current_crop_point : Corner::None,
            texting : false,
            text_string : "".to_string(),
        }
    }
    pub fn setup_rectangle() -> Self{
        return Self{
            drawing : true,
            crop : false,
            drawing_type : DrawingType::Rectangle,
            initial_pos : egui::pos2(-1.0, -1.0),
            crop_point : CropRect::default(),
            current_crop_point : Corner::None,
            texting : false,
            text_string : "".to_string(),
        }
    }

    pub fn setup_text() ->Self {
        return Self{
            drawing : false,
            crop : false,
            drawing_type : DrawingType::None,
            initial_pos : egui::pos2(-1.0, -1.0),
            crop_point : CropRect::default(),
            current_crop_point : Corner::None,
            texting : true,
            text_string : "".to_string(),
        }
    }

    pub fn setup_crop(w:f32, h: f32) -> Self{
        return Self{
            drawing : false,
            crop : true,
            drawing_type : DrawingType::None,
            initial_pos : egui::pos2(-1.0, -1.0),
            crop_point : CropRect::new(0.0, 0.0, w, h),
            current_crop_point : Corner::None,
            texting : false,
            text_string : "".to_string(),
        }
    }
}

impl Default for ImageProcSetting{
    fn default() -> Self {
        return Self{
            drawing: false,
            drawing_type: DrawingType::None,
            initial_pos: egui::pos2(-1.0, -1.0),
            crop: false,
            crop_point: CropRect::default(),
            current_crop_point: Corner::None,
            texting: false,
            text_string: "".to_string(),
        }
    }
}