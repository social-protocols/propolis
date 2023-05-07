use anyhow::Result;

use maud::{html, Markup, PreEscaped};
use serde_json::json;
use sqlx::SqlitePool;

use crate::{db::statement_stats, structs::StatementStats};

pub async fn yes_no_pie_chart(statement_id: i64, pool: &SqlitePool) -> Result<Markup> {
    let StatementStats {
        total_votes,
        yes_votes,
        no_votes,
        itdepends_votes,
        ..
    } = statement_stats(statement_id, pool).await?;
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
                    "width": 150,
                     events: {{
                       dataPointMouseEnter: function(event) {{
                           // workaround from https://stackoverflow.com/questions/68503392/apexcharts-cursor-pointer
                           event.target.style.cursor = "pointer";
                       }},
                       click: function(event, chartContext, config) {{
                           // workaround from https://github.com/apexcharts/apexcharts.js/issues/2251#issuecomment-904377385
                           const seriesIndex = event.target.parentElement.getAttribute("data:realIndex")
                           const targetQuery = ['target_yes', 'target_all', 'target_no'];
                           location.href = `/new?target=${{{statement_id}}}&${{targetQuery[seriesIndex]}}=true`;
                       }}
                     }}
                  }},
                  "colors": [
                    "\#16a34a",
                    "\#64748b",
                    "\#dc2626",
                  ],
                  "dataLabels": {{
                    "enabled": false
                  }},
                  "legend": {{
                    "show": false
                  }},
                  "series": {}
                }}"#,
                json!([yes_votes, itdepends_votes, no_votes]),
            )
            .as_str(),
        ))
    }
}

#[cfg(feature = "with_predictions")]
/// Yield code for a radar chart displaying the most common ideologies of a user
pub fn ideologies_radar_chart(
    ideologies_map: &std::collections::HashMap<String, crate::db::UserIdeologyStats>,
) -> Result<Markup> {
    use std::collections::HashMap;
    let mut hash_vec: Vec<(&String, &crate::db::UserIdeologyStats)> =
        ideologies_map.iter().collect();
    hash_vec.sort_by(|a, b| {
        b.1.votes_weight
            .partial_cmp(&a.1.votes_weight)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let mut data: Vec<HashMap<&str, String>> = vec![];
    for (
        ideology,
        crate::db::UserIdeologyStats {
            votes_cast,
            votes_weight,
        },
    ) in hash_vec
    {
        data.push(HashMap::from([
            ("x", ideology.into()),
            ("y", votes_weight.to_string()),
            ("total", votes_cast.to_string()),
        ]));
    }
    let data_json = json!(data);
    Ok(apex_chart(
        format!(
            r#"
                {{
          series: [{{
          name: 'Weighted votes',
          data: {data_json},
        }},
],
          chart: {{
          height: 350,
          type: 'radar',
        }},
        title: {{
          text: 'Ideologies'
        }},
        xaxis: {{
          type: 'category'
        }},
        yaxis: {{
          tickAmount: 4,
          labels: {{
            formatter: function(val, i) {{
              return (Math.round(val * 100) / 100).toFixed(2);
            }}
          }}
        }},
        tooltip: {{
          y: {{
            formatter: function(value, {{ series, seriesIndex, dataPointIndex, w }}) {{
              var data = w.globals.initialSeries[seriesIndex].data[dataPointIndex];
              return (Math.round(value * 100) / 100).toFixed(2) + " (Score: " + data['total'] + ")";
            }}
          }}
        }}
        }}"#,
        )
        .as_str(),
    ))
}

pub fn apex_chart(options: &str) -> Markup {
    let uuid = uuid::Uuid::new_v4();
    let chart_id = format!("chart-{uuid}");

    html! {
        div id=(chart_id) {}
        script {
            (PreEscaped(format!("var options = {options};")))
                (PreEscaped(format!("var chart = new ApexCharts(document.querySelector(\"#{chart_id}\"), options);")))
            (PreEscaped("chart.render();"))
        }
    }
}
