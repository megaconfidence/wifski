use actix_multipart::Multipart;
use actix_web::{error, web, App, Error, HttpResponse, HttpServer, Responder};
use futures_util::stream::StreamExt;
use sanitize_filename::sanitize;
use serde::Deserialize;
use serde_json::json;
use std::io::Write;
use std::process::Command;
use tempfile::NamedTempFile;
use tokio::fs;
use uuid::Uuid;

#[derive(Deserialize, Debug)]
enum ResizeOption {
    #[serde(rename = "100")]
    _100,
    #[serde(rename = "75")]
    _75,
    #[serde(rename = "50")]
    _50,
    #[serde(rename = "25")]
    _25,
}

impl Default for ResizeOption {
    fn default() -> Self {
        ResizeOption::_75
    }
}

impl ResizeOption {
    fn to_ffmpeg_scale(&self) -> &str {
        match self {
            ResizeOption::_100 => "scale=iw:ih",
            ResizeOption::_75 => "scale=iw*0.75:ih*0.75",
            ResizeOption::_50 => "scale=iw*0.5:ih*0.5",
            ResizeOption::_25 => "scale=iw*0.25:ih*0.25",
        }
    }
}

#[derive(Deserialize, Debug)]
enum LoopOption {
    #[serde(rename = "forever")]
    Forever,
    #[serde(rename = "bounce")]
    Bounce,
    #[serde(untagged)]
    Count(i16),
}

impl Default for LoopOption {
    fn default() -> Self {
        LoopOption::Forever
    }
}

#[derive(Deserialize, Debug, Default)]
pub struct ConversionOptions {
    #[serde(default)]
    resize: ResizeOption,
    #[serde(default = "default_speed")]
    speed: f32,
    #[serde(default = "default_fps")]
    fps: u8,
    #[serde(default = "default_quality")]
    quality: u8,
    #[serde(default)]
    #[serde(alias = "loop")]
    loop_opt: LoopOption,
    start_time: Option<String>,
    end_time: Option<String>,
}

fn default_speed() -> f32 {
    1.0
}
fn default_fps() -> u8 {
    8
}
fn default_quality() -> u8 {
    75
}

async fn status() -> impl Responder {
    HttpResponse::Ok().json(json!({ "status": "ok" }))
}

async fn convert_to_gif(mut payload: Multipart) -> Result<HttpResponse, Error> {
    println!("\n[LOG] Received new conversion request.");
    let mut video_file: Option<NamedTempFile> = None;
    let mut options = ConversionOptions::default();

    while let Some(item) = payload.next().await {
        let mut field = item?;
        let content_disposition = field.content_disposition();
        let field_name = content_disposition.get_name().unwrap_or("").to_string();

        if field_name == "video" {
            let mut temp_file = NamedTempFile::new().map_err(error::ErrorInternalServerError)?;
            while let Some(chunk) = field.next().await {
                let data = chunk?;
                temp_file =
                    web::block(move || temp_file.write_all(&data).map(|_| temp_file)).await??;
            }
            video_file = Some(temp_file);
        } else {
            let mut bytes = Vec::new();
            while let Some(chunk) = field.next().await {
                bytes.extend_from_slice(&chunk?);
            }
            let value = String::from_utf8(bytes).map_err(error::ErrorBadRequest)?;

            match field_name.as_str() {
                "resize" => {
                    options.resize = match value.as_str() {
                        "100" => ResizeOption::_100,
                        "50" => ResizeOption::_50,
                        "25" => ResizeOption::_25,
                        _ => ResizeOption::_75,
                    }
                }
                "speed" => options.speed = value.parse().unwrap_or_else(|_| default_speed()),
                "fps" => options.fps = value.parse().unwrap_or_else(|_| default_fps()),
                "quality" => options.quality = value.parse().unwrap_or_else(|_| default_quality()),
                "loop" => {
                    options.loop_opt = match value.as_str() {
                        "forever" => LoopOption::Forever,
                        "bounce" => LoopOption::Bounce,
                        num_str => num_str
                            .parse::<i16>()
                            .map(LoopOption::Count)
                            .unwrap_or_default(),
                    };
                }
                "start_time" => options.start_time = Some(value),
                "end_time" => options.end_time = Some(value),
                _ => {}
            }
        }
    }

    println!("[LOG] Parsed form data. Options: {:?}", options);

    let input_file = video_file.ok_or_else(|| {
        eprintln!("[ERROR] Video file not provided in the request.");
        error::ErrorBadRequest("Video file not provided")
    })?;
    let input_path = input_file.path().to_str().unwrap();
    println!("[LOG] Video file saved temporarily to: {}", input_path);

    let unique_id = Uuid::new_v4();
    let palette_path = std::env::temp_dir().join(sanitize(format!("{}-palette.png", unique_id)));
    let output_path = std::env::temp_dir().join(sanitize(format!("{}.gif", unique_id)));

    let fps = options.fps.clamp(3, 10);
    let speed = options.speed.clamp(0.5, 5.0);
    let dither_option = if options.quality > 85 {
        "sierra2_4a"
    } else if options.quality > 60 {
        "bayer"
    } else {
        "none"
    };

    // --- Build Filter Chain ---
    let mut filters: Vec<String> = Vec::new();
    let mut trimmed = false;

    // 1. Build the trim filter part, quoting the time values to handle colons.
    let mut trim_parts: Vec<String> = Vec::new();
    if let Some(ref start) = options.start_time {
        if !start.is_empty() {
            trim_parts.push(format!("start='{}'", start));
        }
    }
    if let Some(ref end) = options.end_time {
        if !end.is_empty() {
            trim_parts.push(format!("end='{}'", end));
        }
    }

    if !trim_parts.is_empty() {
        trimmed = true;
        filters.push(format!("trim={}", trim_parts.join(":")));
    }

    // 2. FPS and Scaling
    filters.push(format!("fps={}", fps));
    filters.push(options.resize.to_ffmpeg_scale().to_string());

    // 3. Combine timestamp reset (if trimmed) with speed adjustment.
    // This must come AFTER other filters.
    if trimmed {
        filters.push(format!("setpts=(PTS-STARTPTS)/{}", speed));
    } else {
        filters.push(format!("setpts=PTS/{}", speed));
    }

    let base_filters = filters.join(",");

    // --- FFMPEG Pass 1: Generate Palette ---
    let mut palette_cmd = Command::new("ffmpeg");
    palette_cmd.arg("-i").arg(input_path);

    if matches!(options.loop_opt, LoopOption::Bounce) {
        // Corrected logic for bounce: filter, split, reverse one copy, then concat.
        let filter_complex = format!(
            "[0:v]{},split[a][b];[b]reverse[r];[a][r]concat=n=2:v=1:a=0,palettegen=stats_mode=full",
            base_filters
        );
        palette_cmd.arg("-filter_complex").arg(filter_complex);
    } else {
        let filter_vf = format!("{},palettegen=stats_mode=full", base_filters);
        palette_cmd.arg("-vf").arg(filter_vf);
    }

    palette_cmd.arg("-y").arg(&palette_path);

    println!("[LOG] Generating palette with command: {:?}", palette_cmd);
    let palette_output = palette_cmd.output().map_err(|e| {
        eprintln!("[ERROR] Failed to start FFMPEG palette generation: {}", e);
        error::ErrorInternalServerError(e)
    })?;

    if !palette_output.status.success() {
        let stderr = String::from_utf8_lossy(&palette_output.stderr);
        eprintln!("[ERROR] FFMPEG palette generation failed: {}", stderr);
        return Err(error::ErrorInternalServerError(
            "Failed to generate palette",
        ));
    }
    println!(
        "[LOG] Palette generated successfully at: {}",
        palette_path.display()
    );

    // --- FFMPEG Pass 2: Generate GIF using Palette ---
    let mut gif_cmd = Command::new("ffmpeg");
    gif_cmd
        .arg("-i")
        .arg(input_path)
        .arg("-i")
        .arg(&palette_path);

    if matches!(options.loop_opt, LoopOption::Bounce) {
        // Corrected logic for bounce in the second pass as well.
        let bounce_filter = format!(
            "[0:v]{},split[a][b];[b]reverse[r];[a][r]concat=n=2:v=1:a=0[v];[v][1:v]paletteuse=dither={}",
            base_filters, dither_option
        );
        gif_cmd.arg("-filter_complex").arg(bounce_filter);
    } else {
        let filter_complex = format!(
            "[0:v]{}[v];[v][1:v]paletteuse=dither={}",
            base_filters, dither_option
        );
        gif_cmd.arg("-filter_complex").arg(filter_complex);
    }

    match options.loop_opt {
        LoopOption::Count(n) => {
            gif_cmd.args(["-loop", &n.to_string()]);
        }
        LoopOption::Forever => {
            gif_cmd.args(["-loop", "0"]);
        }
        _ => {}
    }

    gif_cmd.arg("-y").arg(&output_path);

    println!(
        "[LOG] Executing FFMPEG GIF generation command: {:?}",
        gif_cmd
    );
    let gif_output = gif_cmd.output().map_err(|e| {
        eprintln!("[ERROR] Failed to start FFMPEG process: {}", e);
        error::ErrorInternalServerError(e)
    })?;

    if !gif_output.status.success() {
        let stderr = String::from_utf8_lossy(&gif_output.stderr);
        eprintln!("[ERROR] FFMPEG conversion failed: {}", stderr);
        fs::remove_file(&palette_path).await.ok(); // Clean up palette
        return Err(error::ErrorInternalServerError("Failed to convert video"));
    }

    println!("[LOG] FFMPEG conversion successful.");
    println!(
        "[LOG] Reading GIF data from {} and preparing response.",
        output_path.display()
    );

    let gif_data = fs::read(&output_path)
        .await
        .map_err(error::ErrorInternalServerError)?;

    // --- Cleanup ---
    println!("[LOG] Cleaning up temporary files.");
    fs::remove_file(&output_path)
        .await
        .map_err(error::ErrorInternalServerError)?;
    fs::remove_file(&palette_path)
        .await
        .map_err(error::ErrorInternalServerError)?;

    println!("[LOG] Sending GIF response.");
    Ok(HttpResponse::Ok().content_type("image/gif").body(gif_data))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Determine the number of worker threads to use, defaulting to 2 if unavailable.
    let workers = std::thread::available_parallelism().map_or(2, |n| n.get());

    println!(
        "Wifski-Container server starting on http://0.0.0.0:8080 with {} worker(s)",
        workers
    );

    HttpServer::new(|| {
        App::new()
            .route("/convert", web::post().to(convert_to_gif))
            .route("/status", web::get().to(status))
    })
    .workers(workers) // Set the number of worker threads.
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
