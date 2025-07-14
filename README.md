# Blog Engine

A high-performance blog engine built with Rust and Axum, designed for [corybuecker.com](https://corybuecker.com). This static blog engine renders Markdown content with a modern web stack including Tailwind and TypeScript.

## Architecture

- **Backend**: Rust with Axum web framework
- **Frontend**: TailwindCSS 4.x + TypeScript
- **Content**: Markdown files with frontmatter
- **Templates**: Tera templating engine
- **Observability**: OpenTelemetry with Jaeger and Prometheus support
- **Deployment**: Docker with multi-stage builds

## Prerequisites

- Rust 1.70+ (uses 2024 edition)
- Node.js 18+
- Docker (optional, for containerized deployment)

## Quick Start

### Development Setup

1. **Clone and build the backend**:
   ```bash
   cargo build
   ```

2. **Install frontend dependencies**:
   ```bash
   cd assets
   npm install
   ```

3. **Build frontend assets**:
   ```bash
   # CSS (TailwindCSS)
   npm run css

   # JavaScript (TypeScript with esbuild)
   npm run js
   ```

4. **Run the development server**:
   ```bash
   cargo run
   ```

The blog will be available at `http://localhost:8000`.

### Development with Auto-reload

For frontend development with automatic rebuilding:

```bash
cd assets

# Watch CSS changes
npm run css:watch

# Watch TypeScript changes (in another terminal)
npm run js:watch
```

### Content Management

Blog posts are stored as Markdown files in the `content/` directory with numeric prefixes:

```
content/
├── 0000-an-introduction.md
├── 0001-automating-cloud-run-deploy.md
└── ...
```

Each post should include frontmatter with metadata (title, date, etc.).

## Development Services

The `dev/docker-compose.yaml` provides observability services:

```bash
cd dev
docker-compose up -d
```

This starts:
- **Prometheus** (metrics): http://localhost:9090
- **Jaeger** (tracing): http://localhost:16686
- **Grafana** (dashboards): http://localhost:3000

## Production Deployment

### Docker Build

```bash
docker build -t blog .
```

The multi-stage Dockerfile:
1. Builds the Rust backend
2. Compiles frontend assets with Node.js
3. Creates a minimal Debian runtime image

### Running the Container

```bash
docker run -p 8000:8000 blog
```

### Kubernetes Deployment

Kubernetes manifests are available in the `k8s/` directory for production deployment.

## Project Structure

```
├── src/                  # Rust source code
│   ├── main.rs          # Application entry point
│   ├── pages/           # Page handlers
│   └── utilities/       # Utility functions
├── assets/              # Frontend source
│   ├── css/            # TailwindCSS styles
│   ├── js/             # TypeScript code
│   └── package.json    # Node.js dependencies
├── content/            # Markdown blog posts
├── templates/          # Tera HTML templates
├── static/            # Static assets
├── k8s/               # Kubernetes manifests
├── dev/               # Development services
└── Dockerfile         # Multi-stage Docker build
```

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.

## Notes

This README was written by AI.
