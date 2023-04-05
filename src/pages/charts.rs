use anyhow::Result;
use std::collections::HashMap;

use maud::{html, Markup, PreEscaped};
use serde_json::json;
use sqlx::SqlitePool;

use crate::{
    db::statement_stats,
    structs::{StatementStats, User},
};

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
                json!([yes_votes, itdepends_votes, no_votes]),
            )
            .as_str(),
        ))
    }
}

/// Yield code for a radar chart displaying the most common ideologies of a user
pub async fn ideologies_radar_chart(
    ideologies_map: &HashMap<String, (i64, f64)>
) -> Result<Markup, Error> {
    let mut hash_vec: Vec<(&String, &(i64, f64))> = ideologies_map.iter().collect();
    hash_vec.sort_by(|a, b| b.1.partial_cmp(a.1).unwrap());

    let _ideologies = json!(hash_vec.iter().map(|i| i.0).collect::<Vec<&String>>());
    let _nums = json!(hash_vec.iter().map(|i| i.1.0).collect::<Vec<i64>>());
    let _weights = json!(hash_vec.iter().map(|i| i.1.1).collect::<Vec<f64>>());
    let mut data : Vec<HashMap<&str, String>> = vec![];
    for (ideology, (total, weighted)) in hash_vec {
        data.push(HashMap::from([("x", ideology.into()),
                                 ("y", weighted.to_string()),
                                 ("total", total.to_string())]));
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
