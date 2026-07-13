use anyhow::{Context, Result};
use minijinja::Environment;

use crate::catalog::RenderSkill;

pub(crate) fn render_skill(template: &str, skill: &RenderSkill) -> Result<String> {
    let mut env = Environment::new();
    env.add_template("skill.md.j2", template)
        .context("failed to load skill template")?;
    let rendered = env
        .get_template("skill.md.j2")
        .context("failed to resolve skill template")?
        .render(skill)
        .context("failed to render skill template")?;

    Ok(rendered)
}

pub(crate) fn generated_reference(source: &str) -> String {
    source.to_owned()
}
