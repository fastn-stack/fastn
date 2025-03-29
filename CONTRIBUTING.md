# Contributing to Fastn

Thank you for your interest in contributing to Fastn! This document provides guidelines and steps for contributing to the project.

## Table of Contents
- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Pull Request Process](#pull-request-process)
- [Versioning and Release Process](#versioning-and-release-process)

## Code of Conduct

Please read and follow our [Code of Conduct](CODE_OF_CONDUCT.md) to maintain a respectful and inclusive environment.

## Getting Started

### Prerequisites
- Rust toolchain (latest stable version)
- Git

### Setup
1. Clone the repo locally:
   ```bash
   git clone git@github.com:fastn-stack/fastn.git
   cd fastn
   ```
4. Create a new branch for your contribution:
   ```bash
   git checkout -b feature
   ```

## Pull Request Process

1. Update the CHANGELOG.md with details of your changes (see [Versioning section](#versioning-and-release-process))
2. Ensure your code builds, passes all tests, and follows our coding conventions
3. Submit your pull request against the `main` branch
4. Update the PR description with any relevant information
5. Wait for review and address any feedback

## Versioning and Release Process

We use [Semantic Versioning](https://semver.org/) for our releases.

### Updating Version Numbers

When making changes that require a version update:

1. Update the version in the Cargo.toml file:
   ```toml
   # In fastn/Cargo.toml
   [package]
   name = "fastn"
   version = "X.Y.Z"  # Increment the appropriate part of the version
   ```

2. Update the CHANGELOG.md file with your changes:
   ```markdown
   ## Unreleased

   ### fastn: X.Y.Z

   - Brief description of your change
   - Any breaking changes should be clearly marked
   ```

3. Commit these changes with relevant message


Thank you for contributing to Fastn!
