# Wifski

Create high-quality GIFs from your videos on the web.

[![Deploy to Cloudflare](https://deploy.workers.cloudflare.com/button)](https://deploy.workers.cloudflare.com/?url=https://github.com/megaconfidence/wifski)

Wifski is a powerful, web-based tool for converting videos into animated GIFs. It combines a sleek, responsive frontend with a robust, containerized Rust backend, all running on Cloudflare's edge network for high performance and scalability.

---

## Features

- **High-Quality GIF Conversion**: Utilizes a two-pass FFMPEG process to generate an optimized color palette for each GIF, resulting in superior image quality.
- **Extensive Customization**:
  - **Video Trimming**: An intuitive slider UI to select the exact start and end times for your GIF.
  - **Resize**: Scale the output GIF to `100%`, `75%`, `50%`, or `25%` of the original video size.
  - **Speed Control**: Adjust playback speed from `1x` to `5x`.
  - **FPS Adjustment**: Control the frames per second of the output GIF.
  - **Looping Options**: Create GIFs that loop forever or use a "bounce" loop (forward then reverse).
- **Modern Frontend**: A responsive, mobile-friendly interface built with Tailwind CSS.
- **Edge-Powered**: Deployed on Cloudflare's network, using Workers for routing and Containers for processing, ensuring low-latency access for users globally.

---

## Architecture

Wifski is composed of three main components that work together:

1.  **Frontend Application**: A static single-page application built with HTML, Tailwind CSS, and JavaScript. This is the user interface where videos are uploaded and conversion options are configured.

2.  **Cloudflare Worker (`worker/index.js`)**: This is the entry point for all traffic. The Worker serves the static frontend and acts as a router for API calls. It uses the `@cloudflare/containers` package to manage and forward requests to the backend container.

3.  **Backend Container (`Dockerfile`, `src/main.rs`)**: A multi-threaded Rust Actix Web server running inside a Docker container. This backend is responsible for the heavy lifting: it receives the video and options from the Worker, uses FFMPEG to perform the conversion, and streams the resulting GIF back.

---

## Local Development

You can run the entire application stack locally for development and testing.

### Prerequisites

- [Node.js](https://nodejs.org/) and `npm`.
- [Wrangler CLI](https://developers.cloudflare.com/workers/wrangler/install-and-update/): `npm install -g wrangler`
- [Docker Desktop](https://www.docker.com/products/docker-desktop/): Must be running on your machine.
- [Rust Toolchain](https://www.rust-lang.org/tools/install): Install via `rustup`.

### Running the Application

To run the full application, you need to run the backend container and the Cloudflare Worker simultaneously.

**Step 1: Run the Backend Container**

First, build and run the Docker container which houses the Rust API.

```bash
# 1. Build the Docker image
docker build -t wifski-container .

# 2. Run the container and map port 8080
docker run -p 8080:8080 wifski-container
```

The Rust server will now be running and accessible on `http://localhost:8080`.

**Step 2: Run the Cloudflare Worker**

The provided Worker script is configured to proxy requests to your local container when running in development mode.

In a **new terminal window**, start the Wrangler development server:

```bash
npx wrangler dev
```

You can now access the Wifski frontend application at `http://localhost:8787`. All API requests from the frontend will be automatically proxied by Wrangler to your running Docker container.

---

## Deployment to Cloudflare

Deploying Wifski to your Cloudflare account is straightforward with Wrangler.

1.  **Login to Wrangler:**

    ```bash
    npx wrangler login
    ```

2.  **Deploy the application:**
    ```bash
    npx wrangler deploy
    ```

When you run `wrangler deploy`, Wrangler will:

- Build your container image using Docker.
- Push the image to your private Cloudflare Container Registry.
- Deploy your Worker script.
- Create a Container binding, allowing the Worker to send requests to your container instances.

Your application will be live at the URL provided in the deployment output.

---
