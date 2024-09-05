# Kalman Filter Simulator

This is a simple simulator for the Kalman Filter algorithm.

YOu can configure the parameters of the simulation in the `src/main.rs` file.

```rs
let total_time = 10.0;
let dt = 0.1;
let velocity = 1.0;
let sensor_noise_stddev: f64 = 2.0;
let r = sensor_noise_stddev.powi(2);
let q = 0.01;
```

The simulator will generate a noisy signal and apply the Kalman Filter to estimate the true signal.

To run the simulation, execute the following command:

```sh
cargo run --release
```

## Showcase

![Kalman Filter Simulator](kalman_filter_simulator.gif)
