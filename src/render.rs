use anyhow::{Context, Result};
use minijinja::Environment;
use serde::Serialize;

pub(crate) fn render_template(
    name: &str,
    template: &str,
    context: &impl Serialize,
) -> Result<String> {
    let mut env = Environment::new();
    env.add_template(name, template)
        .with_context(|| format!("failed to load template {name}"))?;
    let rendered = env
        .get_template(name)
        .with_context(|| format!("failed to resolve template {name}"))?
        .render(context)
        .with_context(|| format!("failed to render template {name}"))?;

    Ok(rendered.replace("\r\n", "\n"))
}

#[cfg(test)]
mod tests {
    use super::render_template;
    use serde::Serialize;

    #[derive(Serialize)]
    struct Context {
        value: &'static str,
    }

    #[test]
    fn render_template_normalizes_newlines() {
        let rendered = render_template(
            "test",
            "first\r\n{{ value }}\r\nthird",
            &Context { value: "second" },
        )
        .expect("template should render");

        assert_eq!(rendered, "first\nsecond\nthird");
    }
}
