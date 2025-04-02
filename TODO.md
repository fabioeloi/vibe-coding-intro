# TODO List: AI-Powered Personal Web History Knowledge Graph

This document outlines the implementation tasks required to build the AI-Powered Personal Web History Knowledge Graph application as defined in the PRD and technical specification.

## Milestone 1: Project Setup & File Parsing

- [ ] Create Tauri project structure with Svelte frontend
- [ ] Set up SQLite database with initial schema
- [ ] Implement SQLite extractor for Safari history.db files
  - [ ] Parse visit data (URL, title, timestamps, visit count)
  - [ ] Normalize data from multiple history files
  - [ ] Handle error cases and validation
- [ ] Design and implement data models for:
  - [ ] Visit
  - [ ] URL
  - [ ] Metadata
- [ ] Create file upload interface for history.db files
- [ ] Implement source tracking for multiple devices

## Milestone 2: Enrichment Pipeline

- [ ] Set up OpenAI-compatible API client
  - [ ] Configure for both cloud and local endpoints
  - [ ] Implement environment-based configuration
- [ ] Develop content fetching module for URL data
  - [ ] Handle various error cases and timeouts
  - [ ] Implement HTML content extraction
- [ ] Create AI enrichment pipeline:
  - [ ] Summarization functionality
  - [ ] Keyword extraction
  - [ ] Embedding generation
- [ ] Set up local Qdrant vector store
  - [ ] Create "url_embeddings" collection
  - [ ] Define vector schema with payload structure
- [ ] Build batch processing system for URL enrichment
  - [ ] Skip already processed URLs
  - [ ] Cache failures for retry
  - [ ] Implement async batch operations

## Milestone 3: Search & UI Implementation

- [ ] Design and implement search interface
  - [ ] Keyword-based search
  - [ ] Semantic search using embeddings
  - [ ] Combined search with scoring mechanism
- [ ] Create filtering system:
  - [ ] Time-based filters
  - [ ] Domain filters
  - [ ] Topic/tag filters
- [ ] Implement search results view with:
  - [ ] Link preview capabilities
  - [ ] Quick summaries
  - [ ] Metadata display

## Milestone 4: Dashboard & Timeline Views

- [ ] Implement timeline view
  - [ ] Chronological display of browsing history
  - [ ] Annotation capabilities
  - [ ] Color-coding by topic/domain
- [ ] Build time series dashboard
  - [ ] Daily/weekly visit trends
  - [ ] Topic clusters over time
  - [ ] Domain distribution charts
- [ ] Create thematic clustering visualization
  - [ ] NLP-based topic modeling
  - [ ] Interactive topic exploration
  - [ ] Cluster visualization

## Milestone 5: Offline Mode & Local AI Support

- [ ] Implement fully offline processing option
  - [ ] Local embedding models
  - [ ] Local summarization
- [ ] Create desktop agent for scheduled syncing (macOS service)
  - [ ] Background process for Safari DB monitoring
  - [ ] Incremental update system
- [ ] Add privacy controls and transparency features
  - [ ] Data usage explanations
  - [ ] Processing options configuration
  - [ ] Data retention settings

## Milestone 6: Packaging and User Testing

- [ ] Optimize app performance
  - [ ] Ensure < 200MB app size
  - [ ] < 1GB storage for 50k URLs
- [ ] Package application for distribution
  - [ ] macOS build process
  - [ ] Code signing and notarization
- [ ] Create user documentation
  - [ ] Setup guide
  - [ ] Usage tutorials
  - [ ] FAQ section
- [ ] Conduct user testing
  - [ ] Measure time to first insight
  - [ ] Track URL enrichment success rate
  - [ ] Gather user satisfaction metrics for search features

## Future Enhancements (Post-MVP)

- [ ] Implement bookmarking and annotation system
- [ ] Add integration with read-later services (Pocket/Instapaper)
- [ ] Develop mobile-friendly dashboard
- [ ] Create browser extension for real-time data collection

---

**Note**: This TODO list is based on the Product Requirements Document (PRD-history-graph.md) and technical specification (spec-001-history-graph.md). Tasks should be addressed in milestone order, with each milestone completed before moving to the next.
