# Wifski-Container - High-Quality Video to GIF Converter

Wifski-Container is a high-performance, containerized Rust web service that converts video files into high-quality animated GIFs. Built with Actix Web and FFMPEG, it offers a range of customization options through a simple API endpoint.

This application is designed for ease of use and deployment, thanks to its Docker containerization.

## Table of Contents

- [Features](#features)
- [How It Works](#how-it-works)
- [Prerequisites](#prerequisites)
- [Running with Docker (Recommended)](#running-with-docker-recommended)
- [Running Locally (for Development)](#running-locally-for-development)
- [API Documentation](#api-documentation)
  - [Endpoint](#endpoint)
  - [Form Fields](#form-fields)
  - [cURL Examples](#curl-examples)
- [Performance Notes](#performance-notes)

---

## Features

- **High-Quality GIF Conversion**: Utilizes a two-pass FFMPEG process to generate an optimized color palette for each GIF, resulting in superior image quality.
- **Extensive Customization**:
  - **Trimming**: Specify start and end times to convert only a segment of the video.
  - **Resize**: Scale the output GIF to a percentage of the original video size.
  - **Speed**: Adjust playback speed from 0.5x to 5.0x.
  - **FPS**: Control the frames per second.
  - **Looping**: Set a finite loop count, loop forever, or use a "bounce" loop (forward then reverse).
- **Containerized**: Fully containerized with Docker for easy, platform-independent deployment.
- **Robust Logging**: Provides clear console logs for monitoring the status of each conversion.

---

## How It Works

When a video is uploaded, Wifski-Container performs a two-pass conversion process:

1.  **Palette Generation**: FFMPEG first scans the specified video segment to create a custom 256-color palette that best represents the colors in the source clip. This step is crucial for avoiding color banding and artifacts in the final GIF.
2.  **GIF Creation**: FFMPEG then uses this custom palette to encode the video segment into a GIF, applying the user-specified resizing, speed, FPS, and dithering options.

This method is more computationally intensive than a single-pass conversion but produces significantly better-looking GIFs.

---

## Prerequisites

To run this application, you only need **Docker** installed on your system.

- [Install Docker](https://docs.docker.com/get-docker/)

If you wish to run it locally for development purposes, you will need:

- **Rust**: Install via `rustup` from the [official Rust website](https://www.rust-lang.org/tools/install).
- **FFMPEG**: Must be installed and available in your system's PATH. Download from the [official FFMPEG website](https://ffmpeg.org/download.html).

---

## Running with Docker (Recommended)

1.  **Build the Docker image:**
    From the project's root directory, execute:

    ```bash
    docker build -t wifski-container .
    ```

2.  **Run the Docker container:**
    This command starts the server and maps port 8080 on your host to port 8080 in the container.
    `bash
    docker run -p 127.0.0.1:8080:8080 wifski-container
    `
    The server is now running and accessible at `http://127.0.0.1:8080`.

---

## Running Locally (for Development)

1.  **Clone the repository and navigate into it.**

2.  **Build the project in release mode:**

    ```bash
    cargo build --release
    ```

3.  **Run the server:**
    `bash
    cargo run --release
    `
    The server will start on `127.0.0.1:8080`.

---

## API Documentation

### Endpoint

- `POST /convert`

The endpoint accepts `multipart/form-data` requests.

### Form Fields

| Field        | Type       | Description                                                                                        | Default     |
| :----------- | :--------- | :------------------------------------------------------------------------------------------------- | :---------- |
| **`video`**  | File       | **(Required)** The video file to convert.                                                          | -           |
| `start_time` | String     | The start time of the clip in seconds (e.g., '5').                                                 | Video Start |
| `end_time`   | String     | The end time of the clip in seconds (e.g., '10').                                                  | Video End   |
| `resize`     | String     | The resize percentage. Accepted values: `"100"`, `"75"`, `"50"`, `"25"`.                           | `"75"`      |
| `speed`      | Float      | The playback speed multiplier. Clamped between `0.5` and `5.0`.                                    | `1.0`       |
| `fps`        | Integer    | Frames per second for the output GIF. Clamped between `3` and `10`.                                | `8`         |
| `quality`    | Integer    | An abstract quality value (`0-100`) that influences the dithering algorithm used.                  | `75`        |
| `loop`       | String/Int | Looping option. `"forever"`, `"bounce"`, or an integer for a specific loop count (`0` is forever). | `"forever"` |

### cURL Examples

#### **1. Basic Conversion**

This will use all default options on the full video.

```bash
curl -X POST \
  -F "video=@/path/to/your/video.mp4" \
  [http://127.0.0.1:8080/convert](http://127.0.0.1:8080/convert) \
  -o output.gif
```

#### **2. Custom Conversion**

A 50% size GIF, running at 2x speed, with 10 FPS, that loops 5 times.

```bash
curl -X POST \
  -F "video=@/path/to/your/video.mp4" \
  -F "resize=50" \
  -F "speed=2.0" \
  -F "fps=10" \
  -F "loop=5" \
  [http://127.0.0.1:8080/convert](http://127.0.0.1:8080/convert) \
  -o custom_output.gif
```

#### **3. Bounce Loop**

A GIF that plays forwards and then backward, looping indefinitely.

```bash
curl -X POST \
  -F "video=@/path/to/your/video.mp4" \
  -F "loop=bounce" \
  [http://127.0.0.1:8080/convert](http://127.0.0.1:8080/convert) \
  -o bounce_output.gif
```

#### **4. Time-Trimmed Conversion**

Creates a GIF from the 5-second mark to the 10-second mark of the video.

```bash
curl -X POST \
  -F "video=@/path/to/your/video.mp4" \
  -F "start_time=5" \
  -F "end_time=10" \
  [http://127.0.0.1:8080/convert](http://127.0.0.1:8080/convert) \
  -o trimmed_output.gif
```

---

## Performance Notes

- The two-pass conversion is resource-intensive. Processing long, high-resolution videos can consume significant CPU time and memory.
- The service is designed to handle multiple requests concurrently, but performance will depend on the host machine's available CPU cores.
