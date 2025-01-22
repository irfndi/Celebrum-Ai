# Development Guide for Telegram Bot Project

This document provides a guide for developers working on the Telegram bot project. It covers setup, coding conventions, testing, and deployment. We are using the [python-telegram-bot](https://github.com/python-telegram-bot/python-telegram-bot) library for this project.

## Project Setup

1. **Install Python**
   - Ensure you have Python 3.10 or newer installed on your development machine
   - Download from [python.org](https://python.org) if needed

2. **Install `uv` Package Manager**
   ```bash
   pip install uv
   ```

3. **Create Virtual Environment**
   ```bash
   # Create new environment
   uv venv .venv

   # Activate environment
   # On Linux/macOS:
   source .venv/bin/activate
   # On Windows:
   .venv\Scripts\activate
   ```

4. **Install Dependencies**
   ```bash
   # Install from requirements.txt
   uv pip install -r requirements.txt
   
   # Or if using pyproject.toml:
   uv pip install -e .
   ```

5. **Install Development Tools**
   ```bash
   # Install ruff for linting/formatting
   uv pip install ruff

   # Install pylyzer for static type checking
   uv pip install pylyzer
   ```

6. **Code Analysis**
   ```bash
   # Run pylyzer for type checking
   pylyzer src/**/*.py

   # Run ruff for linting
   ruff check .

   # Format code using ruff
   ruff format                          # Format all files in the current directory (and any subdirectories).
   ruff format path/to/code/            # Format all files in `/path/to/code` (and any subdirectories).
   ruff format path/to/code/*.py        # Format all `.py` files in `/path/to/code`.
   ruff format path/to/code/to/file.py  # Format `file.py`.
   ruff format @arguments.txt           # Format using an input file, treating its contents as newline-delimited command-line arguments.
   ```