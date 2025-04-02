# Project Structure: AI-Powered Personal Web History Knowledge Graph

This document outlines the planned project structure for the web history knowledge graph application, following clean code principles and the specifications in our PRD and technical documentation.

## Overview

The application follows the Tauri architecture with a Svelte frontend:

```
my-new-repo/
├── src/                  # Tauri backend (Rust)
│   ├── main.rs           # Entry point for Tauri
│   ├── db/               # Database modules
│   ├── extractor/        # Safari history extraction
│   ├── enrichment/       # AI enrichment pipeline
│   └── api/              # API endpoints for frontend
│
├── src-svelte/           # Svelte frontend
│   ├── lib/              # Shared components and utilities
│   ├── routes/           # Page routes
│   │   ├── search/       # Search interface
│   │   ├── timeline/     # Timeline view
│   │   └── dashboard/    # Analytics dashboard
│   └── main.js           # Frontend entry point
│
├── src-tauri/            # Tauri configuration
│   ├── tauri.conf.json   # Tauri config
│   └── build.rs          # Build script
│
├── database/             # Database schemas and migrations
│   ├── schema.sql        # Initial schema
│   └── migrations/       # Migration scripts
│
└── vector-store/         # Vector store configuration
    └── schema.json       # Qdrant collection config
```

## Key Components

### Backend (Rust)

1. **Database Module**
   - SQLite connection management
   - Data models for Visit, URL, and Metadata
   - Query builders and ORM-like interfaces

2. **Extractor Module**
   - Safari history.db parser
   - Data normalization
   - Multiple file handling

3. **Enrichment Pipeline**
   - OpenAI-compatible client
   - Content fetching and processing
   - Batch processing controller

4. **API Module**
   - Endpoints for frontend communication
   - Data serialization/deserialization

### Frontend (Svelte)

1. **Shared Components**
   - UI components (cards, inputs, etc.)
   - Data visualization components
   - State management stores

2. **Search Interface**
   - Search input with filters
   - Results display with previews
   - Semantic search controls

3. **Timeline View**
   - Interactive timeline visualization
   - Grouping and filtering controls
   - Annotation interface

4. **Dashboard**
   - Time series charts
   - Topic cluster visualization
   - Domain distribution displays

## Development Approach

We will follow these principles:

1. **Modular Design**: Each component has a single responsibility
2. **Clean Interfaces**: Well-defined APIs between modules
3. **Progressive Implementation**: Start with core functionality, then add features
4. **Test Coverage**: Unit tests for critical components
5. **Documentation**: Comments and docs for all public interfaces

This structure will evolve as development progresses, but serves as an initial guide for organizing the codebase.
