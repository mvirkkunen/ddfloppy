use std::env;
use std::fs::File;
use std::io::BufReader;
use std::rc::Rc;

use druid::widget::{Button, Flex, Label};
use druid::{AppLauncher, Data, Lens, PlatformError, Widget, WidgetExt, WindowDesc};

mod ddrescue;
use ddrescue::*;

mod floppy_type;
use floppy_type::FloppyType;

mod floppy_view;
use floppy_view::FloppyView;

fn main() -> Result<(), PlatformError> {
    let filename = env::args().nth(1).unwrap();
    let reader = BufReader::new(File::open(filename).unwrap());
    let map = MapFile::load(reader, None).unwrap();

    /*println!("{:?}", map);

    for sector in map.sectors() {
        println!("{:04} {:08x} {:?}", sector.index, sector.pos, sector.status);
    }*/

    let window = WindowDesc::new(build_ui)
        .title("ddfloppy");

    let state = AppState {
        map_file: Rc::new(map),
    };

    AppLauncher::with_window(window)
        .use_simple_logger()
        .launch(state)
}

#[derive(Clone, Data, Lens)]
struct AppState {
    map_file: Rc<MapFile>,
}

fn build_ui() -> impl Widget<AppState> {
    // The label text will be computed dynamically based on the current locale and count
    /*let text =
        LocalizedString::new("hello-counter").with_arg("count", |data: &u32, _env| (*data).into());
    let label = Label::new(text).padding(5.0).center();
    let button = Button::new("increment")
        .on_click(|_ctx, data, _env| *data += 1)
        .padding(5.0);

    Flex::column().with_child(label).with_child(button)*/

    FloppyView.lens(AppState::map_file)
}
