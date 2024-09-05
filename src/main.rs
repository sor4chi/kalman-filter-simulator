extern crate rand;
extern crate svg;

use std::fs::File;
use std::io::BufWriter;

use image::{Frame, RgbaImage};
use rand::Rng;
use resvg::tiny_skia::Pixmap;
use resvg::usvg;
use svg::node::element::{Circle, Line};
use svg::Document;

#[derive(Debug, Clone, Copy)]
struct State {
    x: f64,
    v: f64,
}

struct KalmanFilter {
    state: State,
    p: f64,
    r: f64,
    q: f64,
    k: f64,
}

impl KalmanFilter {
    fn new(initial_position: f64, initial_velocity: f64, r: f64, q: f64) -> Self {
        KalmanFilter {
            state: State {
                x: initial_position,
                v: initial_velocity,
            },
            p: 1.0,
            r,
            q,
            k: 0.0,
        }
    }

    fn predict(&mut self, dt: f64) {
        self.state.x += self.state.v * dt;
        self.p += self.q;
    }

    fn update(&mut self, measured_position: f64) {
        self.k = self.p / (self.p + self.r);
        self.state.x += self.k * (measured_position - self.state.x);
        self.p *= 1.0 - self.k;
    }
}

#[derive(Default)]
struct SimulateTick {
    true_positions: (f64, f64),
    measured_positions: (f64, f64),
    estimated_positions: (f64, f64),
}

#[derive(Default)]
struct SimulateResult {
    ticks: Vec<SimulateTick>,
}

fn simulate(
    total_time: f64,
    dt: f64,
    velocity: f64,
    sensor_noise_stddev: f64,
    r: f64,
    q: f64,
) -> SimulateResult {
    let steps = (total_time / dt) as usize;

    let mut true_position = 0.0;

    let mut kalman = KalmanFilter::new(0.0, velocity, r, q);

    let mut rng = rand::thread_rng();

    let mut result = SimulateResult::default();

    for step in 0..steps {
        let time = step as f64 * dt;

        true_position += velocity * dt;

        let noise: f64 = rng.gen_range(-sensor_noise_stddev..sensor_noise_stddev);
        let measured_position = true_position + noise;

        kalman.predict(dt);
        kalman.update(measured_position);

        let tick = SimulateTick {
            true_positions: (time, true_position),
            measured_positions: (time, measured_position),
            estimated_positions: (time, kalman.state.x),
        };

        result.ticks.push(tick);
    }

    result
}

fn render(
    true_positions: &[(f64, f64)],
    measured_positions: &[(f64, f64)],
    estimated_positions: &[(f64, f64)],
    size: usize,
    scale: f64,
) -> Document {
    let mut document = Document::new()
        .set("viewBox", (0, 0, size, size))
        .set("width", "500")
        .set("height", "500");

    let background = svg::node::element::Rectangle::new()
        .set("x", 0)
        .set("y", 0)
        .set("width", size)
        .set("height", size)
        .set("fill", "white");
    document = document.add(background);

    for i in 1..true_positions.len() {
        let (x1, y1) = true_positions[i - 1];
        let (x2, y2) = true_positions[i];
        let line = Line::new()
            .set("x1", x1 * scale)
            .set("y1", size as f64 - y1 * scale)
            .set("x2", x2 * scale)
            .set("y2", size as f64 - y2 * scale)
            .set("stroke", "red")
            .set("stroke-width", 2);
        document = document.add(line);
    }

    for i in 1..estimated_positions.len() {
        let (x1, y1) = estimated_positions[i - 1];
        let (x2, y2) = estimated_positions[i];
        let line = Line::new()
            .set("x1", x1 * scale)
            .set("y1", size as f64 - y1 * scale)
            .set("x2", x2 * scale)
            .set("y2", size as f64 - y2 * scale)
            .set("stroke", "green")
            .set("stroke-width", 2);
        document = document.add(line);
    }

    for (x, y) in measured_positions {
        let circle = Circle::new()
            .set("cx", x * scale)
            .set("cy", size as f64 - y * scale)
            .set("r", 2.0)
            .set("fill", "blue");
        document = document.add(circle);
    }

    document
}

fn animate(result: SimulateResult, size: usize, scale: f64) -> Vec<Frame> {
    let mut frames = Vec::new();
    let options = usvg::Options::default();

    let mut true_positions = Vec::new();
    let mut measured_positions = Vec::new();
    let mut estimated_positions = Vec::new();
    for (i, tick) in result.ticks.iter().enumerate() {
        if i % 10 == 9 {
            eprintln!("{}/{} frames", i + 1, result.ticks.len());
        }
        true_positions.push(tick.true_positions);
        measured_positions.push(tick.measured_positions);
        estimated_positions.push(tick.estimated_positions);

        let document = render(
            &true_positions,
            &measured_positions,
            &estimated_positions,
            size,
            scale,
        );

        let svg = document.to_string();
        let tree = usvg::Tree::from_str(&svg, &options).unwrap();
        let mut pixmap = Pixmap::new(500, 500).unwrap();
        resvg::render(&tree, tiny_skia::Transform::default(), &mut pixmap.as_mut());

        let image = RgbaImage::from_raw(500, 500, pixmap.data().to_vec()).unwrap();
        frames.push(Frame::new(image));
    }

    frames
}

fn main() {
    // === Parameters ===
    let total_time = 10.0;
    let dt = 0.1;
    let velocity = 1.0;
    let sensor_noise_stddev: f64 = 2.0;
    let r = sensor_noise_stddev.powi(2);
    let q = 0.01;
    // ==================

    let size = 500;
    let scale = size as f64 / total_time;

    eprintln!("Simulating...");
    let result = simulate(total_time, dt, velocity, sensor_noise_stddev, r, q);

    eprintln!("Rendering frames...");
    let animation = animate(result, size, scale);

    let output_file = File::create("output.gif").unwrap();
    let writer = BufWriter::new(output_file);

    eprintln!("Encoding GIF...");
    let mut encoder = image::codecs::gif::GifEncoder::new(writer);
    encoder.encode_frames(animation).unwrap();

    println!("Output saved to output.gif");
}
