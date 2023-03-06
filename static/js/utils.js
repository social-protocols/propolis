function setupChart(selector, agree, skip, disagree) {
    var options = {
        colors: ["#00FF00", "#AAAAAA", "#FF0000"],
        tooltip: {
            enabled: false,
        },
        chart: {
            height: "80px",
            type: 'bar',
            toolbar: {
                show: false
            },
        },
        plotOptions: {
            bar: {
                borderRadius: 0,
                horizontal: true,
                distributed: true,
            }
        },
        xaxis: {
            categories: ['Agree', 'Skip', 'Disagree'],
            axisBorder: {
                show: false,
            },
            axisTicks: {
                show: false,
            },
            labels: {
                show: false,
            }
        },
        legend: {
            show: false
        },
        yaxis: {
            labels: {
                show: false
            }
        },
        series: [{
            data: [agree, skip, disagree]
        }],
    };

    var chart = new ApexCharts(document.querySelector(selector), options);
    chart.render();
}
