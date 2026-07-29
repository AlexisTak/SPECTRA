//! Orchestrateur de transforms.
//!
//! Gère l'exécution d'un graphe acyclique dirigé (DAG) de transforms avec :
//! - rate limiting par domaine,
//! - cache de réponses,
//! - annulation propre via `CancellationToken`.

use crate::cache::{CacheKey, ResponseCache};
use crate::transform::{BoxedTransform, Transform, TransformContext, TransformError, TransformInput, TransformOutput};
use governor::{clock::DefaultClock, state::keyed::DefaultKeyedStateStore, Quota, RateLimiter};
use std::collections::{HashMap, HashSet};
use std::num::NonZeroU32;
use std::sync::Arc;
use tokio_util::sync::CancellationToken;

/// Plan d'exécution : une liste de nœuds et leurs dépendances.
#[derive(Debug, Default)]
pub struct ExecutionPlan {
    nodes: Vec<PlanNode>,
    /// index dans `nodes` -> indices des prédécesseurs.
    deps: HashMap<usize, Vec<usize>>,
}

#[derive(Debug, Clone)]
struct PlanNode {
    transform_id: String,
    input: TransformInput,
}

/// Résultat d'un nœud du plan.
#[derive(Debug, Clone)]
pub struct NodeResult {
    pub transform_id: String,
    pub output: Result<TransformOutput, TransformError>,
    /// Temps d'exécution en millisecondes.
    pub elapsed_ms: u64,
    /// Vrai si le résultat venait du cache.
    pub from_cache: bool,
}

impl ExecutionPlan {
    /// Crée un plan vide.
    pub fn new() -> Self {
        Self::default()
    }

    /// Ajoute un nœud au plan. Retourne son index.
    pub fn add_node(
        &mut self,
        transform_id: String,
        input: TransformInput,
    ) -> usize {
        let idx = self.nodes.len();
        self.nodes.push(PlanNode { transform_id, input });
        self.deps.insert(idx, Vec::new());
        idx
    }

    /// Déclare que `node` dépend de `dependency` (doit finir avant).
    pub fn add_dependency(
        &mut self,
        node: usize,
        dependency: usize,
    ) {
        self.deps.entry(node).or_default().push(dependency);
    }

    /// Vérifie que le graphe est acyclique (simple détection de cycle DFS).
    pub fn validate(&self) -> Result<(), TransformError> {
        let n = self.nodes.len();
        let mut visited = HashSet::new();
        let mut stack = HashSet::new();

        fn dfs(
            plan: &ExecutionPlan,
            u: usize,
            visited: &mut HashSet<usize>,
            stack: &mut HashSet<usize>,
        ) -> Result<(), TransformError> {
            if stack.contains(&u) {
                return Err(TransformError::Internal(
                    "cycle détecté dans le plan d'exécution".to_string(),
                ));
            }
            if visited.contains(&u) {
                return Ok(());
            }
            visited.insert(u);
            stack.insert(u);
            for &v in plan.deps.get(&u).unwrap_or(&Vec::new()) {
                dfs(plan, v, visited, stack)?;
            }
            stack.remove(&u);
            Ok(())
        }

        for i in 0..n {
            if !visited.contains(&i) {
                dfs(self, i, &mut visited, &mut stack)?;
            }
        }
        Ok(())
    }

    /// Calcule l'ordre topologique (indices des nœuds).
    fn topological_order(&self,
    ) -> Result<Vec<usize>, TransformError> {
        self.validate()?;
        let n = self.nodes.len();
        let mut in_degree = vec![0usize; n];
        for deps in self.deps.values() {
            for &d in deps {
                if d < n {
                    in_degree[d] += 1;
                }
            }
        }

        let mut queue: Vec<usize> = (0..n).filter(|i| in_degree[*i] == 0).collect();
        let mut order = Vec::with_capacity(n);

        while let Some(u) = queue.pop() {
            order.push(u);
            for v in 0..n {
                if self
                    .deps
                    .get(&v)
                    .map(|d| d.contains(&u))
                    .unwrap_or(false)
                {
                    in_degree[v] -= 1;
                    if in_degree[v] == 0 {
                        queue.push(v);
                    }
                }
            }
        }

        if order.len() != n {
            return Err(TransformError::Internal(
                "impossible de topologiser le plan".to_string(),
            ));
        }
        Ok(order)
    }
}

/// Orchestrateur : exécute un plan de transforms.
pub struct Orchestrator {
    transforms: HashMap<String, Arc<dyn Transform>>,
    cache: Arc<ResponseCache>,
    rate_limiter: Arc<RateLimiter<String, DefaultKeyedStateStore<String>, DefaultClock>>,
}

impl Orchestrator {
    /// Crée un orchestrateur avec une limite de débit par domaine.
    pub fn new(
        transforms: Vec<Arc<dyn Transform>>,
        requests_per_second: u32,
    ) -> Self {
        let mut map = HashMap::new();
        for t in transforms {
            map.insert(t.id().to_string(), t);
        }
        let quota = Quota::per_second(
            NonZeroU32::new(requests_per_second.max(1)).expect("quota non nul"),
        );
        Self {
            transforms: map,
            cache: Arc::new(ResponseCache::new(std::time::Duration::from_secs(300))),
            rate_limiter: Arc::new(RateLimiter::keyed(quota)),
        }
    }

    /// Exécute un plan complet et renvoie les résultats par nœud.
    pub async fn execute(
        &self,
        ctx: TransformContext,
        plan: &ExecutionPlan,
        cancel: CancellationToken,
    ) -> Result<Vec<NodeResult>, TransformError> {
        let order = plan.topological_order()?;
        let mut results: Vec<Option<NodeResult>> = vec![None; plan.nodes.len()];

        for idx in order {
            if cancel.is_cancelled() {
                return Err(TransformError::Cancelled);
            }

            let node = &plan.nodes[idx];
            let transform = self
                .transforms
                .get(&node.transform_id)
                .ok_or_else(|| {
                    TransformError::Internal(format!(
                        "transform '{}' non enregistré",
                        node.transform_id
                    ))
                })?;

            // Vérifier le cache.
            let cache_key = CacheKey {
                transform_id: node.transform_id.clone(),
                entity_kind: format!("{:?}", node.input.entity.kind),
                canonical_value: node.input.entity.canonical_value.clone(),
                params_hash: hash_params(&node.input.params,
                ),
            };

            if let Some(cached) = self.cache.get(&cache_key) {
                if let Ok(output) = serde_json::from_str::<TransformOutput>(&cached
                ) {
                    results[idx] = Some(NodeResult {
                        transform_id: node.transform_id.clone(),
                        output: Ok(output),
                        elapsed_ms: 0,
                        from_cache: true,
                    });
                    continue;
                }
            }

            // Rate limiting par domaine (simplifié : on utilise l'id du transform
            // comme clé de rate limiting ; les implémentations spécifiques
            // pourraient utiliser le domaine cible).
            self.rate_limiter
                .until_key_ready(&node.transform_id.clone())
                .await;

            let t0 = std::time::Instant::now();
            let output = transform.execute(ctx.clone(), node.input.clone()).await;
            let elapsed_ms = t0.elapsed().as_millis() as u64;

            if let Ok(ref out) = output {
                if let Ok(json) = serde_json::to_string(out) {
                    self.cache.insert(cache_key, json, None);
                }
            }

            results[idx] = Some(NodeResult {
                transform_id: node.transform_id.clone(),
                output,
                elapsed_ms,
                from_cache: false,
            });
        }

        Ok(results.into_iter().flatten().collect())
    }
}

/// Hash simple des paramètres pour la clé de cache.
fn hash_params(params: &std::collections::BTreeMap<String, serde_json::Value>) -> String {
    let mut s = String::new();
    for (k, v) in params {
        s.push_str(k);
        s.push('=');
        s.push_str(&v.to_string());
        s.push(';');
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;
    use spectra_core::{Entity, EntityId, EntityKind};

    struct DummyTransform {
        id: String,
    }

    impl Transform for DummyTransform {
        fn id(&self) -> &str {
            &self.id
        }
        fn display_name(&self) -> &str {
            &self.id
        }
        fn input_kinds(&self) -> &[spectra_core::EntityKind] {
            &[EntityKind::Domain]
        }
        fn execute(
            &self,
            _ctx: TransformContext,
            _input: TransformInput,
        ) -> std::pin::Pin<
            Box<
                dyn std::future::Future<Output = Result<TransformOutput, TransformError>>
                    + Send
                    + '_,
            >,
        > {
            Box::pin(async move { Ok(TransformOutput::default()) })
        }
    }

    #[tokio::test]
    async fn plan_execution() {
        let t = Arc::new(DummyTransform {
            id: "dummy".to_string(),
        });
        let orch = Orchestrator::new(vec![t], 10);
        let mut plan = ExecutionPlan::new();
        let input = TransformInput {
            entity: Entity {
                id: "e1".to_string(),
                kind: EntityKind::Domain,
                canonical_value: "example.com".to_string(),
                display_label: "example.com".to_string(),
                properties: Default::default(),
                created_at: "2026-01-01T00:00:00Z".to_string(),
                merged_from: vec![],
            },
            params: Default::default(),
        };
        plan.add_node("dummy".to_string(), input);

        let ctx = TransformContext {
            case_id: "c1".to_string(),
            network_profile: "direct".to_string(),
            user_agent: "SPECTRA/0.1".to_string(),
            operator: "test".to_string(),
        };

        let results = orch.execute(ctx, &plan, CancellationToken::new()).await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].transform_id, "dummy");
        assert!(results[0].output.is_ok());
    }

    #[test]
    fn cycle_detection() {
        let mut plan = ExecutionPlan::new();
        let input = TransformInput {
            entity: Entity {
                id: "e1".to_string(),
                kind: EntityKind::Domain,
                canonical_value: "example.com".to_string(),
                display_label: "example.com".to_string(),
                properties: Default::default(),
                created_at: "2026-01-01T00:00:00Z".to_string(),
                merged_from: vec![],
            },
            params: Default::default(),
        };
        let a = plan.add_node("a".to_string(), input.clone());
        let b = plan.add_node("b".to_string(), input);
        plan.add_dependency(a, b);
        plan.add_dependency(b, a); // cycle

        assert!(plan.validate().is_err());
    }
}
