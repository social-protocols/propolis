use maud::{html, Markup, PreEscaped};
use serde_json::json;
use sqlx::SqlitePool;

use crate::{db::statement_stats, error::Error, structs::StatementStats};

pub async fn yes_no_pie_chart(statement_id: i64, pool: &SqlitePool) -> Result<Markup, Error> {
    let StatementStats {
        total_votes,
        yes_votes,
        no_votes,
        itdepends_votes,
        ..
    } = statement_stats(statement_id, &pool).await?;
    if total_votes == 0 {
        Ok(html! {})
    } else {
        Ok(apex_chart(
            format!(
                r#" 
                {{
                  "labels": [
                    "Yes",
                    "It depends",
                    "No"
                  ],
                  "chart": {{
                    "type": "pie",
                    "width": 180,
                     events: {{
                       dataPointMouseEnter: function(event) {{
                           // workaround from https://stackoverflow.com/questions/68503392/apexcharts-cursor-pointer
                           event.target.style.cursor = "pointer";
                       }},
                       click: function(event, chartContext, config) {{
                           // workaround from https://github.com/apexcharts/apexcharts.js/issues/2251#issuecomment-904377385
                           const seriesIndex = event.target.parentElement.getAttribute("data:realIndex")
                           const targetQuery = ['target_yes', 'target_itdepends', 'target_no'];
                           location.href = `/new?target=${{{statement_id}}}&${{targetQuery[seriesIndex]}}=true`;
                       }}
                     }}
                  }},
                  "colors": [
                    "forestgreen",
                    "slategrey",
                    "firebrick"
                  ],
                  "dataLabels": {{
                    "enabled": false
                  }},
                  "legend": {{
                    "show": false
                  }},
                  "series": {}
                }}"#,
                json!([yes_votes, itdepends_votes, no_votes]).to_string(),
            )
            .as_str(),
        ))
    }
}

pub fn apex_chart(options: &str) -> Markup {
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
