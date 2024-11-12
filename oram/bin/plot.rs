use plotters::prelude::*;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;

fn read_stash_data(filename: &str) -> Vec<(i32, f64)> {
    let mut data = Vec::new();

    if let Ok(lines) = read_lines(filename) {
        for (i, line) in lines.flatten().enumerate() {
            // Skip the first line as it only indicates the total number of accesses
            if i == 0 {
                continue;
            }
            if let Some((r, count)) = parse_line(&line) {
                data.push((r, count));
            }
        }
    }

    data
}

fn parse_line(line: &str) -> Option<(i32, f64)> {
    let parts: Vec<&str> = line.split(',').collect();
    if parts.len() == 2 {
        let r = parts[0].parse::<i32>().ok()?;
        let count = parts[1].parse::<f64>().ok()?;
        Some((r, count))
    } else {
        None
    }
}

fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

fn plot_graph(data_sets: Vec<(String, Vec<(i32, f64)>)>) -> Result<(), Box<dyn std::error::Error>> {
    let root_area = BitMapBackend::new("oram_stash_plot.png", (1280, 960)).into_drawing_area();
    root_area.fill(&WHITE)?;
    let mut chart = ChartBuilder::on(&root_area)
        .caption("Stash Size Analysis", ("sans-serif", 40))
        .margin(20)
        .x_label_area_size(50)
        .y_label_area_size(60)
        .build_cartesian_2d(0..50, 0.0..20.0)?;

    chart.configure_mesh().x_desc("R").y_desc("log2(1/δ(R))").draw()?;

    for (label, data) in data_sets {
        let transformed_data: Vec<(i32, f64)> = data
            .iter()
            .map(|(r, count)| (*r, if *count > 0.0 { f64::log2(1.0 / *count) } else { 0.0 }))
            .collect();

        chart
            .draw_series(LineSeries::new(
                transformed_data.into_iter(),
                &BLUE, // Using predefined color
            ))?
            .label(label)
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &BLUE)); // Using predefined color
    }

    chart.configure_series_labels().border_style(&BLACK).draw()?;
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let configs = vec![
        ("stash_data_N1048576_Z2_B32.txt", "N=2^20, Z=2, B=32"),
        ("stash_data_N1048576_Z4_B32.txt", "N=2^20, Z=4, B=32"),
        ("stash_data_N1048576_Z6_B32.txt", "N=2^20, Z=6, B=32"),
    ];

    let mut data_sets = Vec::new();
    for (filename, label) in configs {
        let data = read_stash_data(filename);
        data_sets.push((label.to_string(), data));
    }

    plot_graph(data_sets)?;
    println!("Plot generated: oram_stash_plot.png");

    Ok(())
}
