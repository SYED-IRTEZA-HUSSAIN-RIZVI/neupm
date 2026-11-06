/// AI-assisted package discovery and query interface.
#[derive(Debug, Default)]
pub struct PackageAssistant;

impl PackageAssistant {
    pub fn suggest_alternative(&self, package_name: &str) -> Option<String> {
        match package_name {
            "moment" => Some("dayjs or date-fns (smaller bundle size)".to_string()),
            "request" => Some("undici or reqwest (maintained and performant)".to_string()),
            _ => None,
        }
    }
}
