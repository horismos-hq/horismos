//! Bounded R4 PlanDefinition import and external-CQL result composition.
//!
//! Spec: `docs/framework/14-architecture.md` · Owner: PRP-008.

use std::collections::{BTreeMap, BTreeSet};

use horismos_host::{ArtifactIdentity, HostError, SourceVersionVector};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::normalize_etag;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExternalBooleanResult {
    pub name: String,
    pub value: bool,
    pub evaluator: ArtifactIdentity,
    pub source_versions: SourceVersionVector,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PlanAction {
    id: String,
    title: String,
    description: String,
    cql_booleans: Vec<String>,
}

impl PlanAction {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn description(&self) -> &str {
        &self.description
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ImportedPlanDefinition {
    canonical: String,
    version: String,
    status: String,
    title: String,
    actions: Vec<PlanAction>,
}

impl ImportedPlanDefinition {
    pub fn canonical(&self) -> &str {
        &self.canonical
    }

    pub fn version(&self) -> &str {
        &self.version
    }

    pub fn status(&self) -> &str {
        &self.status
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn from_r4_json(value: &Value) -> Result<Self, HostError> {
        let root = object(value, "PlanDefinition")?;
        reject_unknown(
            root,
            &[
                "resourceType",
                "id",
                "url",
                "version",
                "status",
                "title",
                "action",
            ],
            "PlanDefinition",
        )?;
        if string(root, "resourceType", "PlanDefinition")? != "PlanDefinition" {
            return Err(subset_error(
                "PlanDefinition.resourceType",
                "resourceType must be PlanDefinition",
            ));
        }
        let canonical = string(root, "url", "PlanDefinition")?.to_owned();
        let version = string(root, "version", "PlanDefinition")?.to_owned();
        let status = string(root, "status", "PlanDefinition")?.to_owned();
        if status != "active" && status != "draft" {
            return Err(subset_error(
                "PlanDefinition.status",
                "only active or draft status is supported",
            ));
        }
        let title = string(root, "title", "PlanDefinition")?.to_owned();
        let action_values = root
            .get("action")
            .and_then(Value::as_array)
            .ok_or_else(|| {
                subset_error("PlanDefinition.action", "a flat action array is required")
            })?;
        let mut actions = Vec::with_capacity(action_values.len());
        let mut action_ids = BTreeSet::new();
        for (index, value) in action_values.iter().enumerate() {
            let path = format!("PlanDefinition.action[{index}]");
            let action = object(value, &path)?;
            if action.contains_key("action") {
                return Err(subset_error(
                    &format!("{path}.action"),
                    "nested actions are unsupported",
                ));
            }
            reject_unknown(action, &["id", "title", "description", "condition"], &path)?;
            let id = string(action, "id", &path)?.to_owned();
            if !action_ids.insert(id.clone()) {
                return Err(subset_error(
                    &format!("{path}.id"),
                    "action ids must be unique",
                ));
            }
            let title = string(action, "title", &path)?.to_owned();
            let description = string(action, "description", &path)?.to_owned();
            let conditions = action
                .get("condition")
                .and_then(Value::as_array)
                .ok_or_else(|| {
                    subset_error(
                        &format!("{path}.condition"),
                        "applicability conditions are required",
                    )
                })?;
            if conditions.is_empty() {
                return Err(subset_error(
                    &format!("{path}.condition"),
                    "at least one applicability condition is required",
                ));
            }
            let mut cql_booleans = Vec::with_capacity(conditions.len());
            for (condition_index, value) in conditions.iter().enumerate() {
                let condition_path = format!("{path}.condition[{condition_index}]");
                let condition = object(value, &condition_path)?;
                reject_unknown(condition, &["kind", "expression"], &condition_path)?;
                if string(condition, "kind", &condition_path)? != "applicability" {
                    return Err(subset_error(
                        &format!("{condition_path}.kind"),
                        "only applicability conditions are supported",
                    ));
                }
                let expression_path = format!("{condition_path}.expression");
                let expression = condition
                    .get("expression")
                    .ok_or_else(|| subset_error(&expression_path, "expression is required"))?;
                let expression = object(expression, &expression_path)?;
                reject_unknown(expression, &["language", "expression"], &expression_path)?;
                if string(expression, "language", &expression_path)? != "text/cql-identifier" {
                    return Err(subset_error(
                        &format!("{expression_path}.language"),
                        "only text/cql-identifier is supported",
                    ));
                }
                cql_booleans.push(string(expression, "expression", &expression_path)?.to_owned());
            }
            actions.push(PlanAction {
                id,
                title,
                description,
                cql_booleans,
            });
        }
        Ok(Self {
            canonical,
            version,
            status,
            title,
            actions,
        })
    }

    pub fn execute(
        &self,
        results: &BTreeMap<String, ExternalBooleanResult>,
        expected_versions: &SourceVersionVector,
        expected_evaluator: &ArtifactIdentity,
    ) -> Result<Vec<PlanAction>, HostError> {
        let mut selected = Vec::new();
        for action in &self.actions {
            let mut applicable = true;
            for name in &action.cql_booleans {
                let result = results.get(name).ok_or_else(|| {
                    HostError::boundary(
                        format!(
                            "horis:externalResult requires {name}, but no typed boolean result was supplied"
                        ),
                        "horis:externalResult",
                        name,
                        Some("missing".into()),
                        "named PlanDefinition condition is unresolved",
                    )
                })?;
                if result.name != *name || result.evaluator != *expected_evaluator {
                    return Err(external_result_error(name, "identity mismatch", "invalid"));
                }
                if !versions_match(&result.source_versions, expected_versions) {
                    return Err(external_result_error(
                        name,
                        "source vector differs",
                        "stale",
                    ));
                }
                applicable &= result.value;
            }
            if applicable {
                selected.push(action.clone());
            }
        }
        Ok(selected)
    }
}

fn versions_match(left: &SourceVersionVector, right: &SourceVersionVector) -> bool {
    left.entries().len() == right.entries().len()
        && left.entries().iter().zip(right.entries()).all(|(a, b)| {
            a.resource == b.resource
                && a.version_id == b.version_id
                && a.etag.as_deref().map(normalize_etag) == b.etag.as_deref().map(normalize_etag)
        })
}

fn object<'a>(value: &'a Value, path: &str) -> Result<&'a Map<String, Value>, HostError> {
    value
        .as_object()
        .ok_or_else(|| subset_error(path, "a JSON object is required"))
}

fn string<'a>(
    object: &'a Map<String, Value>,
    field: &str,
    path: &str,
) -> Result<&'a str, HostError> {
    object
        .get(field)
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| subset_error(&format!("{path}.{field}"), "a non-empty string is required"))
}

fn reject_unknown(
    object: &Map<String, Value>,
    allowed: &[&str],
    path: &str,
) -> Result<(), HostError> {
    let allowed: BTreeSet<&str> = allowed.iter().copied().collect();
    if let Some(field) = object
        .keys()
        .find(|field| !allowed.contains(field.as_str()))
    {
        return Err(subset_error(
            &format!("{path}.{field}"),
            "field is outside the bounded PlanDefinition subset",
        ));
    }
    Ok(())
}

fn subset_error(subject: &str, detail: &str) -> HostError {
    HostError::boundary(
        format!("horis:supportedSubset rejected {subject}: {detail}"),
        "horis:supportedSubset",
        subject,
        Some("unsupported".into()),
        detail,
    )
}

fn external_result_error(subject: &str, detail: &str, state: &str) -> HostError {
    HostError::boundary(
        format!("horis:externalResult rejected {subject}: {detail}"),
        "horis:externalResult",
        subject,
        Some(state.into()),
        detail,
    )
}
