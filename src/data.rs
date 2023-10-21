use druid::{
    widget::{
        Svg, Stepper, Button, Container, Either, FillStrat, Flex, Image, Painter, SizedBox, ZStack, Label, TextBox, Align,
    },
    FontDescriptor, FontFamily, Color, Data, Env, EventCtx, ImageBuf, Lens,
    PaintCtx, Point, RenderContext, TimerToken,
    Widget, WidgetExt, WindowDesc, WindowState, Rect, Cursor, CursorDesc,
};
use im::HashMap;
use image::{ImageBuffer, Rgba, DynamicImage};
use imageproc::{*, integral_image::ArrayData};

use kurbo::{Shape, BezPath};
use nalgebra::ComplexField;
use piet_common::{/*D2DTextLayout, */Text, TextLayoutBuilder, d2d::Bitmap, PietImage, Piet};
use crate::controller::*;
use arboard::Clipboard;
use arboard::ImageData;
use screenshots::Screen;
use serde::{Deserialize, Serialize};
use std::{borrow::Cow, hash::Hash, clone};
use bincode;

#[derive(Clone, Data, PartialEq, Debug, Serialize, Deserialize)]
pub enum Format {
    Png,
    Jpg,
    Gif,
    Pnm,
    Tga,
    Qoi,
    Tiff,
    Webp,
    Bmp
}

impl Format {
    pub fn to_string(&self) -> String {
        match self {
            Format::Jpg => ".jpg".to_string(),
            Format::Png => ".png".to_string(),
            Format::Gif => ".gif".to_string(),
            Format::Pnm => ".pnm".to_string(),
            Format::Tga => ".tga".to_string(),
            Format::Qoi => ".qoi".to_string(),
            Format::Tiff => ".tiff".to_string(),
            Format::Webp => ".webp".to_string(),
            Format::Bmp => ".bmp".to_string(),
        }
    }
}

#[derive(Clone, Data, PartialEq, Debug)]
pub enum ShapeTool{
    Arrow,
    Circle,
    Square,
}

#[derive(Clone, Data, PartialEq, Debug, Serialize, Deserialize)]
pub enum EditTool{
    Pencil,
    Highlighter,
    Shape,
    Text,
    Eraser,
}

#[derive(Clone, Data, PartialEq, Debug, Serialize, Deserialize)]
pub enum ColorTool{
    Black,
    Red,
    Blue,
    Yellow,
    Green,
    White,
}

#[derive(Clone, Data, PartialEq, Eq, Debug, Serialize, Deserialize, Hash)]
pub enum Shortcut{
    Save,
    SaveAs,
    Open,
    Customize,
    Screenshot,
    Capture,
    Quit,
}

#[derive(Clone, Data, Lens)]
pub struct RgbaArea {
    pub r: f64,
    pub g: f64,
    pub b: f64,
    pub a: f64,
}
impl RgbaArea {
    pub fn reset(&mut self) {
        self.r = 10.0;
        self.g = 10.0;
        self.b = 10.0;
        self.a = 0.4;
    }
}

#[derive(Clone, Data, Lens)]
pub struct SelectedArea {
    pub start: Point,
    pub end: Point,
    pub width: f64,
    pub heigth: f64,
    pub scale: f64,
    pub rgba: RgbaArea,
}
impl SelectedArea {
    pub fn new() -> Self {
        let displays = screenshots::DisplayInfo::all().expect("error");
        let scale = displays[0].scale_factor as f64;
        Self {
            start: Point { x: 0.0, y: 0.0 },
            end: Point { x: 0.0, y: 0.0 },
            width: 0.0,
            heigth: 0.0,
            scale,
            rgba: RgbaArea {
                r: 10.0,
                g: 10.0,
                b: 10.0,
                a: 0.4,
            },
        }
    }
}

#[derive(Clone, Data, Lens)]
pub struct ResizedArea {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}
impl ResizedArea {
    pub fn new() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            width: 0.0,
            height: 0.0,
        }
    }
    pub fn new_parameter(x: f64, y: f64, width: f64, height: f64) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }
}

#[derive(Clone, Data, Lens)]
pub struct Draw{
    pub points: im::Vector<(im::Vector<Point>, Color, f64, f64)>,  //(punti, colore della traccia, spessore linea, alpha)
    pub segment: usize,
}

#[derive(Clone, Data, Lens)]
pub struct Write{
    pub text: String,
    pub position: Point,
    pub color: Color,
    pub thickness: f64,
    pub dimensions: (f64, f64),
}
impl Write{
    pub fn new()->Self{
        Self { text: String::from(""), position: Point { x: 0., y: 0. }, color: Color::WHITE, thickness: 5., dimensions: ( 0., 0. ) }
    }
}

#[derive(Clone, Data, Lens)]
pub struct Arrow{
    pub start: Point,
    pub end: Point,
    pub color: Color,
    pub thickness: f64,
}
impl Arrow {
    pub fn new()->Self{
        Self { start: Point::new(0., 0.), end: Point::new(0., 0.), color: Color::WHITE, thickness: 1. }
    }
}

#[derive(Clone, Data, Lens)]
pub struct Circle{
    pub start: Point,
    pub end: Point,
    pub color: Color,
    pub thickness: f64,
}
impl Circle {
    pub fn new()->Self{
        Self { start: Point::new(0., 0.), end: Point::new(0., 0.), color: Color::WHITE, thickness: 1. }
    }
}

#[derive(Clone, Data, Lens)]
pub struct Square{
    pub start: Point,
    pub end: Point,
    pub color: Color,
    pub thickness: f64,
    pub image: ImageBuf,
}
impl Square {
    pub fn new()->Self{
        Self { start: Point::new(0., 0.), end: Point::new(0., 0.), color: Color::WHITE, thickness: 1. , image: ImageBuf::empty()}
    }
}

#[derive(Clone, Data, Lens)]
pub struct Screenshot {
    pub name: String,
    pub format: Format,
    pub new_name: String,
    pub editing_name: bool,
    pub screen_fatto: bool,
    pub img: ImageBuf,
    pub area: SelectedArea,
    pub flag_transparency: bool,
    pub flag_selection: bool, //serve per fare far partire il controller solo dopo aver acquisito l'area
    pub full_screen: bool,
    pub time_interval: f64,
    pub default_save_path: String,
    pub flag_resize: bool,
    pub resized_area: ResizedArea,
    pub shortcut: HashMap<Shortcut, String>,
    pub prec_hotkey: String,
    pub selected_shortcut: Shortcut,
    pub editing_shortcut: bool,
    pub duplicate_shortcut: bool,
    pub monitor_id: usize,
    pub flag_desk2: bool,
    pub flag_edit: bool,
    pub edit_tool: EditTool,
    pub color_tool: ColorTool,
    pub shape_tool: ShapeTool,
    pub draw: Draw,
    pub write: (im::Vector<Write>, usize),
    pub arrows: (im::Vector<Arrow>, usize),
    pub circles: (im::Vector<Circle>, usize),
    pub squares: (im::Vector<Square>, usize),
    pub text: String,
    pub editing_text: i32,
    pub line_thickness: f64,
    pub painter: ImageBuf,
    pub custom_cursor: Cursor,
    #[data(ignore)]
    pub custom_cursor_desc: CursorDesc,
}

impl Screenshot {
    pub fn new(name: String, format: Format, newname: String) -> Self {
        let mut shortcut = HashMap::new();
        shortcut.insert(Shortcut::Save, String::from("Control+s"));
        shortcut.insert(Shortcut::SaveAs, String::from("Control+a"));
        shortcut.insert(Shortcut::Open, String::from("Control+o"));
        shortcut.insert(Shortcut::Customize, String::from("Control+k"));
        shortcut.insert(Shortcut::Screenshot, String::from("Control+t"));
        shortcut.insert(Shortcut::Capture, String::from("Control+y"));
        shortcut.insert(Shortcut::Quit, String::from("Control+q"));

        let mut points = im::Vector::new();
        points.push_back((im::Vector::new(), Color::WHITE, 1., 1.));

        let mut text: im::Vector<Write> = im::Vector::new();
        text.push_back(Write::new());

        let mut arrows = im::Vector::new();
        arrows.push_back(Arrow::new()); 

        let mut circles = im::Vector::new();
        circles.push_back(Circle::new()); 

        let mut squares = im::Vector::new();
        squares.push_back(Square::new());
        
        let cursor_image = ImageBuf::from_data(include_bytes!("./svg/icons8-pencil-48.png")).unwrap();
        // The (0,0) refers to where the "hotspot" is located, so where the mouse actually points.
        // (0,0) is the top left, and (cursor_image.width(), cursor_image.width()) the bottom right.
        let custom_cursor_desc = CursorDesc::new(cursor_image, (0.0, 0.0));

        Self {
            name,
            format,
            new_name: newname,
            editing_name: false,
            screen_fatto: false,
            img: ImageBuf::empty(),
            area: SelectedArea::new(),
            flag_transparency: false,
            flag_selection: false,
            full_screen: false,
            time_interval: 0.0,
            default_save_path: "C:/Users/Utente/Pictures".to_string(),
            flag_resize: false,
            resized_area: ResizedArea::new(),
            shortcut,
            prec_hotkey: String::from(""),
            selected_shortcut: Shortcut::Screenshot,
            editing_shortcut: true,
            duplicate_shortcut: false,
            monitor_id: 0,
            flag_desk2: false,
            flag_edit: false,
            edit_tool: EditTool::Pencil,
            color_tool: ColorTool::White,
            shape_tool: ShapeTool::Arrow,
            draw: Draw { points, segment: 0},
            write: (text, 0),
            text: String::from(""),
            arrows: (arrows, 0),
            circles: (circles, 0),
            squares: (squares, 0),
            editing_text: -1,
            line_thickness: 3.,
            painter: ImageBuf::empty(),
            custom_cursor: Cursor::Arrow, //do we really need it here? Could we move it to the controller?!
            custom_cursor_desc, //to check 
        }
    }

    pub fn toggle_textbox_state(data: &mut Screenshot) {
        if data.editing_name {
            data.editing_name = false;
        } else {
            data.editing_name = true;
        }
    }

    pub fn action_screen(&mut self, ctx: &mut EventCtx){
        let displays = screenshots::DisplayInfo::all().expect("error");
        let scale = displays[0].scale_factor as f64;
        let width = displays[0].width as f64 * scale;
        let height = displays[0].height as f64 * scale;

        let mut current = ctx.window().clone();
        current.set_window_state(WindowState::Minimized);
        self.full_screen = true;

        self.area.start = Point::new(0.0, 0.0);
        self.area.end = Point::new(0.0, 0.0);
        self.area.width = 0.0;
        self.area.heigth = 0.0;
        self.area.rgba.reset();

        let new_win = WindowDesc::new(draw_rect())
            .show_titlebar(false)
            .transparent(true)
            .window_size((width, height))
            // .set_window_state(WindowState::Maximized)
            .resizable(true)
            .set_position((0.0, 0.0))
            .set_always_on_top(true);

        ctx.new_window(new_win);
    }

    pub fn action_capture(&mut self, ctx: &mut EventCtx){
        let displays = screenshots::DisplayInfo::all().expect("error");
        let scale = displays[0].scale_factor as f64;
        let width = displays[0].width as f64 * scale;
        let height = displays[0].height as f64 * scale;

        let mut current = ctx.window().clone();
        current.set_window_state(WindowState::Minimized);
        self.full_screen = false;

        self.area.start = Point::new(0.0, 0.0);
        self.area.end = Point::new(0.0, 0.0);
        self.area.width = 0.0;
        self.area.heigth = 0.0;
        self.area.rgba.reset();

        let container = Either::new(
            |data: &Screenshot, _: &Env| data.flag_transparency,
            Container::new(draw_rect()).background(Color::rgba(0.0, 0.0, 0.0, 0.0)),
            Container::new(draw_rect()).background(Color::rgba(0.0, 0.0, 0.0, 0.6)),
        );

        let container2 = Either::new(
            |data: &Screenshot, _: &Env| data.flag_transparency,
            Container::new(draw_rect()).background(Color::rgba(0.0, 0.0, 0.0, 0.0)),
            Container::new(draw_rect()).background(Color::rgba(0.0, 0.0, 0.0, 0.6)),
        );

        let stack = Either::new(
            |data: &Screenshot, _: &Env| data.monitor_id == 0,
            container,
            {
                self.do_screen();
                let img = Image::new(self.img.clone());
                let sizedbox = SizedBox::new(Flex::column().with_child(img)).fix_size(width, height);
                let col = ZStack::new(sizedbox)
                .with_centered_child(container2);
                col
            }
        ).center();

        let new_win = WindowDesc::new(stack)
            .show_titlebar(false)
            .transparent(true)
            .window_size((width, height))
            .resizable(false);
            // .set_window_state(WindowState::Maximized);
            // .set_position((0.0, 0.0));

        ctx.new_window(new_win);
    }

    pub fn do_screen(&mut self) {
        let screens = Screen::all().unwrap();
        let image: ImageBuffer<Rgba<u8>, Vec<u8>> = screens[self.monitor_id].capture().unwrap();
        let time: String = chrono::offset::Utc::now().to_string();

        self.name = time;
        self.name = self
            .name
            .replace(".", "-")
            .replace(":", "-")
            .replace(" ", "_");
        // self.name += &self.format.to_string();

        self.img = ImageBuf::from_raw(
            image.clone().into_raw(),
            druid::piet::ImageFormat::RgbaPremul,
            image.clone().width() as usize,
            image.clone().height() as usize,
        );
        // if self.monitor_id != 0{
        //     self.flag_desk2 = true;
        // }
        self.screen_fatto = true;
        self.flag_transparency = false;
    }

    pub fn do_screen_area(&mut self) {
        let screens = Screen::all().unwrap();
        let image = screens[0]
            .capture_area(
                ((self.area.start.x) * self.area.scale) as i32,
                ((self.area.start.y) * self.area.scale) as i32,
                (self.area.width) as u32,
                (self.area.heigth) as u32,
            )
            .unwrap();

        self.name = chrono::offset::Utc::now().to_string();
        self.name = self
            .name
            .replace(".", "-")
            .replace(":", "-")
            .replace(" ", "_");
        // self.name += &self.format.to_string();

        self.img = ImageBuf::from_raw(
            image.clone().into_raw(),
            druid::piet::ImageFormat::RgbaPremul,
            image.clone().width() as usize,
            image.clone().height() as usize,
        );

        self.screen_fatto = true;
        self.flag_transparency = false;
    }

    pub fn screen_window(&mut self, ctx: &mut EventCtx) {
        let window = WindowDesc::new(show_screen(ctx, self.img.clone(), self))
            .title(self.name.clone())
            .set_window_state(druid_shell::WindowState::Maximized)
            .set_always_on_top(true);
        ctx.new_window(window);
    }

    pub fn reset_resize_rect(&mut self) {
        let area_width = 1000.;
        let area_height = 562.5;
        let original_width = self.img.width() as f64;
        let original_height = self.img.height() as f64;

        // Calcola le dimensioni ridimensionate dell'immagine mantenendo i rapporti tra larghezza e altezza.
        let mut new_width = original_width;
        let mut new_height = original_height;

        if original_width > area_width {
            new_width = area_width;
            new_height = (area_width * original_height) / original_width;
        }

        if new_height > area_height {
            new_height = area_height;
            new_width = (area_height * original_width) / original_height;
        }

        let center_x = area_width / 2.;
        let center_y = area_height / 2.;

        let top_left_x = center_x - (new_width / 2.);
        let top_left_y = center_y - (new_height / 2.);

        self.resized_area.x = top_left_x;
        self.resized_area.y = top_left_y;
        self.resized_area.width = new_width;
        self.resized_area.height = new_height;
    }

}

pub fn build_toolbar() -> impl Widget<Screenshot>{
    let mut row = Flex::row();
    let pencil = Either::new(
        |data, _| data.edit_tool == EditTool::Pencil,
        Image::new(ImageBuf::from_data(include_bytes!("./svg/icons8-pencil-48.png")).unwrap()).fix_size(30., 30.).on_click(
            |_ctx, data: &mut Screenshot, _: &Env|{
                data.edit_tool = EditTool::Pencil;
                data.line_thickness = 3.;
            }
        ).border( Color::BLACK, 2.).background(Color::GRAY),
        Image::new(ImageBuf::from_data(include_bytes!("./svg/icons8-pencil-48.png")).unwrap()).fix_size(30., 30.).on_click(
            |_ctx, data: &mut Screenshot, _: &Env|{
                data.edit_tool = EditTool::Pencil;
                data.line_thickness = 3.;
            }
        ),
    );

    let highlighter = Either::new(
        |data, _| data.edit_tool == EditTool::Highlighter,
        Image::new(ImageBuf::from_data(include_bytes!("./svg/icons8-highlighter-48.png")).unwrap()).fix_size(30., 30.).on_click(
            |_ctx, data: &mut Screenshot, _: &Env|{
                data.edit_tool = EditTool::Highlighter;
                data.line_thickness = 10.;
            }
        ).border( Color::BLACK, 2.).background(Color::GRAY),
        Image::new(ImageBuf::from_data(include_bytes!("./svg/icons8-highlighter-48.png")).unwrap()).fix_size(30., 30.).on_click(
            |_ctx, data: &mut Screenshot, _: &Env|{
                data.edit_tool = EditTool::Highlighter;
                data.line_thickness = 10.;
            }
        ),
    );

    let shapes = Either::new(
        |data, _| data.edit_tool == EditTool::Shape,
        Image::new(ImageBuf::from_data(include_bytes!("./svg/icons8-shape-32.png")).unwrap()).fix_size(30., 30.).on_click(
            |_ctx, data: &mut Screenshot, _: &Env|{
                data.edit_tool = EditTool::Shape;
                data.line_thickness = 3.;
            }
        ).border( Color::BLACK, 2.).background(Color::GRAY),
        Image::new(ImageBuf::from_data(include_bytes!("./svg/icons8-shape-32.png")).unwrap()).fix_size(30., 30.).on_click(
            |_ctx, data: &mut Screenshot, _: &Env|{
                data.edit_tool = EditTool::Shape;
                data.line_thickness = 3.;
            }
        ),
    );

    let text = Either::new(
        |data, _| data.edit_tool == EditTool::Text,
        Image::new(ImageBuf::from_data(include_bytes!("./svg/icons8-text-50.png")).unwrap()).fix_size(30., 30.).on_click(
            |_ctx, data: &mut Screenshot, _: &Env|{
                data.edit_tool = EditTool::Text;
                data.line_thickness = 20.;
            }
        ).border( Color::BLACK, 2.).background(Color::GRAY),
        Image::new(ImageBuf::from_data(include_bytes!("./svg/icons8-text-50.png")).unwrap()).fix_size(30., 30.).on_click(
            |_ctx, data: &mut Screenshot, _: &Env|{
                data.edit_tool = EditTool::Text;
                data.line_thickness = 20.;
            }
        ),
    );

    let eraser = Either::new(
        |data, _| data.edit_tool == EditTool::Eraser,
        Image::new(ImageBuf::from_data(include_bytes!("./svg/eraser2.png")).unwrap()).fix_size(30., 30.).on_click(
            |_ctx, data: &mut Screenshot, _: &Env|{
                data.edit_tool = EditTool::Eraser;
            }
        ).border( Color::BLACK, 2.).background(Color::GRAY),
        Image::new(ImageBuf::from_data(include_bytes!("./svg/eraser2.png")).unwrap()).fix_size(30., 30.).on_click(
            |_ctx, data: &mut Screenshot, _: &Env|{
                data.edit_tool = EditTool::Eraser;
            }
        ),
    );

    let mut row_color = Flex::row();
    row_color.add_child(Either::new(
        |data: &Screenshot, _| data.color_tool == ColorTool::White,
        Image::new(ImageBuf::from_data(include_bytes!("./svg/icons8-white-circle-48.png")).unwrap()).fix_size(30., 30.).on_click(
            |_ctx, data: &mut Screenshot, _: &Env|{
                data.color_tool = ColorTool::White;
            }
        ).border( Color::BLACK, 2.).background(Color::GRAY),
        Image::new(ImageBuf::from_data(include_bytes!("./svg/icons8-white-circle-48.png")).unwrap()).fix_size(30., 30.)).on_click(
            |_ctx, data: &mut Screenshot, _: &Env|{
                data.color_tool = ColorTool::White;
            }
        ));
    row_color.add_child(Either::new(
        |data: &Screenshot, _| data.color_tool == ColorTool::Black,
        Image::new(ImageBuf::from_data(include_bytes!("./svg/icons8-black-circle-48.png")).unwrap()).fix_size(30., 30.).on_click(
            |_ctx, data: &mut Screenshot, _: &Env|{
                data.color_tool = ColorTool::Black;
            }
        ).border( Color::BLACK, 2.).background(Color::GRAY),
        Image::new(ImageBuf::from_data(include_bytes!("./svg/icons8-black-circle-48.png")).unwrap()).fix_size(30., 30.)).on_click(
            |_ctx, data: &mut Screenshot, _: &Env|{
                data.color_tool = ColorTool::Black;
            }
        ));
    row_color.add_child(Either::new(
        |data: &Screenshot, _| data.color_tool == ColorTool::Red,
        Image::new(ImageBuf::from_data(include_bytes!("./svg/icons8-red-circle-48.png")).unwrap()).fix_size(30., 30.).on_click(
            |_ctx, data: &mut Screenshot, _: &Env|{
                data.color_tool = ColorTool::Red;
            }
        ).border( Color::BLACK, 2.).background(Color::GRAY),
        Image::new(ImageBuf::from_data(include_bytes!("./svg/icons8-red-circle-48.png")).unwrap()).fix_size(30., 30.)).on_click(
            |_ctx, data: &mut Screenshot, _: &Env|{
                data.color_tool = ColorTool::Red;
            }
        ));
    row_color.add_child(Either::new(
        |data: &Screenshot, _| data.color_tool == ColorTool::Green,
        Image::new(ImageBuf::from_data(include_bytes!("./svg/icons8-green-circle-48.png")).unwrap()).fix_size(30., 30.).on_click(
            |_ctx, data: &mut Screenshot, _: &Env|{
                data.color_tool = ColorTool::Green;
            }
        ).border( Color::BLACK, 2.).background(Color::GRAY),
        Image::new(ImageBuf::from_data(include_bytes!("./svg/icons8-green-circle-48.png")).unwrap()).fix_size(30., 30.)).on_click(
            |_ctx, data: &mut Screenshot, _: &Env|{
                data.color_tool = ColorTool::Green;
            }
        ));
    row_color.add_child(Either::new(
        |data: &Screenshot, _| data.color_tool == ColorTool::Yellow,
        Image::new(ImageBuf::from_data(include_bytes!("./svg/icons8-yellow-circle-48.png")).unwrap()).fix_size(30., 30.).on_click(
            |_ctx, data: &mut Screenshot, _: &Env|{
                data.color_tool = ColorTool::Yellow;
            }
        ).border( Color::BLACK, 2.).background(Color::GRAY),
        Image::new(ImageBuf::from_data(include_bytes!("./svg/icons8-yellow-circle-48.png")).unwrap()).fix_size(30., 30.)).on_click(
            |_ctx, data: &mut Screenshot, _: &Env|{
                data.color_tool = ColorTool::Yellow;
            }
        ));
    row_color.add_child(Either::new(
        |data: &Screenshot, _| data.color_tool == ColorTool::Blue,
        Image::new(ImageBuf::from_data(include_bytes!("./svg/icons8-blue-circle-48.png")).unwrap()).fix_size(30., 30.).on_click(
            |_ctx, data: &mut Screenshot, _: &Env|{
                data.color_tool = ColorTool::Blue;
            }
        ).border( Color::BLACK, 2.).background(Color::GRAY),
        Image::new(ImageBuf::from_data(include_bytes!("./svg/icons8-blue-circle-48.png")).unwrap()).fix_size(30., 30.)).on_click(
            |_ctx, data: &mut Screenshot, _: &Env|{
                data.color_tool = ColorTool::Blue;
            }
        ));

    

    let pt_box = Stepper::new()
        .with_range(0.0, 100.0)
        .with_step(1.0)
        .lens(Screenshot::line_thickness);

    let label = Label::new(|data: &Screenshot, _: &Env| {
        format!("Line thickness: {} pt", data.line_thickness)
    });

    let mut row_text = Flex::row();
    let textbox = Either::new(
        |data: &Screenshot, _| data.edit_tool == EditTool::Text,
        TextBox::multiline()
        .with_placeholder("Insert text")
        .lens(Screenshot::text)
        .border(Color::BLACK, 2.),
        Label::new("")
    );

    let tick = Either::new(
        |data: &Screenshot, _| data.edit_tool == EditTool::Text && data.editing_text == -1,
        Image::new(ImageBuf::from_data(include_bytes!("./svg/tick-svgrepo-com.png")).unwrap()).fix_size(30., 30.)
        .on_click(
            |_ctx, data: &mut Screenshot, _env|{
                let color = match data.color_tool{
                    ColorTool::Black => Color::BLACK,
                    ColorTool::Red => Color::RED,
                    ColorTool::Blue => Color::BLUE,
                    ColorTool::Yellow => Color::YELLOW,
                    ColorTool::White => Color::WHITE,
                    ColorTool::Green => Color::GREEN,
                };
                data.write.0[data.write.1].color = color;
                data.write.0[data.write.1].thickness = data.line_thickness;
                // data.write.0[data.write.1].position = mouse_event.pos;
                data.write.0[data.write.1].position = Point::new(data.resized_area.x + data.resized_area.width/2., data.resized_area.y + data.resized_area.height/2. );
                data.write.0[data.write.1].text = data.text.clone();
                assign_dimensions_to_textbox(data, data.write.1);
                //SVUOTA
                data.write.0.push_back(Write::new());
                data.text = "".to_string();
                data.write.1 += 1;
            }
        ),
        Label::new("")
    );

    let edit = Either::new(
        |data: &Screenshot, _| data.edit_tool == EditTool::Text && data.editing_text != -1,
        Image::new(ImageBuf::from_data(include_bytes!("./svg/save-svgrepo-com.png")).unwrap()).fix_size(30., 30.)
        .on_click(
            |_ctx, data: &mut Screenshot, _env|{
                let color = match data.color_tool{
                    ColorTool::Black => Color::BLACK,
                    ColorTool::Red => Color::RED,
                    ColorTool::Blue => Color::BLUE,
                    ColorTool::Yellow => Color::YELLOW,
                    ColorTool::White => Color::WHITE,
                    ColorTool::Green => Color::GREEN,
                };
                data.write.0[data.editing_text as usize].text = data.text.clone();
                data.write.0[data.editing_text as usize].color = color;
                data.write.0[data.editing_text as usize].thickness = data.line_thickness;

                assign_dimensions_to_textbox(data, data.editing_text as usize);
                data.editing_text = -1;
                data.text = "".to_string();
            }
        ),
        Label::new("")
    );

    let delete = Either::new(
        |data: &Screenshot, _| data.edit_tool == EditTool::Text && data.editing_text != -1,
        Image::new(ImageBuf::from_data(include_bytes!("./svg/delete-svgrepo-com.png")).unwrap()).fix_size(30., 30.)
        .on_click(
            |_ctx, data: &mut Screenshot, _env|{
               data.write.0.remove(data.editing_text as usize);
               data.write.1 -= 1;
               data.editing_text = -1;
               data.text = "".to_string();
            }
        ),
        Label::new("")
    );

    let mut shape_selector = Flex::row();
    shape_selector.add_child(Either::new(
        |data: &Screenshot, _| data.shape_tool == ShapeTool::Arrow,
        Image::new(ImageBuf::from_data(include_bytes!("./svg/arrow.png")).unwrap()).fix_size(30., 30.).on_click(
            |_ctx, data: &mut Screenshot, _: &Env|{
                data.shape_tool = ShapeTool::Arrow;
            }
        ).border( Color::BLACK, 2.).background(Color::GRAY),
        Image::new(ImageBuf::from_data(include_bytes!("./svg/arrow.png")).unwrap()).fix_size(30., 30.)).on_click(
            |_ctx, data: &mut Screenshot, _: &Env|{
                data.shape_tool = ShapeTool::Arrow;
            }
        ));
    shape_selector.add_default_spacer();
    shape_selector.add_child(Either::new(
        |data: &Screenshot, _| data.shape_tool == ShapeTool::Circle,
        Image::new(ImageBuf::from_data(include_bytes!("./svg/circle.png")).unwrap()).fix_size(30., 30.).on_click(
            |_ctx, data: &mut Screenshot, _: &Env|{
                data.shape_tool = ShapeTool::Circle;
            }
        ).border( Color::BLACK, 2.).background(Color::GRAY),
        Image::new(ImageBuf::from_data(include_bytes!("./svg/circle.png")).unwrap()).fix_size(30., 30.)).on_click(
            |_ctx, data: &mut Screenshot, _: &Env|{
                data.shape_tool = ShapeTool::Circle;
            }
        ));
    shape_selector.add_default_spacer();
    shape_selector.add_child(Either::new(
        |data: &Screenshot, _| data.shape_tool == ShapeTool::Square,
        Image::new(ImageBuf::from_data(include_bytes!("./svg/square.png")).unwrap()).fix_size(30., 30.).on_click(
            |_ctx, data: &mut Screenshot, _: &Env|{
                data.shape_tool = ShapeTool::Square;
            }
        ).border( Color::BLACK, 2.).background(Color::GRAY),
        Image::new(ImageBuf::from_data(include_bytes!("./svg/square.png")).unwrap()).fix_size(30., 30.)).on_click(
            |_ctx, data: &mut Screenshot, _: &Env|{
                data.shape_tool = ShapeTool::Square;
            }
        ));
    
    let row_shape = Either::new(
        |data: &Screenshot, _| data.edit_tool == EditTool::Shape,
        shape_selector.border(Color::GRAY, 2.),
        Label::new("")
    );


    row_text.add_child(textbox);
    row_text.add_default_spacer();
    row_text.add_child(tick);
    row_text.add_child(edit);
    row_text.add_default_spacer();
    row_text.add_child(delete);

    row_color.add_default_spacer();
    row_color.add_child(label);
    row_color.add_spacer(1.);
    row_color.add_child(pt_box);

    row.add_default_spacer();
    row.add_default_spacer();
    row.add_child(pencil);
    row.add_default_spacer();
    row.add_child(highlighter);
    row.add_default_spacer();
    row.add_child(shapes);
    row.add_default_spacer();
    row.add_child(text);
    row.add_default_spacer();
    row.add_child(eraser);
    row.add_default_spacer();
    row.add_default_spacer();
    row.add_child(row_color.border(Color::GRAY, 2.));
    row.add_default_spacer();
    row.add_default_spacer();

    let mut col = Flex::column();
    col.add_child(row);
    col.add_spacer(5.);
    col.add_child(row_text);
    col.add_child(row_shape);

    col
}

pub fn show_screen(
    _ctx: &mut EventCtx,
    image: ImageBuf,
    data: &mut Screenshot,
) -> impl Widget<Screenshot> {
    data.flag_edit = false;
    data.draw.points.clear();
    data.draw.points.push_back((im::Vector::new(), Color::WHITE, 1., 1.));
    data.draw.segment = 0;
    data.flag_resize = false;
    data.reset_resize_rect();
    let original_x = data.resized_area.x;
    let original_y = data.resized_area.y;
    let original_w = data.resized_area.width;
    let original_h = data.resized_area.height;

    let img = Image::new(image.clone()).fill_mode(FillStrat::ScaleDown);

    let mut col = Flex::column();
    let mut row_button1 = Flex::row();
    let mut row_button2 = Flex::row();
    let row_toolbar = build_toolbar();

    let sizedbox = SizedBox::new(img).width(1000.).height(562.5);

    let resize_button =
        Button::new("resize").on_click(move |_ctx: &mut EventCtx, data: &mut Screenshot, _env| {
            data.flag_resize = true;
        });

    let cancel_button =
        Button::new("cancel").on_click(move |_ctx: &mut EventCtx, data: &mut Screenshot, _env| {
            data.flag_resize = false;
            data.reset_resize_rect();
        });

    let edit_button =
    Button::new("edit").on_click(move |_ctx: &mut EventCtx, data: &mut Screenshot, _env| {
        data.flag_edit = true;
    });

    let copy_button = Button::new("copy to clipboard").on_click(
        move |_ctx: &mut EventCtx, data: &mut Screenshot, _env| {
            let mut clip = Clipboard::new().unwrap();
            let formatted: ImageData = ImageData {
                width: data.img.width(),
                height: data.img.height(),
                bytes: Cow::from(data.img.raw_pixels()),
            };
            clip.set_image(formatted).unwrap();
        },
    );

    let update_button =
        Button::new("update").on_click(move |ctx: &mut EventCtx, data: &mut Screenshot, _env| {
            let image1: ImageBuffer<image::Rgba<u8>, Vec<u8>> = ImageBuffer::from_vec(
                data.img.width() as u32,
                data.img.height() as u32,
                data.img.raw_pixels().to_vec(),
            )
            .unwrap();

            let dynamic_image = DynamicImage::ImageRgba8(image1);
            let new = dynamic_image.crop_imm(
                ((data.resized_area.x - original_x) * (data.img.width() as f64 / original_w))
                    as u32,
                ((data.resized_area.y - original_y) * (data.img.height() as f64 / original_h))
                    as u32,
                (data.resized_area.width * (data.img.width() as f64 / original_w)) as u32,
                (data.resized_area.height * (data.img.height() as f64 / original_h)) as u32,
            );
            let image2 = new.to_rgba8();

            data.img = ImageBuf::from_raw(
                image2.clone().into_raw(),
                druid::piet::ImageFormat::RgbaPremul,
                image2.clone().width() as usize,
                image2.clone().height() as usize,
            );

            data.screen_window(ctx);
            ctx.window().close();
        });

    let button1 = Either::new(
        |data: &Screenshot, _: &Env| data.flag_resize,
        cancel_button,
        copy_button,
    );

    let button2 = Either::new(
        |data: &Screenshot, _: &Env| data.flag_resize,
        update_button,
        resize_button,
    );

    let button3 = Either::new(
        |data: &Screenshot, _: &Env| data.flag_resize,
        Label::new(""),
        edit_button,
    );

    let save = Button::new("Save").on_click(
        |_ctx, data: &mut Screenshot, _env: &Env|{

            // let mut bottom: ImageBuffer<image::Rgba<u8>, Vec<u8>> = ImageBuffer::from_vec(
            //     data.img.width() as u32,
            //     data.img.height() as u32,
            //     data.img.raw_pixels().to_vec(),
            // )
            // .unwrap();

            // let mut top: ImageBuffer<image::Rgba<u8>, Vec<u8>> = ImageBuffer::from_vec(
            //     data.squares.0[0].image.width() as u32,
            //     data.squares.0[0].image.height() as u32,
            //     data.squares.0[0].image.raw_pixels().to_vec(),
            // )
            // .unwrap();

            // let screens = Screen::all().unwrap();
            // let image: ImageBuffer<Rgba<u8>, Vec<u8>> = screens[0].capture().unwrap();


            // // let top1 = ImageBuf::from_raw(
            // //     image.clone().into_raw(),
            // //     druid::piet::ImageFormat::RgbaPremul,
            // //     image.clone().width() as usize,
            // //     image.clone().height() as usize,
            // // );

            // // let mut top: ImageBuffer<image::Rgba<u8>, Vec<u8>> = ImageBuffer::from_vec(
            // //     top1.width() as u32,
            // //     top1.height() as u32,
            // //     top1.raw_pixels().to_vec(),
            // // )
            // // .unwrap();
    
                
            // image::imageops::overlay(&mut bottom, &mut top, 0, 0);
            // data.img = ImageBuf::from_raw(
            //     bottom.clone().into_raw(),
            //     druid::piet::ImageFormat::RgbaPremul,
            //     bottom.clone().width() as usize,
            //     bottom.clone().height() as usize,
            // ); 
            
        }
    );

    let cancel = Button::new("Cancel").on_click(
        |_ctx, data: &mut Screenshot, _env: &Env|{
            data.flag_edit = false;
            data.draw.points.clear();
            data.draw.points.push_back((im::Vector::new(), Color::WHITE, 1., 1.));
            data.draw.segment = 0;

            data.write.0.clear();
            data.write.0.push_back(Write::new());
            data.write.1 = 0;

            data.arrows.0.clear();
            data.arrows.0.push_back(Arrow::new());
            data.arrows.1 = 0;

            data.circles.0.clear();
            data.circles.0.push_back(Circle::new());
            data.circles.1 = 0;

            data.squares.0.clear();
            data.squares.0.push_back(Square::new());
            data.squares.1 = 0;
        }
    );

    row_button1.add_child(button2);
    row_button1.add_child(button1);
    row_button1.add_child(button3);
    row_button2.add_child(save);
    row_button2.add_child(cancel);
    col.add_child(Either::new(
        |data: &Screenshot, _: &Env| data.flag_edit,
        row_button2,
        row_button1,
    ));

    let zstack_layout = ZStack::new(sizedbox).with_centered_child(Either::new(
        |data: &Screenshot, _: &Env| data.flag_resize,
        draw_resize(data),
        Either::new(
            |data: &Screenshot, _: &Env| data.flag_edit,
            manage_edit(data),
            druid::widget::Label::new(""),
        ),
    ));

    // // let size = druid::kurbo::Size::new(800.0, 600.0); // Sostituisci con le dimensioni desiderate.
    // let mut image = ImageBuffer::new(data.img.width() as u32, data.img.height() as u32);
    // zstack_layout.(ctx, data, env)
    // // Creare un RenderContext basato sull'immagine.
    // // let mut ctx = RenderContext::Image

    // // Disegnare il widget sull'immagine.
    // widget.draw(&mut ctx, &druid::Env::default(), size.to_rect());

    col.add_default_spacer();
    col.add_default_spacer();
    col.add_child(zstack_layout);
    col.add_default_spacer();
    col.add_default_spacer();
    col.add_child(Either::new(
        |data: &Screenshot, _: &Env| data.flag_edit,
        row_toolbar,
        Label::new(""),
    ));
    col
}

pub fn draw_rect() -> impl Widget<Screenshot> {
    let paint = Painter::new(|ctx: &mut PaintCtx<'_, '_, '_>, data: &Screenshot, _env| {
        let (start, end) = (data.area.start, data.area.end);
        let rect = druid::Rect::from_points(start, end);

        ctx.fill(
            rect,
            &Color::rgba(
                data.area.rgba.r,
                data.area.rgba.g,
                data.area.rgba.b,
                data.area.rgba.a,
            ),
        );
        // ctx.stroke(rect, &druid::Color::RED, 0.8);
    })
    .controller(MouseClickDragController {
        t1: TimerToken::next(),
        flag: true,
    })
    .center();

    // Flex::column().with_child(paint)
    paint
}

pub fn draw_resize(data: &Screenshot) -> impl Widget<Screenshot>{
    Painter::new(|ctx, data: &Screenshot, _env| {
        let rect = druid::Rect::from_points(
            (data.resized_area.x, data.resized_area.y),
            (
                data.resized_area.x + data.resized_area.width,
                data.resized_area.y + data.resized_area.height,
            ),
        );
        ctx.fill(rect, &Color::rgba(0.0, 0.0, 0.0, 0.5));
        ctx.stroke(rect, &druid::Color::RED, 2.0);
    })
    .center()
    .controller(ResizeController {
        selected_part: ResizeInteraction::NoInteraction,
        original_area: ResizedArea::new_parameter(
            data.resized_area.x,
            data.resized_area.y,
            data.resized_area.width,
            data.resized_area.height,
        ),
    })
}

pub fn manage_edit(data: &mut Screenshot) -> impl Widget<Screenshot>{

    let paint = Painter::new(move |ctx: &mut PaintCtx<'_, '_, '_>, data: &Screenshot, env| {
        
            //GESTIONE DRAW
            let color = match data.color_tool{
                ColorTool::Black => Color::BLACK,
                ColorTool::Red => Color::RED,
                ColorTool::Blue => Color::BLUE,
                ColorTool::Yellow => Color::YELLOW,
                ColorTool::White => Color::WHITE,
                ColorTool::Green => Color::GREEN,
            };

            let point0 = Point::new(0.0, 0.0);
            let mut path = druid::kurbo::BezPath::new();

            path.move_to(data.draw.points[data.draw.segment].0.head().unwrap_or(&point0).clone());
            for point in data.draw.points[data.draw.segment].0.iter().skip(1) {
                path.line_to(point.clone());
            }
            let brush = ctx.solid_brush(color.with_alpha(data.draw.points[data.draw.segment].3));
            ctx.stroke(path, &brush, data.line_thickness);

            for i in 0..data.draw.segment{
                let mut path = druid::kurbo::BezPath::new();
                path.move_to(data.draw.points[i].0.head().unwrap_or(&point0).clone());
                for point in data.draw.points[i].0.iter().skip(1) {
                    path.line_to(point.clone());
                }
                let brush = ctx.solid_brush(data.draw.points[i].1.with_alpha(data.draw.points[i].3));
                ctx.stroke(path, &brush, data.draw.points[i].2);

                // let mut dynamic = DynamicImage::ImageRgba8(path);
                // let rgba_buffer = dynamic.as_mut_rgba8().expect("failed to get rgba buffer").as_raw().as_slice();

                // // let context = ImageBuffer::from_raw(100, 100, rgba_buffer).unwrap();
                
                // let top = ctx.make_image(100, 100, rgba_buffer, piet::ImageFormat::RgbaPremul).unwrap();


                // let image1: ImageBuffer<image::Rgba<u8>, Vec<u8>> = ImageBuffer::from_vec(
                //     data.img.width() as u32,
                //     data.img.height() as u32,
                //     data.img.raw_pixels().to_vec(),
                // )
                // .unwrap();
    
                // let mut bottom = DynamicImage::ImageRgba8(image1);
                // image::imageops::overlay(&mut image1, &mut dynamic, 0, 0);
            }

            //GESTIONE WRITE
            let text = data.write.0[data.write.1].text.clone();
            let text_layout = ctx.text().new_text_layout(text.clone())
                .font(FontFamily::MONOSPACE, data.write.0[data.write.1].thickness)
                .text_color(data.write.0[data.write.1].color)
                .build()
                .unwrap();
        
            ctx.draw_text(&text_layout, data.write.0[data.write.1].position);
            if data.editing_text != -1 && data.text != "" {
                let start = data.write.0[data.editing_text as usize].position;
                let txt_w =  data.write.0[data.editing_text as usize].dimensions.0;
                let txt_h =  data.write.0[data.editing_text as usize].dimensions.1;
                let end = Point::new(data.write.0[data.editing_text as usize].position.x + txt_w, data.write.0[data.editing_text as usize].position.y + txt_h);
                ctx.stroke(druid::Rect::from_points(start, end), &data.write.0[data.editing_text as usize].color, 2.);
            }

            for write in data.write.0.clone(){
                let text_layout = ctx.text().new_text_layout(write.text)
                .font(FontFamily::MONOSPACE, write.thickness)
                .text_color(write.color)
                .build()
                .unwrap();
                ctx.draw_text(&text_layout, write.position);
            }

            //GESTIONE ARROW          
            let start = data.arrows.0[data.arrows.1].start;
            let end = data.arrows.0[data.arrows.1].end;

            let direction = (end - start).normalize(); // Calcola la direzione della freccia
            let direction2 = druid::kurbo::Vec2::from_angle(direction.angle() + 50.);
            let direction3 = druid::kurbo::Vec2::from_angle(direction.angle() - 50.);

            let len = end.distance(start);
            let arrow_base1 = end - direction2 * len*1./3.;
            let arrow_base2 = end - direction3 * len*1./3.;
            
            let mut path2 = druid::kurbo::BezPath::new();
            path2.move_to(start);
            path2.line_to(end);
            path2.move_to(end);
            path2.line_to(arrow_base1);
            path2.move_to(end);
            path2.line_to(arrow_base2);
            ctx.stroke(path2, &color, data.line_thickness);
            
            for arrow in data.arrows.0.clone(){
                let start = arrow.start;
                let end = arrow.end;
                
                let direction = (end - start).normalize(); // Calcola la direzione della freccia
                let direction2 = druid::kurbo::Vec2::from_angle(direction.angle() + 50.);
                let direction3 = druid::kurbo::Vec2::from_angle(direction.angle() - 50.);

                let len = end.distance(start);
                let arrow_base1 = end - direction2 * len*1./3.;
                let arrow_base2 = end - direction3 * len*1./3.;
                
                let mut path2 = druid::kurbo::BezPath::new();
                path2.move_to(start);
                path2.line_to(end);
                path2.move_to(end);
                path2.line_to(arrow_base1);
                path2.move_to(end);
                path2.line_to(arrow_base2);
                ctx.stroke(path2, &arrow.color, arrow.thickness);
            }

            //GESTIONE CIRCLE
            let rect = druid::Rect::from_points(data.circles.0[data.circles.1].start, data.circles.0[data.circles.1].end);
            let ellipse = druid::kurbo::Ellipse::from_rect(rect);
            ctx.stroke(ellipse, &color, data.line_thickness);
        
            for circle in data.circles.0.clone(){
                let rect = druid::Rect::from_points(circle.start, circle.end);
                let ellipse = druid::kurbo::Ellipse::from_rect(rect);
                ctx.stroke(ellipse, &circle.color, circle.thickness);
            }

            //GESTIONE SQUARE
            if data.squares.0[data.squares.1].start != data.squares.0[data.squares.1].end {
                let rect = druid::Rect::from_points(data.squares.0[data.squares.1].start, data.squares.0[data.squares.1].end);
                ctx.stroke(rect, &color, data.line_thickness);
            }

            for square in data.squares.0.clone(){
                if square.start != square.end{
                    let rect = druid::Rect::from_points(square.start, square.end);
                    ctx.stroke(rect, &square.color, square.thickness);

                    // let mut  image1: ImageBuffer<image::Rgba<u8>, Vec<u8>> = ImageBuffer::from_vec(
                    //     data.painter.width() as u32,
                    //     data.painter.height() as u32,
                    //     data.painter.raw_pixels().to_vec(),
                    // )
                    // .unwrap();

                    
                    
                    // assign_painter_img(&mut data.clone(), image1);
                    
                }
            }
            
            


    })
    .controller(Drawer {
        flag_drawing: false,
        flag_writing: false,
        first_click_pos: Point::new(0., 0.),
    })
    .center();

    paint
}

fn assign_dimensions_to_textbox(data: &mut Screenshot, index: usize) {
    let str = data.text.clone();
    let mut numlines = 0.; 
    let mut longest = 0;
    for line in str.split_terminator('\n'){
        numlines+=1.;
        if line.len() > longest {
            longest = line.len()
        }
    }
    
    //assign dimensions to the virtual text box to be lately clickable
    data.write.0[index].dimensions = (data.write.0[index].thickness * longest as f64 * 0.555, data.write.0[index].thickness * numlines * 1.35);

}

// fn assign_painter_img(data: &mut Screenshot, image: ImageBuffer<image::Rgba<u8>, Vec<u8>>){
//     data.painter = 
//         ImageBuf::from_raw(
//             image.clone().into_raw(),
//             druid::piet::ImageFormat::RgbaPremul,
//             image.clone().width() as usize,
//             image.clone().height() as usize,
//         );
// }