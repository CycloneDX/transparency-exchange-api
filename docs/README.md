# Transparency Exchange API Documentation

This documentation site is built with [Just the Docs](https://just-the-docs.com), a Jekyll theme for documentation.

## Local Development

### Prerequisites

- Ruby (version 2.5 or higher)
- Bundler

### Setup

1. Install dependencies:

   ```bash
   bundle install
   ```

2. Run the development server:

   ```bash
   bundle exec jekyll serve
   ```

3. Open your browser to `http://localhost:4000`

### Building

To build the site for production:

```bash
bundle exec jekyll build
```

The site will be generated in the `_site` directory.

## Deployment

This site is configured for deployment via GitHub Pages. Commits to the `main` branch will automatically deploy the site.

## Structure

- `_config.yml`: Jekyll configuration
- `index.md`: Home page
- `architecture.md`: Architecture documentation
- `evidence.md`: Evidence collection guide
