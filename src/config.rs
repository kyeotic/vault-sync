use anyhow::{Context, Result};
use indexmap::IndexMap;
use minijinja::{Environment, context};
use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;

const CONFIG_FILE: &str = ".vault-sync.toml";
const BWS_FILE: &str = ".bws";

#[derive(Deserialize)]
pub struct Config {
    #[serde(default = "default_max_threads")]
    pub max_threads: usize,
    #[serde(default = "default_max_retries")]
    pub max_retries: u32,
    pub secrets: IndexMap<String, SecretMapping>,
}

fn default_max_threads() -> usize {
    3
}

fn default_max_retries() -> u32 {
    3
}

#[derive(Deserialize)]
pub struct SecretMapping {
    #[serde(skip)]
    pub name: String,
    pub id: String,
    pub path: String,
}

impl Config {
    pub fn load() -> Result<Self> {
        let content = std::fs::read_to_string(CONFIG_FILE)
            .with_context(|| format!("Could not read {CONFIG_FILE}"))?;

        let mut config: Config =
            toml::from_str(&content).with_context(|| format!("Failed to parse {CONFIG_FILE}"))?;

        for (key, secret) in &mut config.secrets {
            secret.name = key.clone();
        }

        config.expand_templates()?;
        Ok(config)
    }

    fn expand_templates(&mut self) -> Result<()> {
        let env_vars: HashMap<String, String> = std::env::vars().collect();

        for (name, secret) in &mut self.secrets {
            secret.path = expand_template(&secret.path, &env_vars)
                .with_context(|| format!("Failed to expand template in path for secret '{name}': {}", secret.path))?;
        }

        Ok(())
    }
}

fn expand_template(template: &str, env_vars: &HashMap<String, String>) -> Result<String> {
    // Skip processing if no template markers present
    if !template.contains("{{") {
        return Ok(template.to_string());
    }

    let mut jinja = Environment::new();
    jinja.add_template("_", template)?;
    let tmpl = jinja.get_template("_")?;

    Ok(tmpl.render(context! { env => env_vars })?)
}

pub fn resolve_bws_token() -> Result<String> {
    if let Ok(token) = std::env::var("BWS_ACCESS_TOKEN")
        && !token.is_empty()
    {
        return Ok(token);
    }

    let home = PathBuf::from(std::env::var("HOME").context("HOME not set")?);
    let mut dir = std::env::current_dir().context("Failed to get current directory")?;

    loop {
        let candidate = dir.join(BWS_FILE);
        if candidate.is_file() {
            let content = std::fs::read_to_string(&candidate)
                .with_context(|| format!("Failed to read {}", candidate.display()))?;

            for line in content.lines() {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }
                if let Some((key, value)) = line.split_once('=')
                    && key.trim() == "BWS_ACCESS_TOKEN"
                {
                    let value = value.trim().trim_matches('"').trim_matches('\'');
                    if !value.is_empty() {
                        return Ok(value.to_string());
                    }
                }
            }

            anyhow::bail!(
                "Found {} but it does not contain BWS_ACCESS_TOKEN",
                candidate.display()
            );
        }

        if dir == home || !dir.starts_with(&home) {
            break;
        }
        if !dir.pop() {
            break;
        }
    }

    anyhow::bail!(
        "BWS_ACCESS_TOKEN not found. Set it as an environment variable or in a {BWS_FILE} file."
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expand_template_no_markers_returns_unchanged() {
        let env_vars = HashMap::new();
        let result = expand_template("plain/path/file.env", &env_vars).unwrap();
        assert_eq!(result, "plain/path/file.env");
    }

    #[test]
    fn expand_template_substitutes_env_var() {
        let mut env_vars = HashMap::new();
        env_vars.insert("HOME".to_string(), "/home/user".to_string());
        let result = expand_template("{{ env.HOME }}/projects/.env", &env_vars).unwrap();
        assert_eq!(result, "/home/user/projects/.env");
    }

    #[test]
    fn expand_template_substitutes_multiple_vars() {
        let mut env_vars = HashMap::new();
        env_vars.insert("HOME".to_string(), "/home/user".to_string());
        env_vars.insert("PROJECT".to_string(), "myapp".to_string());
        let result =
            expand_template("{{ env.HOME }}/{{ env.PROJECT }}/.env", &env_vars).unwrap();
        assert_eq!(result, "/home/user/myapp/.env");
    }

    #[test]
    fn expand_template_missing_var_returns_empty() {
        let env_vars = HashMap::new();
        let result = expand_template("{{ env.MISSING }}/.env", &env_vars).unwrap();
        assert_eq!(result, "/.env");
    }

    #[test]
    fn expand_template_mixed_literal_and_var() {
        let mut env_vars = HashMap::new();
        env_vars.insert("USER".to_string(), "alice".to_string());
        let result = expand_template("/home/{{ env.USER }}/config", &env_vars).unwrap();
        assert_eq!(result, "/home/alice/config");
    }
}
