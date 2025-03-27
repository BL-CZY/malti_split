use std::io::BufRead;

use hound::WavReader;
use serde::Serialize;

fn detect_stops(
    file_path: &str,
    silence_threshold: i16,
    min_silence_duration: f64,
) -> Result<Vec<f64>, Box<dyn std::error::Error>> {
    // Open the WAV file
    let mut reader = WavReader::open(file_path)?;
    let spec = reader.spec();

    // Prepare to read samples
    let samples = reader.samples::<i16>();
    let sample_rate = spec.sample_rate as f64;

    // Convert parameters to sample-based measurements
    let samples_per_second = sample_rate;
    let min_silence_samples = (min_silence_duration * samples_per_second) as usize;

    let mut stops = Vec::new();
    let mut current_silence_duration = 0usize;
    let mut current_sample_time = 0f64;

    // Iterate through samples
    for sample in samples {
        let sample_value = sample.unwrap().abs();

        // Check if sample is below silence threshold
        if sample_value < silence_threshold {
            current_silence_duration += 1;

            // Check if silence duration exceeds minimum
            if current_silence_duration >= min_silence_samples {
                // Record the stop timestamp (end of silence)
                stops.push(current_sample_time / samples_per_second);

                // Reset silence duration to avoid multiple detections
                current_silence_duration = 0;
            }
        } else {
            // Reset silence duration if sound is detected
            current_silence_duration = 0;
        }

        current_sample_time += 1.0;
    }

    Ok(stops)
}

#[derive(Serialize, Default, Debug)]
struct Stop {
    sentence: String,
    audio_stop: f64,
}

#[derive(Serialize, Default, Debug)]
struct Res {
    audio_path: String,
    text_path: String,
    stops: Vec<Stop>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let stdin = std::io::stdin();
    let file_path = &stdin.lock().lines().next().unwrap().unwrap();
    let text_path = &stdin.lock().lines().next().unwrap().unwrap();

    let texts = std::fs::read_to_string(&text_path)?
        .split(".")
        .map(|str| str.trim().to_string())
        .collect::<Vec<String>>();

    let mut res: Res = Res::default();
    res.audio_path = file_path.clone();
    res.text_path = text_path.clone();

    // Parameters:
    // - Silence threshold (adjust based on your audio characteristics)
    // - Minimum silence duration (1.5 seconds)
    let stops = detect_stops(
        file_path, 1,   // Adjust this value based on your audio's characteristics
        0.5, // 1.5 seconds of silence
    )?;

    res.stops = texts
        .into_iter()
        .zip(stops.into_iter())
        .map(|(str, num)| Stop {
            sentence: str,
            audio_stop: num,
        })
        .collect();

    println!("{}", serde_json::to_string(&res)?);

    Ok(())
}
