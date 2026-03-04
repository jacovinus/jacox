# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased] - 2026-03-04

### Added
- **DuckDB Memory Inspection**: Added detailed breakdown of memory usage in the Dashboard with Monokai-styled progress bars.
- **Database Purge**: New CLI command `database purge` and API endpoint `DELETE /sessions` to clear all data and reset sequences.
- **Model Support**: Metadata example for `ministral-3:8b` model.
- **Project Historian Skill**: New standard and skill for maintaining project history and documentation.

### Fixed
- **SQL Error**: Corrected DuckDB memory query column name from `memory_usage` to `memory_usage_bytes`.
- **Frontend Types**: Fixed TypeScript implicitly 'any' errors in `Dashboard.tsx` and improved type safety for memory stats.

### Changed
- **Database Schema Access**: Made `SCHEMA` string public in `connection.rs` to allow clean-slate re-initialization during purge.
