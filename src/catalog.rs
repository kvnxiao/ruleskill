use std::collections::HashSet;
use std::env;
use std::fs;

use anyhow::{anyhow, bail, Context, Result};
use camino::{Utf8Component, Utf8Path, Utf8PathBuf};
use serde::{Deserialize, Serialize};
use strsim::jaro_winkler;
use walkdir::WalkDir;

use crate::render;

const CATALOG_DIR_ENV: &str = "RULESKILL_CATALOG_DIR";

#[derive(Debug)]
pub(crate) struct Catalog {
    root: Utf8PathBuf,
    template: String,
    skills: Vec<Skill>,
}

#[derive(Debug)]
pub(crate) struct Skill {
    folder_name: String,
    dir: Utf8PathBuf,
    manifest_path: Utf8PathBuf,
    manifest: SkillManifest,
}

#[derive(Debug, Deserialize)]
struct SkillManifest {
    name: Option<String>,
    title: Option<String>,
    description: Option<String>,
    #[serde(default)]
    rules: Vec<RuleManifest>,
}

#[derive(Debug, Deserialize)]
struct RuleManifest {
    title: Option<String>,
    file: Option<String>,
    when: Option<String>,
}

#[derive(Debug, Clone)]
pub(crate) struct ResolvedSkill {
    pub(crate) render: RenderSkill,
    pub(crate) references: Vec<ResolvedReference>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct RenderSkill {
    name: String,
    pub(crate) output_name: String,
    title: String,
    description: String,
    description_yaml: String,
    rules: Vec<RenderRule>,
}

#[derive(Debug, Clone, Serialize)]
struct RenderRule {
    title: String,
    file: String,
    when: String,
    reference_file: String,
}

#[derive(Debug, Clone)]
pub(crate) struct ResolvedReference {
    pub(crate) source: Utf8PathBuf,
    pub(crate) reference_file: String,
}

impl Catalog {
    pub(crate) fn load_default() -> Result<Self> {
        Self::load(default_catalog_root()?)
    }

    pub(crate) fn load(root: Utf8PathBuf) -> Result<Self> {
        let template_path = root.join("templates").join("skill.md.j2");
        let template = fs::read_to_string(&template_path)
            .with_context(|| format!("failed to read template at {template_path}"))?;

        let rules_dir = root.join("rules");
        if !rules_dir.is_dir() {
            bail!("catalog rules directory does not exist: {rules_dir}");
        }

        let mut skills = Vec::new();
        for entry in WalkDir::new(&rules_dir).min_depth(2).max_depth(2) {
            let entry = entry.with_context(|| format!("failed to walk {rules_dir}"))?;
            let is_manifest =
                entry.path().file_name().and_then(|name| name.to_str()) == Some("skill.toml");
            if !entry.file_type().is_file() || !is_manifest {
                continue;
            }

            let manifest_path = path_to_utf8(entry.path())?;
            let dir = manifest_path
                .parent()
                .ok_or_else(|| anyhow!("manifest has no parent directory: {manifest_path}"))?
                .to_path_buf();
            let folder_name = dir
                .file_name()
                .ok_or_else(|| anyhow!("skill directory has no name: {dir}"))?
                .to_owned();
            let manifest_text = fs::read_to_string(&manifest_path)
                .with_context(|| format!("failed to read {manifest_path}"))?;
            let manifest = toml::from_str::<SkillManifest>(&manifest_text)
                .with_context(|| format!("failed to parse {manifest_path}"))?;

            skills.push(Skill {
                folder_name,
                dir,
                manifest_path,
                manifest,
            });
        }

        skills.sort_by(|left, right| left.display_name().cmp(right.display_name()));

        Ok(Self {
            root,
            template,
            skills,
        })
    }

    pub(crate) fn root(&self) -> &Utf8Path {
        &self.root
    }

    pub(crate) fn template(&self) -> &str {
        &self.template
    }

    pub(crate) fn skills(&self) -> &[Skill] {
        &self.skills
    }

    pub(crate) fn validate(&self) -> Result<()> {
        let mut errors = Vec::new();
        for skill in &self.skills {
            skill.validate(&self.template, &mut errors);
        }
        finish_validation(&errors)
    }

    pub(crate) fn find_validated_skill(&self, name: &str) -> Result<&Skill> {
        let skill = self
            .skills
            .iter()
            .find(|skill| skill.name() == Some(name))
            .ok_or_else(|| self.unknown_skill_error(name))?;

        let mut errors = Vec::new();
        skill.validate(&self.template, &mut errors);
        finish_validation(&errors)?;
        Ok(skill)
    }

    fn unknown_skill_error(&self, name: &str) -> anyhow::Error {
        match self.closest_skill_name(name) {
            Some(suggestion) => anyhow!("unknown skill '{name}'. Did you mean '{suggestion}'?"),
            None => {
                anyhow!("unknown skill '{name}'. Run 'ruleskill list' to see available skills.")
            }
        }
    }

    fn closest_skill_name(&self, name: &str) -> Option<&str> {
        self.skills
            .iter()
            .filter_map(Skill::name)
            .map(|candidate| (candidate, jaro_winkler(name, candidate)))
            .filter(|(_, score)| *score >= 0.7)
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(candidate, _)| candidate)
    }
}

impl Skill {
    pub(crate) fn display_name(&self) -> &str {
        self.name().unwrap_or(&self.folder_name)
    }

    pub(crate) fn name(&self) -> Option<&str> {
        non_empty(self.manifest.name.as_deref())
    }

    pub(crate) fn resolve(&self) -> Result<ResolvedSkill> {
        let mut rules = Vec::with_capacity(self.manifest.rules.len());
        let mut references = Vec::with_capacity(self.manifest.rules.len());

        for rule in &self.manifest.rules {
            let file = required(rule.file.as_deref(), "rule file")?;
            let source = self.dir.join(file);
            let reference_file = output_reference_file(file)?;

            rules.push(RenderRule {
                title: required(rule.title.as_deref(), "rule title")?.to_owned(),
                file: file.to_owned(),
                when: required(rule.when.as_deref(), "rule when")?.to_owned(),
                reference_file: reference_file.clone(),
            });
            references.push(ResolvedReference {
                source,
                reference_file,
            });
        }

        let name = required(self.manifest.name.as_deref(), "skill name")?;
        let description = required(self.manifest.description.as_deref(), "skill description")?;

        Ok(ResolvedSkill {
            render: RenderSkill {
                name: name.to_owned(),
                output_name: output_skill_name(name),
                title: required(self.manifest.title.as_deref(), "skill title")?.to_owned(),
                description: description.to_owned(),
                description_yaml: yaml_double_quoted(description),
                rules,
            },
            references,
        })
    }

    fn validate(&self, template: &str, errors: &mut Vec<String>) {
        let mut local_errors = Vec::new();
        let subject = format!("{}:", self.manifest_path);

        match non_empty(self.manifest.name.as_deref()) {
            Some(name) => {
                if name != self.folder_name {
                    local_errors.push(format!(
                        "{subject} name '{name}' must match folder '{}'",
                        self.folder_name
                    ));
                }
                if !is_kebab_case(name) {
                    local_errors.push(format!("{subject} name '{name}' must be kebab-case"));
                }
            }
            None => local_errors.push(format!("{subject} name is required")),
        }

        if non_empty(self.manifest.title.as_deref()).is_none() {
            local_errors.push(format!("{subject} title is required"));
        }
        if non_empty(self.manifest.description.as_deref()).is_none() {
            local_errors.push(format!("{subject} description is required"));
        }
        if self.manifest.rules.is_empty() {
            local_errors.push(format!(
                "{subject} at least one [[rules]] entry is required"
            ));
        }

        let mut seen_files = HashSet::new();
        let mut seen_outputs = HashSet::new();
        for (index, rule) in self.manifest.rules.iter().enumerate() {
            let rule_subject = format!("{subject} rules[{}]", index + 1);

            if non_empty(rule.title.as_deref()).is_none() {
                local_errors.push(format!("{rule_subject} title is required"));
            }
            if non_empty(rule.when.as_deref()).is_none() {
                local_errors.push(format!("{rule_subject} when is required"));
            }

            let Some(file) = non_empty(rule.file.as_deref()) else {
                local_errors.push(format!("{rule_subject} file is required"));
                continue;
            };

            if !is_safe_relative_path(file) {
                local_errors.push(format!(
                    "{rule_subject} file '{file}' must stay inside the skill folder"
                ));
                continue;
            }

            let source = self.dir.join(file);
            if !source.is_file() {
                local_errors.push(format!(
                    "{rule_subject} referenced file does not exist: {source}"
                ));
            }

            if !seen_files.insert(file.to_owned()) {
                local_errors.push(format!("{rule_subject} duplicate rule file '{file}'"));
            }

            match output_reference_file(file) {
                Ok(reference_file) => {
                    let output = format!("references/{reference_file}");
                    if !seen_outputs.insert(output.clone()) {
                        local_errors.push(format!(
                            "{rule_subject} duplicate output reference path '{output}'"
                        ));
                    }
                }
                Err(err) => local_errors.push(format!("{rule_subject} {err:#}")),
            }
        }

        if local_errors.is_empty() {
            match self
                .resolve()
                .and_then(|resolved| render::render_skill(template, &resolved.render))
            {
                Ok(_) => {}
                Err(err) => {
                    local_errors.push(format!("{subject} template rendering failed: {err:#}"));
                }
            }
        }

        errors.extend(local_errors);
    }
}

fn default_catalog_root() -> Result<Utf8PathBuf> {
    if let Some(root) = env::var_os(CATALOG_DIR_ENV) {
        return Utf8PathBuf::from_path_buf(root.into())
            .map_err(|path| anyhow!("catalog path is not valid UTF-8: {}", path.display()));
    }

    Ok(Utf8PathBuf::from(env!("CARGO_MANIFEST_DIR")))
}

fn finish_validation(errors: &[String]) -> Result<()> {
    if errors.is_empty() {
        Ok(())
    } else {
        bail!("validation failed:\n{}", errors.join("\n"));
    }
}

fn path_to_utf8(path: &std::path::Path) -> Result<Utf8PathBuf> {
    Utf8PathBuf::from_path_buf(path.to_path_buf())
        .map_err(|path| anyhow!("path is not valid UTF-8: {}", path.display()))
}

fn non_empty(value: Option<&str>) -> Option<&str> {
    value.map(str::trim).filter(|value| !value.is_empty())
}

fn required<'a>(value: Option<&'a str>, name: &str) -> Result<&'a str> {
    non_empty(value).ok_or_else(|| anyhow!("{name} is required"))
}

fn is_kebab_case(value: &str) -> bool {
    let mut previous_was_hyphen = false;
    let mut saw_char = false;

    for byte in value.bytes() {
        match byte {
            b'a'..=b'z' | b'0'..=b'9' => {
                previous_was_hyphen = false;
                saw_char = true;
            }
            b'-' if saw_char && !previous_was_hyphen => {
                previous_was_hyphen = true;
            }
            _ => return false,
        }
    }

    saw_char && !previous_was_hyphen
}

fn is_safe_relative_path(path: &str) -> bool {
    let path = Utf8Path::new(path);
    !path.is_absolute()
        && path
            .components()
            .all(|component| matches!(component, Utf8Component::Normal(_) | Utf8Component::CurDir))
}

fn output_reference_file(file: &str) -> Result<String> {
    Utf8Path::new(file)
        .file_name()
        .map(str::to_owned)
        .ok_or_else(|| anyhow!("file '{file}' has no filename"))
}

fn output_skill_name(name: &str) -> String {
    if name.ends_with("-rules") {
        name.to_owned()
    } else {
        format!("{name}-rules")
    }
}

fn yaml_double_quoted(value: &str) -> String {
    let mut quoted = String::with_capacity(value.len() + 2);
    quoted.push('"');

    for ch in value.chars() {
        match ch {
            '\\' => quoted.push_str("\\\\"),
            '"' => quoted.push_str("\\\""),
            '\n' => quoted.push_str("\\n"),
            '\r' => quoted.push_str("\\r"),
            '\t' => quoted.push_str("\\t"),
            _ => quoted.push(ch),
        }
    }

    quoted.push('"');
    quoted
}
