# RoboAMO

Intelligent personnel assignment optimization for Naval Aviation Maintenance. Upload squadron data and receive optimal team assignments in seconds.

## Features

- **Smart Analysis** - Identifies qualification gaps and manning priorities
- **Rapid Processing** - Results in under a second
- **100% Private** - All processing occurs locally in your browser
- **Multi-Platform** - Runs on web, desktop, and mobile

## Quick Start

1. **Install dependencies:**
   ```bash
   npm install
   ```

2. **Start Tailwind CSS compiler:**
   ```bash
   npx tailwindcss -i ./tailwind.css -o ./assets/tailwind.css --watch
   ```

3. **Run the application:**
   ```bash
   dx serve
   ```

## How It Works

Upload 4 files through the guided workflow:
1. **Requirements** (.csv) - Team structures and qualification requirements
2. **Qual Definitions** (.csv) - Maps ASM codes to requirement names  
3. **ASM Report** (.xlsx) - Personnel roster with qualifications
4. **FLTMPS Roster** (.xlsx) - Projected rotation dates for TAR personnel

The system processes your data and generates optimal personnel assignments using advanced flow graph algorithms.

## Development

- `dx serve --platform desktop` - Run as desktop application
- `dx serve --platform mobile` - Run as mobile application  
- `cargo check` - Quick syntax/type checking