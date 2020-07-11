use std::f64::consts::PI;
use std::rc::Rc;

use druid::kurbo::{Circle, CircleSegment, Line, Vec2};
//use druid::piet::{FontBuilder, ImageFormat, InterpolationMode, Text, TextLayoutBuilder};

use druid::{
    Affine, AppLauncher, BoxConstraints, Color, Env, Event, EventCtx, LayoutCtx, LifeCycle,
    LifeCycleCtx, PaintCtx, Point, Rect, RenderContext, Size, UpdateCtx, Widget,
};

use crate::ddrescue::{MapFile, BlockStatus};

pub struct FloppyView;

impl FloppyView {
    pub fn status_color(status: BlockStatus) -> Color {
        match status {
            BlockStatus::NonTried => Color::rgb8(128, 128, 128),
            BlockStatus::NonTrimmed => Color::rgb8(240, 240, 0),
            BlockStatus::NonScraped => Color::rgb8(0, 0, 240),
            BlockStatus::BadSector => Color::rgb8(240, 0, 0),
            BlockStatus::Finished => Color::rgb8(0, 240, 0),
        }
    }
}

impl Widget<Rc<MapFile>> for FloppyView {
    fn event(&mut self, _ctx: &mut EventCtx, _event: &Event, _data: &mut Rc<MapFile>, _env: &Env) {

    }

    fn lifecycle(
        &mut self,
        _ctx: &mut LifeCycleCtx,
        _event: &LifeCycle,
        _data: &Rc<MapFile>,
        _env: &Env,
    ) {
    }

    fn update(&mut self, _ctx: &mut UpdateCtx, _old_data: &Rc<MapFile>, _data: &Rc<MapFile>, _env: &Env) {

    }

    fn layout(
        &mut self,
        _layout_ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        _data: &Rc<MapFile>,
        _env: &Env,
    ) -> Size {
        bc.max()
    }

    fn paint(&mut self, paint: &mut PaintCtx, data: &Rc<MapFile>, _env: &Env) {
        let margin = 10.0;

        let size = paint.size();
        let radius = (size.width - margin * 3.0) / 4.0;
        let center0 = Point::new(margin + radius, margin + radius);
        let side_step = radius * 2.0 + margin;
        let floppy = data.floppy_type();

        let track_radius = radius * 0.1;
        let track_step = (radius - track_radius) / floppy.tracks as f64;
        let sector_angle = 2.0 * PI / floppy.sectors as f64;

        //let side_center = |side: u64| center0 + Vec2::new(0.0, side_step * side as f64);
        let side_center = |side: u64| center0 + Vec2::new(side_step * side as f64, 0.0);

        // Background
        paint.clear(Color::WHITE);

        // Disk backgrounds
        for side in 0..floppy.sides {
            paint.fill(
                Circle::new(side_center(side), radius)
                    .segment(radius * 0.05, 0.0, 2.0 * PI),
                &Self::status_color(BlockStatus::NonTried));
        }

        // Sector status
        for sector in data.sectors().filter(|s| s.status != BlockStatus::NonTried) {
            let r = radius - (sector.track + 1) as f64 * track_step;
            let a = (sector.sector - 1) as f64 * sector_angle;

            paint.fill(
                CircleSegment::new(
                    side_center(sector.side),
                    r + track_step,
                    r,
                    a,
                    sector_angle),
                &Self::status_color(sector.status));
        }

        for side in 0..floppy.sides {
            let center = side_center(side);

            // Sector radius lines
            for sector in 0..floppy.sectors {
                let angle = sector as f64 * sector_angle;

                paint.stroke(
                    Line::new(
                        center + Vec2::new(angle.cos() * track_radius, -angle.sin() * track_radius),
                        center + Vec2::new(angle.cos() * radius, -angle.sin() * radius)),
                    &Color::rgba8(0, 0, 0, 64),
                    1.0);
            }

            // Track borders
            /*for track in 0..=floppy.tracks {
                paint.stroke(
                    Circle::new(center, track_base + track_step * track as f64),
                    &Color::rgba8(0, 0, 0, 128),
                    0.5);
            }*/
        }
    }
}