use crate::calculate::Calculation;
use crate::exceptions::{
    IOError,
    PlotError,
};
use chrono::prelude::*;
use plotters::prelude::*;
use std::fs;
use std::fs::DirEntry;
use std::path::{Path, PathBuf};

pub fn draw_plot() -> Result<(), PlotError> {
    // TODO `as` type coersion sucks. swap it out for something safer.
    let ts_diff: Vec<(f32, f32)> = read_data(Path::new("plots/raw_data/"))?
        .into_iter()
        .map(|(ts, calc)| (ts.timestamp() as f32, calc.data.difference as f32))
        .collect();

    let min_x = ts_diff.clone().into_iter().map(|(ts, _)| ts).reduce(f32::min).unwrap();
    let max_x = ts_diff.clone().into_iter().map(|(ts, _)| ts).reduce(f32::max).unwrap();

    let min_y = ts_diff.clone().into_iter().map(|(_, data)| data).reduce(f32::min).unwrap();
    let max_y = ts_diff.clone().into_iter().map(|(_, data)| data).reduce(f32::max).unwrap();

    // TODO debugging
    println!("data: {:?}", ts_diff);

    let root = BitMapBackend::new("plots/5.png", (1600, 1200)).into_drawing_area();
    root.fill(&WHITE)
        .or_else(|e| Err(PlotError::ChartErr(Box::new(e))))?;
    let root = root.margin(10, 10, 10, 10);
    // After this point, we should be able to draw construct a chart context
    let mut chart = ChartBuilder::on(&root)
        // Set the caption of the chart
        .caption("This is our first plot", ("sans-serif", 40).into_font())
        // Set the size of the label region
        .x_label_area_size(20)
        .y_label_area_size(40)
        // Finally attach a coordinate on the drawing area and make a chart context
        .build_cartesian_2d(min_x..max_x, min_y..max_y)
        .or_else(|e| Err(PlotError::ChartErr(Box::new(e))))?;

    // Then we can draw a mesh
    chart
        .configure_mesh()
        // We can customize the maximum number of labels allowed for each axis
        .x_labels(5)
        .y_labels(5)
        // We can also change the format of the label text
        .y_label_formatter(&|x| format!("{:.3}", x))
        .draw()
        .or_else(|e| Err(PlotError::ChartErr(Box::new(e))))?;

    // And we can draw something in the drawing area
    chart.draw_series(LineSeries::new(
        ts_diff.clone(),
        &RED,
    ))
    .or_else(|e| Err(PlotError::ChartErr(Box::new(e))))?;
    // Similarly, we can draw point series
    chart.draw_series(PointSeries::of_element(
        ts_diff.clone(),
        5,
        &RED,
        &|c, s, st| {
            return EmptyElement::at(c)    // We want to construct a composed element on-the-fly
            + Circle::new((0,0),s,st.filled()) // At this point, the new pixel coordinate is established
            + Text::new(format!("{:?}", c), (10, 0), ("sans-serif", 20).into_font());
        },
    ))
    .or_else(|e| Err(PlotError::ChartErr(Box::new(e))))?;
    Ok(())

    // let root = BitMapBackend::new("plots/0.png", (640, 480)).into_drawing_area();
    // root.fill(&WHITE)
    //     .or_else(|e| Err(PlotError::ChartErr(Box::new(e))))?;
    // let mut chart = ChartBuilder::on(&root)
    //     .caption("y=x^2", ("sans-serif", 50).into_font())
    //     .margin(5)
    //     .x_label_area_size(30)
    //     .y_label_area_size(30)
    //     .build_cartesian_2d(-1f32..1f32, -0.1f32..1f32)
    //     .or_else(|e| Err(PlotError::ChartErr(Box::new(e))))?;

    // chart.configure_mesh().draw()
    //     .or_else(|e| Err(PlotError::ChartErr(Box::new(e))))?;

    // chart
    //     .draw_series(LineSeries::new(
    //         ts_diff,
    //         &RED,
    //     ))
    //     .or_else(|e| Err(PlotError::ChartErr(Box::new(e))))?;
    //     //.legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &RED));

    // chart
    //     .configure_series_labels()
    //     .background_style(&WHITE.mix(0.8))
    //     .border_style(&BLACK)
    //     .draw()
    //     .or_else(|e| Err(PlotError::ChartErr(Box::new(e))))?;

    // Ok(())
}

fn read_data(results_directory: &Path) -> Result<Vec<(NaiveDateTime, Calculation)>, PlotError> {
    fs::read_dir(results_directory)
        .or_else(|e| Err(IOError::ReadErr(results_directory.to_path_buf(), Some(e))))
        .or_else(|e| Err(PlotError::PlotIOErr(e)))?
        .into_iter()
        .map(|entry| {
            let ent: DirEntry = entry
                .or_else(|e| Err(IOError::ReadErr(results_directory.to_path_buf(), Some(e))))
                .or_else(|e| Err(PlotError::PlotIOErr(e)))?;

            Ok(ent.path())
        })
        .collect::<Result<Vec<PathBuf>, PlotError>>()?
        .iter()
        .filter(|path| {
            path.extension()
                .and_then(|ext| ext.to_str())
                .map_or(false, |ext| ext.ends_with("json"))
        })
        .map(|p| {
            // TODO pull this filename nonsense out into a lib fn
            let filename = p.file_stem()
                .ok_or_else(|| IOError::MissingFilenameErr(p.to_path_buf()))
                .and_then(|name| {
                    name.to_str()
                        .ok_or_else(|| IOError::FilenameNotUnicodeErr(p.to_path_buf()))
                })
                .or_else(|e| Err(PlotError::PlotIOErr(e)));

            let timestamp: Result<NaiveDateTime, PlotError> = filename.and_then(|fname| {
                fname
                    .parse::<i64>()
                    // not a timestamp because it's not a number
                    .or_else(|_| Err(PlotError::FilenameNotTimestampErr(fname.to_owned())))
                    .and_then(|secs| {
                        // not a timestamp because the number is out of range
                        NaiveDateTime::from_timestamp_opt(secs, 0)
                            .ok_or_else(|| PlotError::FilenameNotTimestampErr(fname.to_owned()))
                    })
            });
            
            let x: Result<Vec<(NaiveDateTime, Calculation)>, PlotError> = timestamp.and_then(|ts| {
                fs::read_to_string(p)
                    .or_else(|e| Err(IOError::BadFileContentsErr(p.clone(), Some(e))))
                    .or_else(|e| Err(PlotError::PlotIOErr(e)))
                    .and_then(|contents| {
                        serde_json::from_str::<Vec<Calculation>>(&contents)
                            .or_else(|e| Err(PlotError::BadJSONErr(p.clone(), Some(e))))
                            .map(|calcs| {
                                calcs.iter().map(|c| (ts, c.clone())).collect()
                            })
                    })
            });
            x
        })
        .collect::<Result<Vec<Vec<(NaiveDateTime, Calculation)>>, PlotError>>()
        .map(|x| x.concat())
}
