/// AI-assisted dependency advice.
#[derive(Debug, Default)]
pub struct DependencyAdvisor;

impl DependencyAdvisor {
    pub fn advise_dependencies(&self, dependencies: &[String]) -> Vec<String> {
        let mut recommendations = Vec::new();
        if dependencies.iter().any(|d| d == "express") {
            recommendations
                .push("Consider fastify for higher throughput HTTP services.".to_string());
        }
        recommendations
    }
}
