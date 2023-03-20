use maud::{html, Markup, PreEscaped};
use serde_json::json;
use sqlx::SqlitePool;

use crate::{db::statement_stats, error::Error, structs::StatementStats};

pub async fn yes_no_pie_chart(statement_id: i64, pool: &SqlitePool) -> Result<Markup, Error> {
    let StatementStats {
        yes_votes,
        no_votes,
        itdepends_votes,
        ..
    } = statement_stats(statement_id, &pool).await?;
    Ok(html! {
        (apex_chart(json!({
            "series": [yes_votes, itdepends_votes, no_votes],
            "labels": ["Yes", "It depends", "No"],
            "colors": ["forestgreen", "darkorange", "firebrick"],
            "chart": {
                "width": 180,
                "type": "pie",
            },
            "legend": {
                "position": "bottom",
                "labels": {
                    "colors": "var(--cfg)",
                },
            },
            "dataLabels": {
                "enabled": false,
            },
        }).to_string()))
    })
}

pub fn apex_chart(options: String) -> Markup {
    let uuid = uuid::Uuid::new_v4();
    let chart_id = format!("chart-{uuid}");

    html! {
        div id=(chart_id) {}
        script {
            (PreEscaped(format!("var options = {};", options)))
            (PreEscaped(format!("var chart = new ApexCharts(document.querySelector(\"#{}\"), options);", chart_id)))
            (PreEscaped("chart.render();"))
        }
    }
}
