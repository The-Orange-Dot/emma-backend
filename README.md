# emma: E-commerce Merchandise Matching Agent

## Overview

**emma** (all lowercase) is an AI agent designed to function as a store clerk for e-commerce platforms. This repository contains the backend service, which provides a conversational interface for organic product discovery. By implementing a **Retrieval-Augmented Generation (RAG)** pipeline, the system bridges the gap between raw e-commerce datasets and Large Language Models (LLMs).

## Mission

The primary objective of emma is to facilitate personalized product discovery through semantic retrieval. The system ensures generated responses are strictly grounded in a provided Shopify or CSV inventory to eliminate model hallucinations and provide factual product feedback.

## Core Functionality

- **Multi-Format Support:** Asynchronous processing of Shopify API exports and standard CSV datasets.
- **Vector Pipeline:** Automated text chunking and embedding generation for product metadata and descriptions.
- **Semantic Search:** Integration with vector databases to enable intent-based product matching.

## Technical Specifications

### Core Architecture

The system utilizes a RAG architecture optimized for low-latency matching. By leveraging **PGAI** extensions within a **PostgreSQL** environment, emma manages data chunking, vectorization, and retrieval within the database layer. This keeps inventory data and embeddings synchronized in real-time.

### Models

- **LLM:** Gemma 4 (`gemma4:e4b`) — A high-efficiency vision-capable model utilized for context-aware feedback and multimodal input processing.
- **Embeddings:** Nomic Embed Text — Used for transforming structured product metadata into high-dimensional vectors.

### Infrastructure & Storage

- **Language:** **Rust** — Selected for memory safety, high concurrency, and execution speed.
- **Database:** **PostgreSQL** — Chosen for its reliability, advanced querying, and scalability.
- **Vector Engine:** **PGAI** — A PostgreSQL extension that handles the end-to-end RAG pipeline (chunking and embedding) directly within the database.

---

## Dependencies

The backend is built on the **Actix** ecosystem with **SQLx** for type-safe database interactions.

| Crate              | Purpose                                                            |
| :----------------- | :----------------------------------------------------------------- |
| `actix-web`        | High-performance asynchronous web framework.                       |
| `sqlx`             | Type-safe SQL toolkit with PostgreSQL and Tokio support.           |
| `pgai`             | PostgreSQL extension for in-database vector operations and RAG.    |
| `reqwest`          | HTTP client for external API communication (LLM integration).      |
| `tokio`            | Asynchronous runtime for non-blocking I/O.                         |
| `serde`            | Framework for serializing and deserializing Rust data structures.  |
| `image` / `base64` | Processing and encoding image data for vision-model compatibility. |

---

## Developer Notes

### The Move to Rust

This project represents a strategic shift from Node.js to Rust. The primary drivers were the requirements for high-performance request throughput and strict memory safety. While the Rust compiler is notoriously demanding, its rigorous "yell-at-you" feedback loop ensures the backend is resource-safe and free from common race conditions.

### Isolated PostgreSQL with PGAI

To mitigate liability and maintain data integrity, emma does not operate directly on the client’s production database. Instead, the system ingests a synchronized copy of the store’s data into a standalone PostgreSQL instance.

- **In-Database RAG Pipeline:** Using the **PGAI** extension, the system handles data chunking and vectorization directly within the PostgreSQL environment. This keeps the vector embeddings and the product inventory synchronized within a single source of truth.
- **Efficient State Synchronization:** Each item is mapped by a unique ID. When an item is updated (currently supported via Shopify), emma re-chunks and re-embeds only the specific product metadata, replacing the outdated vector data without requiring a full database re-ingestion.

### Model Selection (Gemma 4)

The system utilizes `gemma4:e4b` to maintain performance without high compute overhead. This model provides an ideal balance of small-footprint efficiency and the multimodal capabilities required to interpret complex e-commerce datasets and user-provided images.

### Privacy & Stateless Image Processing

Privacy is a priority for this implementation. When end-users upload an image with their prompt (e.g., for visual product matching), the image data is processed in-memory for inference and is **not persisted** to any long-term storage or logs.
