use anyhow::Result;

use maud::{html, Markup, PreEscaped};
use serde_json::json;

#[cfg(feature = "with_predictions")]
use serde_json::Value;

use sqlx::SqlitePool;

use crate::{db::statement_stats, structs::StatementStats};

pub async fn yes_no_pie_chart(statement_id: i64, pool: &SqlitePool) -> Result<Markup> {
    let StatementStats {
        total_votes,
        yes_votes,
        no_votes,
        ..
    } = statement_stats(statement_id, pool).await?;
    if total_votes == 0 {
        Ok(html! {})
    } else {
        Ok(apex_chart(
            format!(
                r##" 
                {{
                  "labels": [
                    "Yes",
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
                     }},
                     "animations": {{ "enabled": false }},
                  }},
                  "colors": [
                    "#16a34a",
                    "#dc2626",
                  ],
                  "tooltip": {{ "enabled": false }},
                  "dataLabels": {{
                    "enabled": true,
                    "formatter": function(value, {{ seriesIndex, dataPointIndex, w }}) {{
                      let percentage = Math.round(value);
                      return `${{w.config.labels[seriesIndex]}}:  ${{w.config.series[seriesIndex]}} (${{percentage}}%)`
                    }},
                    "style": {{
                      "colors": [
                        "#16a34a",
                        "#dc2626",
                      ],
                    }},
                    "background": {{
                      "enabled": true,
                      "opacity": 1,
                      "dropShadow": {{
                        "enabled": false
                      }}
                    }},
                    "dropShadow": {{
                      "enabled": false
                    }}
                  }},
                  "legend": {{
                    "show": false
                  }},
                  "series": {}
                }}"##,
                json!([yes_votes, no_votes]),
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
            // creating a local variable scope to avoid polluting the global scope
            // see https://en.wikipedia.org/wiki/Immediately_invoked_function_expression
            (PreEscaped(format!(r##"
            (() => {{
                var options = {options};

                if (window.propolisDarkMode) {{
                    options.theme = {{"mode": "dark"}};
                }}
                var elem = document.querySelector("#{chart_id}")
                elem.innerHTML = ""; // avoid rendering twice
                new ApexCharts(elem, options).render();
            }})()
            "##)))
        }
    }
}

#[cfg(feature = "with_predictions")]
pub fn weighted_user_chart(title: &str, data_json: Value) -> Markup {
    apex_chart(
        format!(
            r#"
                {{
          series: [{{
          name: 'Weighted votes',
          data: {data_json},
        }},
],
          chart: {{
          background: 'transparent',
          height: 350,
          type: 'radar',
        }},
        title: {{
          text: '{title}'
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
    )
}

#[cfg(feature = "with_predictions")]
/// Yield code for a radar chart displaying the weighted stats
pub fn user_radar_chart(
    title: &str,
    weights_map: &std::collections::HashMap<String, crate::db::UserStat>,
) -> Result<Markup> {
    use std::collections::HashMap;

    use crate::db::UserStat;
    let mut hash_vec: Vec<(&String, &crate::db::UserStat)> = weights_map.iter().collect();
    hash_vec.sort_unstable_by_key(|item| (((item.1.votes_weight * -100_f64) as i64), item.0));
    let hash_vec: Vec<&(&String, &UserStat)> = hash_vec.iter().take(5).collect();

    let mut data: Vec<HashMap<&str, String>> = vec![];
    for &(
        ideology,
        crate::db::UserStat {
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
    Ok(weighted_user_chart(title, data_json))
}
