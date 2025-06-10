# OpenAPI to Node.js Module Generator

This tool is a Rust-based CLI that parses an OpenAPI 3.0 YAML file and generates a structured Node.js module exposing each API path as a callable function.

## Features

- Input: OpenAPI 3.0 YAML file
- Output: Clean Node.js module in `./output/apiClient/`
- Supports GET, POST, PUT, DELETE, etc.
- Functions are auto-generated and structured by path
- Uses `node-fetch@2` for HTTP requests
- Adds comments with OpenAPI summaries and request body schemas (if available)

## Example

Given this OpenAPI path:

```yaml
/users/settings:
  get:
    summary: Get user settings
  post:
    summary: Update user settings
    requestBody:
      content:
        application/json:
          schema:
            $ref: '#/components/schemas/SettingsInput'
```

It generates:

```js
// output/apiClient/user/settings/index.js

/// Get user settings
const fetch = require('node-fetch');

async function main() {
    const response = await fetch('https://api.example.com/users/settings', { method: 'GET' });
    return await response.json();
}

module.exports = main;

/// Update user settings
/// body:
///   "settingName": "string"
///   "enabled": "boolean"
const fetch = require('node-fetch');

async function post(body) {
    const response = await fetch('https://api.example.com/users/settings', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(body)
    });
    return await response.json();
}

module.exports.post = post;
```

## Installation

1. Clone the repo
2. Build the tool:

```bash
cargo build --release
```

3. Run it:

```bash
./target/release/openapi2node openapi.yaml
```

## Requirements

- Rust
- Valid OpenAPI 3.0 YAML file
- `node-fetch@2` in your Node.js project

## Output Structure

```
output/
└── apiClient/
    ├── index.js
    └── user/
        └── settings/
            └── index.js
```

## License

MIT
