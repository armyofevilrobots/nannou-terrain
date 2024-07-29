use nannou::{
    noise::{HybridMulti, NoiseFn /*Perlin*/},
    prelude::*,
};
use nannou_egui::{egui, Egui};
mod terrain;
use std::time::{Duration, Instant};
use terrain::*;

fn main() {
    nannou::app(model)
        .update(update)
        .loop_mode(LoopMode::rate_fps(30.0))
        .run();
}

struct Model {
    altitude: f64,
    triangles: Vec<(DVec3, DVec3, DVec3)>, //::new(),
    noise_gen: HybridMulti,
    egui: Egui,
    span: usize,
    show_triangles: bool,
    grid_spacing: f32,
    height: f64,
    height_step: f64,
    noise_scale: f64,
    background: Hsv,
    color_start: Hsv,
    color_end: Hsv,
    last_tick: Instant,
}

fn model(app: &App) -> Model {
    let window_id = app
        .new_window()
        .view(view)
        .raw_event(raw_window_event)
        .build()
        .unwrap();
    let window = app.window(window_id).unwrap();
    let egui = Egui::from_window(&window);
    Model {
        altitude: 0.0,
        triangles: Vec::new(),
        noise_gen: HybridMulti::new(),
        egui: egui,
        span: 20,
        show_triangles: false,
        grid_spacing: 24.,
        height: 16.,
        height_step: 1.8,
        noise_scale: 1.,
        background: hsv(10.0, 0.0, 1.0),
        color_start: hsv(1.0, 1., 0.1),
        color_end: hsv(1.0, 0., 1.0),
        last_tick: Instant::now(),
    }
}

fn raw_window_event(_app: &App, model: &mut Model, event: &nannou::winit::event::WindowEvent) {
    // Let egui handle things like keyboard and mouse input.
    model.egui.handle_raw_event(event);
}

fn edit_hsv(ui: &mut egui::Ui, color: &mut Hsv) {
    let mut egui_hsv = egui::ecolor::Hsva::new(
        color.hue.to_positive_radians() as f32 / (std::f32::consts::PI * 2.0),
        color.saturation,
        color.value,
        1.0,
    );

    if egui::color_picker::color_edit_button_hsva(
        ui,
        &mut egui_hsv,
        egui::color_picker::Alpha::Opaque,
    )
    .changed()
    {
        *color = nannou::color::hsv(egui_hsv.h, egui_hsv.s, egui_hsv.v);
    }
}

fn update(_app: &App, model: &mut Model, update: Update) {
    let now = Instant::now();
    let duration = now - model.last_tick;
    model.last_tick = now;

    let egui = &mut model.egui;
    egui.set_elapsed_time(update.since_start);
    let ctx = egui.begin_frame();
    egui::Window::new("Settings").show(&ctx, |ui| {
        ui.label(format!("Frame time: {:4}ms", duration.as_millis()));
        ui.label("Grid Size:");
        ui.add(egui::Slider::new(&mut model.span, 1..=64));
        ui.add(egui::Checkbox::new(
            &mut model.show_triangles,
            "Show Triangles?",
        ));
        ui.label("Grid Spacing");
        ui.add(egui::Slider::new(&mut model.grid_spacing, 8.0..=64.0));
        ui.label("Noise Scale");
        ui.add(egui::Slider::new(&mut model.noise_scale, 0.1..=4.0));
        ui.label("Terrain Height");
        ui.add(egui::Slider::new(&mut model.height, 2.00..=32.00));
        ui.label("Terrain contour step");
        ui.add(egui::Slider::new(&mut model.height_step, 0.1..=8.0));
        ui.label("Background");
        edit_hsv(ui, &mut model.background);
        ui.horizontal(|ui| {
            ui.label("Bottom color");
            edit_hsv(ui, &mut model.color_start);
            ui.label("Top color");
            edit_hsv(ui, &mut model.color_end);
        });
    });

    let triangles = generate_terrain(
        model.span,
        model.span,
        model.grid_spacing as f64,
        model.altitude,
        model.height,
        &model.noise_gen,
        model.noise_scale,
    );
    // let triangles = triangles;
    //     .into_iter()
    //     .map(|tri| triangle_dz_decompose(&tri, model.height / 4.))
    //     .flatten()
    //     .collect();

    model.triangles = triangles;
    model.altitude = model.altitude + 0.005;
}

fn view(app: &App, model: &Model, frame: Frame) {
    frame.clear(model.background);
    let draw = app.draw();
    let (width, height) = frame.rect().w_h();
    // println!("W/H: {}/{}", width, height);
    if model.show_triangles {
        for triangle in &model.triangles {
            // println!("Triangle: {:?}.", triangle);
            draw.tri()
                .stroke(BLACK)
                .stroke_weight(1.)
                .color(WHITE)
                .points(
                    Point2::new(
                        triangle.0.x as f32 - (model.span as f32 * model.grid_spacing) / 2.,
                        triangle.0.y as f32 - (model.span as f32 * model.grid_spacing) / 2.,
                    ),
                    Point2::new(
                        triangle.1.x as f32 - (model.span as f32 * model.grid_spacing) / 2.,
                        triangle.1.y as f32 - (model.span as f32 * model.grid_spacing) / 2.,
                    ),
                    Point2::new(
                        triangle.2.x as f32 - (model.span as f32 * model.grid_spacing) / 2.,
                        triangle.2.y as f32 - (model.span as f32 * model.grid_spacing) / 2.,
                    ),
                );
            // println!("RADIUS: {}", triangle.0.z);
            // draw.ellipse()
            //     .stroke(BLACK)
            //     .stroke_weight(1.0)
            //     .color(WHITE)
            //     .radius(triangle.0.z as f32)
            //     .x_y(
            //         triangle.0.x as f32 - (model.span as f32 * model.grid_spacing) / 2.,
            //         triangle.0.y as f32 - (model.span as f32 * model.grid_spacing) / 2.,
            //     );
        }
    }
    let ofs = 0.; //(model.grid_spacing / 2.) as f32;
    let mut h = 0.0;
    while h < model.height {
        // println!("h/height: {}/{}={}", h, height, h as f32/model.height as f32);
        let lines = lines_from_terrain(&model.triangles, h as f64);
        let pc = h as f32 / model.height as f32;
        let color_h = model.color_start.hue.to_positive_degrees() / 360. * (1. - pc)
            + (model.color_end.hue.to_positive_degrees() / 360. * pc);
        let color_s = model.color_start.saturation * (1. - pc) + model.color_end.saturation * pc;
        let color_v = model.color_start.value * (1. - pc) + model.color_end.value * pc;
        let color = hsv(color_h, color_s, color_v);
        for line in lines {
            draw.line()
                .stroke_weight(1.5)
                .color(color)
                .start(pt2(
                    ofs + line.0.x as f32 - (model.span as f32 * model.grid_spacing) / 2.,
                    ofs + line.0.y as f32 - (model.span as f32 * model.grid_spacing) / 2.,
                ))
                .end(pt2(
                    ofs + line.1.x as f32 - (model.span as f32 * model.grid_spacing) / 2.,
                    ofs + line.1.y as f32 - (model.span as f32 * model.grid_spacing) / 2.,
                ));
        }
        h += model.height_step;
    }

    draw.to_frame(app, &frame).unwrap();
    model.egui.draw_to_frame(&frame).unwrap();
}
