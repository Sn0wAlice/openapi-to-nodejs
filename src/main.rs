use std::collections::BTreeMap;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;
use std::fs::create_dir_all;

use serde::Deserialize;
use serde_yaml::from_str;

#[derive(Debug, Deserialize)]
struct OpenAPI {
    paths: BTreeMap<String, BTreeMap<String, Operation>>,
    components: Option<Components>,
}

#[derive(Debug, Deserialize)]
struct Components {
    schemas: Option<BTreeMap<String, Schema>>,
}
#[derive(Debug, Deserialize)]
struct Operation {
    summary: Option<String>,
    requestBody: Option<RequestBody>,
}

#[derive(Debug, Deserialize)]
struct RequestBody {
    content: BTreeMap<String, MediaType>, // usually application/json
}

#[derive(Debug, Deserialize)]
struct MediaType {
    schema: Option<Schema>,
}

#[derive(Debug, Deserialize)]
struct Schema {
    #[serde(rename = "type")]
    typ: Option<String>,

    #[serde(rename = "$ref")]
    reference: Option<String>,

    properties: Option<BTreeMap<String, SchemaProperty>>,

    items: Option<Box<SchemaProperty>>,

    oneOf: Option<Vec<SchemaProperty>>,
}

#[derive(Debug, Deserialize)]
struct SchemaProperty {
    #[serde(rename = "type")]
    typ: Option<String>,

    #[serde(rename = "$ref")]
    reference: Option<String>,

    format: Option<String>,

    enum_values: Option<Vec<serde_yaml::Value>>,

    items: Option<Box<SchemaProperty>>,

    oneOf: Option<Vec<SchemaProperty>>,
}

fn sanitize(name: &str) -> String {
    name.replace("{", "_param_").replace("}", "")
}

fn generate_js_function(method: &str, summary: &Option<String>, request_body: &Option<RequestBody>, components:  Option<&Components>, full_path: &str) -> String {
    let comment = match summary {
        Some(text) => format!("/// {}", text),
        None => String::from("/// No description"),
    };

    let url = format!("https://api.alice-snow.ru{}", full_path);

    if method == "get" {
        format!(
            "{comment}\nconst fetch = require('node-fetch');\n\nasync function main() {{\n    const response = await fetch('{url}', {{ method: 'GET' }});\n    return await response.json();\n}}\n\nmodule.exports = main;\n",
            comment = comment,
            url = url
        )
    } else {

        let body_doc = match request_body {
            Some(rb) => {
                let mut doc = String::from("/// body:\n");

                if let Some(media) = rb.content.get("application/json") {
                    if let Some(schema) = &media.schema {
                        let effective_schema = if let Some(reference) = &schema.reference {
                            // extraire le nom depuis #/components/schemas/XXX
                            let key = reference.rsplit('/').next().unwrap();
                            components
                                .and_then(|c| c.schemas.as_ref())
                                .and_then(|m| m.get(key))
                        } else {
                            Some(schema)
                        };

                        if let Some(real_schema) = effective_schema {
                            if let Some(props) = &real_schema.properties {
                                for (k, v) in props {
                                    let mut type_str = v.typ.clone().unwrap_or_else(|| "unknown".into());

                                    if let Some(format) = &v.format {
                                        type_str = format!("{} ({})", type_str, format);
                                    } else if v.reference.is_some() {
                                        type_str = format!("ref -> {}", v.reference.as_ref().unwrap());
                                    } else if v.oneOf.is_some() {
                                        type_str = "oneOf".into();
                                    } else if v.items.is_some() {
                                        type_str = format!("array<{}>", v.items.as_ref().unwrap().typ.clone().unwrap_or("unknown".into()));
                                    }

                                    doc.push_str(&format!("///   \"{}\": \"{}\"\n", k, type_str));
                                }
                            }
                        }
                    }
                }

                doc
            }
            None => String::new(),
        };

        format!(
            "/// {summary}\nconst fetch = require('node-fetch');\n\n{body_doc}async function {method}(body) {{\n    const response = await fetch('{url}', {{\n        method: '{method_upper}',\n        headers: {{ 'Content-Type': 'application/json' }},\n        body: JSON.stringify(body)\n    }});\n    return await response.json();\n}}\n\nmodule.exports.{method} = {method};\n",
            summary = summary.as_deref().unwrap_or("No description"),
            body_doc = body_doc,
            method = method,
            method_upper = method.to_uppercase(),
            url = url
        )
    }
}

fn write_structure(openapi: OpenAPI, output_dir: &str, module_name: &str) {
    let root = Path::new(output_dir).join(module_name);
    create_dir_all(&root).unwrap();

    let mut index_exports = String::new();

    for (path, methods) in openapi.paths {
        let clean_path = path.trim_start_matches('/').replace('/', ".");
        let parts: Vec<&str> = clean_path.split('.').collect();

        let mut current_path = root.clone();
        for part in &parts {
            current_path = current_path.join(sanitize(part));
            create_dir_all(&current_path).unwrap();
        }

        for (method, op) in methods {
            let js_file_path = current_path.join("index.js");
            let function_code = generate_js_function(&method, &op.summary, &op.requestBody, openapi.components.as_ref(), &path);

            if method == "get" {
                fs::write(&js_file_path, function_code).unwrap();
            } else {
                let mut file = OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(&js_file_path)
                    .unwrap();

                writeln!(file, "{}", function_code).unwrap();
            }
        }

        // Update main index
        let mut accessor = String::new();
        for part in &parts {
            accessor.push_str(&format!("['{}']", sanitize(part)));
        }
        index_exports.push_str(&format!("module.exports{} = require('./{}');\n",
                                        accessor,
                                        parts.join("/")
        ));
    }

    fs::write(root.join("index.js"), index_exports).unwrap();
}

fn main() {
    let yaml_path = "docs.yaml";
    let module_name = "apiClient";
    let output_dir = "./output";

    let content = fs::read_to_string(yaml_path).expect("Cannot read openapi.yaml");
    let spec: OpenAPI = from_str(&content).expect("Invalid OpenAPI format");

    write_structure(spec, output_dir, module_name);
    println!("Node.js module generated in {}/{}", output_dir, module_name);
}