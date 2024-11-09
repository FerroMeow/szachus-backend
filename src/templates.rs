use anyhow::anyhow;
use handlebars::Handlebars;
use serde::Serialize;

pub(crate) async fn get_template<T>(
    hb: &Handlebars<'_>,
    path: &str,
    data: &T,
) -> anyhow::Result<String>
where
    T: Serialize,
{
    hb.render_template(
        &String::from_utf8(tokio::fs::read(format!("templates/{path}.hbs")).await?)?,
        data,
    )
    .map_err(|_| anyhow!("Template not found."))
}
