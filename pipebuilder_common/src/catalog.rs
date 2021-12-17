use crate::{invalid_catalog_name, json_schema_error, Result};
use jsonschema::JSONSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Serialize, Deserialize)]
pub struct CatalogSchemaMetadata {
    pub namespace: String,
    // schema id, note that catalog schema shared within namespace
    // regardless of project
    pub schema_id: String,
}

impl CatalogSchemaMetadata {
    pub fn get_path(&self) -> String {
        format!("{}/{}.json", self.namespace, self.schema_id)
    }
}

#[derive(Serialize, Deserialize)]
pub struct Catalog {
    pub schema: CatalogSchemaMetadata,
    // configuration filename
    pub name: String,
    // connfiguration context in yaml
    pub yml: String,
}

impl Catalog {
    pub fn accept<V>(&self, visitor: &mut V) -> Result<()>
    where
        V: VisitCatalog,
    {
        visitor.visit(self)
    }

    pub fn get_schema(&self) -> &CatalogSchemaMetadata {
        &self.schema
    }

    pub fn get_name(&self) -> &String {
        &self.name
    }
}

pub trait VisitCatalog {
    fn visit(&mut self, c: &Catalog) -> Result<()>;
}

pub trait ValidateCatalog: VisitCatalog {
    fn validate(&self) -> Result<()>;
}

pub struct CatalogSchemaValidator {
    pub schema: JSONSchema,
    pub instance: Option<serde_json::Value>,
}

impl CatalogSchemaValidator {
    pub fn new(schema: &str) -> Result<Self> {
        let schema = serde_json::from_str(schema)?;
        let schema = match JSONSchema::compile(&schema) {
            Ok(schema) => schema,
            Err(err) => {
                let operation = String::from("compile");
                let messages = vec![format!("{}", err)];
                return Err(json_schema_error(operation, messages));
            }
        };
        Ok(CatalogSchemaValidator {
            schema,
            instance: None,
        })
    }
}

impl VisitCatalog for CatalogSchemaValidator {
    fn visit(&mut self, c: &Catalog) -> Result<()> {
        // convert yml to json
        let json = yml_to_json(c.yml.as_str())?;
        let instance = serde_json::from_str(json.as_str())?;
        self.instance = Some(instance);
        Ok(())
    }
}

impl ValidateCatalog for CatalogSchemaValidator {
    fn validate(&self) -> Result<()> {
        let instance = self.instance.as_ref().expect("instance not defined");
        match self.schema.validate(instance) {
            Ok(_) => Ok(()),
            Err(errs) => {
                let messages: Vec<String> =
                    errs.into_iter().map(|err| format!("{}", err)).collect();
                Err(json_schema_error(String::from("validate"), messages))
            }
        }
    }
}

pub struct CatalogNameValidator {
    names: Vec<String>,
}

impl CatalogNameValidator {
    pub fn new() -> Self {
        CatalogNameValidator { names: vec![] }
    }
}

impl Default for CatalogNameValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl VisitCatalog for CatalogNameValidator {
    fn visit(&mut self, c: &Catalog) -> Result<()> {
        let name = c.get_name().to_owned();
        self.names.push(name);
        Ok(())
    }
}

impl ValidateCatalog for CatalogNameValidator {
    fn validate(&self) -> Result<()> {
        let len = self.names.len();
        let mut name_set: HashSet<String> = HashSet::new();
        // validate snake & lower case and non-empty
        // validate uniqueness
        for i in 0..len {
            let name = self.names.get(i).unwrap();
            if !is_non_empty(name) {
                return Err(invalid_catalog_name(
                    String::from("empty string"),
                    format!(".[{}], empty catalog name", i),
                ));
            }
            if !is_snake_lower_case(name) {
                return Err(invalid_catalog_name(
                    String::from("expect snake and lower case"),
                    format!(".[{}], catalog name not in snake or lower case", i),
                ));
            }
            if name_set.contains(name) {
                return Err(invalid_catalog_name(
                    String::from("duplicate string"),
                    format!(".[{}], catalog name duplicate", i),
                ));
            }
            name_set.insert(name.to_owned());
        }
        Ok(())
    }
}

fn is_non_empty(s: &str) -> bool {
    !s.is_empty()
}

fn is_snake_lower_case(s: &str) -> bool {
    is_snake_case(s, false)
}

fn is_snake_case(s: &str, uppercase: bool) -> bool {
    // no leading underscore
    let mut underscore = true;
    let mut initial_char = true;
    for c in s.chars() {
        if initial_char && !c.is_ascii() {
            return false;
        }
        initial_char = false;
        if c.is_numeric() {
            underscore = false;
            continue;
        }
        if c.is_ascii() && c.is_ascii_uppercase() == uppercase {
            underscore = false;
            continue;
        }
        if c == '_' {
            if underscore {
                // consecutive underscore
                return false;
            }
            underscore = true;
            continue;
        }
        return false;
    }
    true
}

fn yml_to_json(yml: &str) -> Result<String> {
    let value: serde_yaml::Value = serde_yaml::from_str(yml)?;
    let json = serde_json::to_string(&value)?;
    Ok(json)
}

#[cfg(test)]
mod tests {

    use crate::{
        Catalog, CatalogNameValidator, CatalogSchemaMetadata, CatalogSchemaValidator,
        ValidateCatalog,
    };

    const TEST_NAMESPACE: &str = "test";
    const TEST_CATALOG_SCHEMA_SCHEMA_ID: &str = "test_schema";
    const TEST_CATALOG_NAME: &str = "test_catalog";
    const TEST_CATALOG_YAML: &str = r#"
---
interval:
    Secs: 1000
ticks: 10
"#;
    const TEST_CATALOG_SCHEMA: &str = r##"
{
    "title": "test_catalog_schema",
    "type": "object",
    "definitions": {
        "interval_in_millis": {
            "type": "object",
            "properties": {
                "Millis": {
                    "type": "integer"
                }
            },
            "required": [ "Millis" ],
            "additionalProperties": false
        },
        "interval_in_secs": {
            "type": "object",
            "properties": {
                "Secs": {
                    "type": "integer"
                }
            },
            "required": [ "Secs" ],
            "additionalProperties": false
        },
        "interval_in_minutes": {
            "type": "object",
            "properties": {
                "Minutes": {
                    "type": "integer"
                }
            },
            "required": [ "Minutes" ],
            "additionalProperties": false
        },
        "interval_in_hours": {
            "type": "object",
            "properties": {
                "Hours": {
                    "type": "integer"
                }
            },
            "required": [ "Hours" ],
            "additionalProperties": false
        },
        "interval_in_days": {
            "type": "object",
            "properties": {
                "Days": {
                    "type": "integer"
                }
            },
            "required": [ "Days" ],
            "additionalProperties": false
        },
        "intervals": {
            "oneOf": [
                {
                    "$ref": "#/definitions/interval_in_millis"
                },
                {
                    "$ref": "#/definitions/interval_in_secs"
                },
                {
                    "$ref": "#/definitions/interval_in_minutes"
                },
                {
                    "$ref": "#/definitions/interval_in_hours"
                },
                {
                    "$ref": "#/definitions/interval_in_days"
                }
            ]
        }
    },
    "properties": {
        "interval": {
            "oneOf": [
                {
                    "$ref": "#/definitions/intervals"
                }
            ]
        },
        "delay": {
            "oneOf": [
                {
                    "$ref": "#/definitions/intervals"
                }
            ]
        },
        "ticks": {
            "type": "integer"
        }
    },
    "required": [ "interval", "ticks" ],
    "additionalProperties": false
}
"##;

    // sample validation for timer catalog
    #[test]
    fn test_valid_catalog() {
        let test_catalog_schema = CatalogSchemaMetadata {
            namespace: String::from(TEST_NAMESPACE),
            schema_id: String::from(TEST_CATALOG_SCHEMA_SCHEMA_ID),
        };
        let test_catalog = Catalog {
            schema: test_catalog_schema,
            name: String::from(TEST_CATALOG_NAME),
            yml: String::from(TEST_CATALOG_YAML),
        };
        let mut schema_validator = CatalogSchemaValidator::new(TEST_CATALOG_SCHEMA)
            .expect("failed to create schema validator");
        test_catalog
            .accept(&mut schema_validator)
            .expect("failed to visit catalog");
        schema_validator.validate().expect("invalid catalog schema");

        let mut name_validator = CatalogNameValidator::new();
        test_catalog
            .accept(&mut name_validator)
            .expect("failed to visit catalog");
        name_validator.validate().expect("invalidate catalog name");
    }
}
