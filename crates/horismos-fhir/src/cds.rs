//! CDS Hooks-shaped request and recommendation response values.
//!
//! The host owns hook lifecycle and UI. These types contain no network or
//! model behavior. Spec: `docs/framework/14-architecture.md` · Owner: PRP-008.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::AuthorizedFhirCommit;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CdsHookRequest {
    pub hook: String,
    pub hook_instance: String,
    pub context: Value,
    pub prefetch: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CdsHooksResponse {
    pub cards: Vec<CdsCard>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CdsCard {
    pub summary: String,
    pub detail: String,
    pub indicator: String,
    pub source: CdsSource,
    pub suggestions: Vec<CdsSuggestion>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CdsSource {
    pub label: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CdsSuggestion {
    pub label: String,
    pub uuid: String,
    pub actions: Vec<CdsAction>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CdsAction {
    #[serde(rename = "type")]
    pub action_type: String,
    pub description: String,
    pub resource: Value,
}

impl CdsCard {
    pub fn recommendation(summary: impl Into<String>, proposal: &AuthorizedFhirCommit) -> Self {
        let summary = summary.into();
        let proof_ref = proposal.evidence().proof_ref();
        let actions = proposal
            .effects()
            .effects
            .iter()
            .flat_map(|effect| {
                let resource = effect.payload["resource"].clone();
                let mut provenance = effect.payload["provenance"].clone();
                provenance["target"][0]["reference"] = Value::String(effect.target.clone());
                [
                    CdsAction {
                        action_type: "create".into(),
                        description: format!("Create the labeled proposed {}", effect.target),
                        resource,
                    },
                    CdsAction {
                        action_type: "create".into(),
                        description: format!("Create proof Provenance for {}", effect.target),
                        resource: provenance,
                    },
                ]
            })
            .collect();
        Self {
            detail: format!("Horismos symbolic recommendation; proof: {proof_ref}"),
            indicator: "info".into(),
            source: CdsSource {
                label: "Horismos".into(),
            },
            suggestions: vec![CdsSuggestion {
                label: summary.clone(),
                uuid: proof_ref.into(),
                actions,
            }],
            summary,
        }
    }
}
