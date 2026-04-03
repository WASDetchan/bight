//! Utilites for plotting data from table slices. Only minimal configuration is supported. The
//! intended usage is previewing the data.

use std::{
    error::Error,
    fmt::Debug,
    num::NonZero,
    ops::Range,
    path::{Path, PathBuf},
};

use crate::table::{Table, TableSlice};

use plotters::{coord::types::RangedCoordf64, prelude::*, style::full_palette::PURPLE};
use polyfit::{ChebyshevFit, MonomialFit, score, statistics::DegreeBound};

/// Describes how the limits of a plot axis should be calculated
/// - MinMax: the limits for the axis is lowest and highest coordinate of the data points
/// - MinMaxOrZero: the limits for the axis is lowest and highest coordinate of the data points,
///   but a bound is chanded to 0.0 if it's positive (for lower bound) or negative (for upper bound)
/// - Custom: a custom range
#[derive(Default, Clone, Debug)]
pub enum PlotLimits {
    #[default]
    MinMax,
    MinMaxOrZero,
    Custom(Range<f64>),
}

/// General options for plotting that are shared betwwen all data sets.
#[derive(Debug)]
pub struct PlotOptions {
    pub limits_x: PlotLimits,
    pub limits_y: PlotLimits,
    pub label: String,
    pub label_x: String,
    pub label_y: String,
    pub size: (u32, u32),
    /// The number of points that will be used for plotting curve or line approximations. The
    /// default is 100.
    pub approx_points: NonZero<u64>,
}

impl PlotOptions {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Default for PlotOptions {
    fn default() -> Self {
        Self {
            limits_x: PlotLimits::default(),
            limits_y: PlotLimits::default(),
            label: String::from("Data preview"),
            label_x: String::from("x values"),
            label_y: String::from("y values"),
            size: (800, 600),
            approx_points: NonZero::new(100).unwrap(),
        }
    }
}
/// Describes how the data should be plotted
#[derive(Default, Clone, Copy)]
pub enum PlotType {
    #[default]
    /// Plots each individual point
    Scatter,
    /// Plots each individual point and connects them with segments
    Segments,
    /// Plots each individual point and their linear approximation
    Linear,
    /// Plots each individual point and their curve approximation
    Curve,
}

/// Describes what should be drawn
#[derive(Default)]
pub enum DrawType {
    #[default]
    Points,
    Segments,
}

#[derive(Default)]
pub struct DataOptions {
    pub plot_type: DrawType,
}

/// The data for plotting. Can be either a dataset (a vector of points) or a function.
pub enum PlotData {
    Points(Vec<(f64, f64)>),
    Function {
        f: Box<dyn Fn(f64) -> f64>,
        range: Range<f64>,
        points: NonZero<u64>,
    },
}

impl Debug for PlotData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Points(data) => write!(f, "Points data: {data:?}"),
            Self::Function {
                f: _,
                range,
                points: _,
            } => write!(f, "Function data on range {range:?}"),
        }
    }
}

impl PlotData {
    /// Draws the data on the given chart
    pub fn plot<DB, DE>(
        &self,
        chart: &mut FloatChartContext<'_, DB>,
        plot_type: DrawType,
        style: impl Into<ShapeStyle>,
    ) -> Result<(), PlotError<DE>>
    where
        DB: DrawingBackend<ErrorType = DE>,
        DE: Error + Send + Sync,
    {
        let style = style.into();

        let point_iter = self.point_iter();

        match plot_type {
            DrawType::Points => {
                chart.draw_series(point_iter.map(|(x, y)| Circle::new((x, y), 3.0, style)))?;
            }
            DrawType::Segments => {
                chart.draw_series(LineSeries::new(point_iter, style))?;
            }
        }
        Ok(())
    }

    /// Returns an iterator over the poins that will be plotted
    pub fn point_iter(&self) -> Box<dyn Iterator<Item = (f64, f64)> + '_> {
        let iter: Box<dyn Iterator<Item = (f64, f64)> + '_> = match self {
            PlotData::Points(data) => Box::new(data.iter().copied()),

            PlotData::Function { f, range, points } => {
                let points: u64 = (*points).into();
                Box::new((0u64..points).map(move |p| {
                    let x = (range.end - range.start) / ((points - 1) as f64) * (p as f64)
                        + range.start;
                    let y = f(x);
                    (x, y)
                }))
            }
        };

        iter
    }

    pub fn data(&self) -> std::borrow::Cow<'_, [(f64, f64)]> {
        match self {
            PlotData::Points(data) => std::borrow::Cow::Borrowed(data),
            PlotData::Function {
                f: _,
                range: _,
                points: _,
            } => std::borrow::Cow::Owned(self.point_iter().collect()),
        }
    }

    /// Returns a vector of points that will be plotted
    pub fn owned_data(&self) -> Vec<(f64, f64)> {
        match self {
            PlotData::Points(data) => data.clone(),
            PlotData::Function {
                f: _,
                range: _,
                points: _,
            } => self.point_iter().collect(),
        }
    }

    /// Finds the maximum and minimum x coordinates of the points
    pub fn x_range(&self) -> Option<Range<f64>> {
        match self {
            Self::Points(data) => {
                let mut data = data.iter().copied();
                let (mut x_min, _) = data.next()?;
                let mut x_max = x_min;

                for (x, _y) in data {
                    if x < x_min {
                        x_min = x;
                    }
                    if x > x_max {
                        x_max = x;
                    }
                }
                Some(x_min..x_max)
            }
            Self::Function {
                f: _,
                range,
                points: _,
            } => Some(range.clone()),
        }
    }

    /// Changes on which range of x values the PlotData::Function is plotted. Does not adjust
    /// number of approximation points. Has no effect on PlotData::Points
    pub fn set_x_range(&mut self, range: Range<f64>) {
        if let PlotData::Function {
            f: _,
            range: self_range,
            points: _,
        } = self
        {
            *self_range = range
        }
    }

    /// Finds the maximum and minimum y coordinates of the points
    pub fn y_range(&self) -> Option<Range<f64>> {
        let mut data = self.point_iter();
        let (_, mut y_min) = data.next()?;
        let mut y_max = y_min;

        for (_x, y) in data {
            if y < y_min {
                y_min = y;
            }
            if y > y_max {
                y_max = y;
            }
        }
        Some(y_min..y_max)
    }

    pub fn from_iters(
        x: impl Iterator<Item: TryInto<f64>>,
        y: impl Iterator<Item: TryInto<f64>>,
    ) -> Self {
        let data: Vec<(f64, f64)> = x
            .zip(y)
            .map(|(x, y)| {
                (
                    x.try_into().unwrap_or_default(),
                    y.try_into().unwrap_or_default(),
                )
            })
            .collect();

        Self::Points(data)
    }

    /// Approximates the data with a linear function y = a * x + b. Returns the approximation and
    /// its coefficients (a, b).
    pub fn linear_approximation(&self, approx_points: NonZero<u64>) -> Option<(Self, f64, f64)> {
        let data = self.data();
        let range = self.x_range()?;
        let fit = MonomialFit::new(data, 1).ok()?;
        let poly = fit.into_polynomial();

        let (a, b) = (poly.coefficients()[0], poly.coefficients()[1]);

        Some((
            Self::Function {
                f: Box::new(move |x: f64| poly.y(x)),
                range,
                points: approx_points,
            },
            a,
            b,
        ))
    }

    /// Approximates the data with a Chebyshev polynomial of a variable degree. Returns the approximation and a human-readable description.
    pub fn curve_approximation(&self, approx_points: NonZero<u64>) -> Option<(Self, String)> {
        let data = self.data();
        let range = self.x_range()?;

        let fit = ChebyshevFit::new_auto(data, DegreeBound::Custom(10), &score::Aic).ok()?;
        let poly = fit.into_polynomial();

        let desc = poly.equation();
        Some((
            Self::Function {
                f: Box::new(move |x: f64| poly.y(x)),
                range,
                points: approx_points,
            },
            desc,
        ))
    }
}

/// The location and type in which the plot will be saved (currently only bitmap is supported)
pub enum PlotOutput {
    BitMapFile(PathBuf),
}

#[derive(Debug, thiserror::Error)]
pub enum PlotError<DE: Error + Send + Sync> {
    #[error("Failed to convert cells into numeric values")]
    DataConversionError,
    #[error("There was not enough data to plot with the given options")]
    SizeError,
    #[error(transparent)]
    DrawingError(#[from] DrawingAreaErrorKind<DE>),
    #[error(transparent)]
    FitError(#[from] polyfit::error::Error),
}

const PLOT_COLORS: [RGBColor; 5] = [BLUE, RED, BLACK, GREEN, PURPLE];

/// Plots each series from `data` with the corresponding (by index) plot type and the shared
/// options. If the length of
/// plot_types is less that length of data, the data without the type is not plotted. Saves the
/// plot to `output`.
pub fn plot(
    mut data: Vec<PlotData>,
    plot_types: Vec<PlotType>,
    mut options: PlotOptions,
    output: &PlotOutput,
) -> Result<Vec<String>, PlotError<impl Error + Send + Sync + use<>>> {
    let PlotOutput::BitMapFile(path) = output;

    let root = BitMapBackend::new(path, options.size).into_drawing_area();
    root.fill(&WHITE)?;
    let mut chart = prepare_chart(ChartBuilder::on(&root), &mut options, &mut data[..])?;

    let mut output = Vec::new();

    for ((data, plot_type), color) in data
        .into_iter()
        .zip(plot_types.into_iter())
        .zip(PLOT_COLORS.iter().cycle())
    {
        let style: ShapeStyle = <&RGBColor as Into<ShapeStyle>>::into(color).filled();

        match plot_type {
            PlotType::Scatter => {
                data.plot(&mut chart, DrawType::Points, style)?;
                output.push(String::new());
            }
            PlotType::Segments => {
                data.plot(&mut chart, DrawType::Points, style)?;
                data.plot(&mut chart, DrawType::Segments, style)?;
                output.push(String::new());
            }
            PlotType::Linear => {
                let (lin_data, a, b) = data
                    .linear_approximation(options.approx_points)
                    .ok_or(PlotError::SizeError)?;
                data.plot(&mut chart, DrawType::Points, style)?;
                lin_data.plot(&mut chart, DrawType::Segments, style)?;

                let mut buffer_a = ryu::Buffer::new();
                let a = buffer_a.format(a);
                let mut buffer_b = ryu::Buffer::new();
                let b = buffer_b.format(b);
                output.push(format!("y = {a} * x + {b}"));
            }
            PlotType::Curve => {
                let (curve_data, desc) = data
                    .curve_approximation(options.approx_points)
                    .ok_or(PlotError::SizeError)?;
                data.plot(&mut chart, DrawType::Points, style)?;
                curve_data.plot(&mut chart, DrawType::Segments, style)?;

                output.push(desc);
            }
        }
    }

    // To avoid the IO failure being ignored silently, we manually call the present function
    root.present()?;
    log::info!("Plot has been saved to {path:?}");

    Ok(output)
}

pub fn plot_slice_default_with_type<T, TB>(
    data: TableSlice<'_, TB>,
    plot_type: PlotType,
    file: PathBuf,
) -> Result<Vec<String>, PlotError<impl Error + use<T, TB>>>
where
    f64: TryFrom<T>,
    T: Clone,
    TB: Table<Item = T>,
{
    let data = prepare_data(data).ok_or(PlotError::SizeError)?;

    log::trace!("Plotting data: {data:?}");

    let plot_types = data.iter().map(|_| plot_type).collect();

    plot(
        data,
        plot_types,
        PlotOptions::default(),
        &PlotOutput::BitMapFile(file),
    )
}

/// Plots the data from the slice using straight segments to connect the points.
///
/// Requires a TableSlice to be at least of width 2 and height 1. Interprets the first column as x
/// values, and the rest as y values. Plots y(x) for each column of y values on the same plot. The
/// data is converted to floats with the [`TryFrom<T>`] trait. The errors of conversion are
/// ignored, and 0.0 is used for value conversion of which has failed.
pub fn plot_segments_to_file<T, U: Table<Item = T>>(
    data: TableSlice<'_, U>,
    path: &Path,
) -> Result<Vec<String>, PlotError<impl Error + use<T, U>>>
where
    f64: TryFrom<T>,
    T: Clone,
{
    plot_slice_default_with_type(data, PlotType::Segments, path.to_owned())
}

/// Plots the data from the slice, approiximating each data series with a curve
///
/// Requires a TableSlice to be at least of width 2 and height 1. Interprets the first column as x
/// values, and the rest as y values. Plots y(x) for each column of y values on the same plot. The
/// data is converted to floats with the [`TryFrom<T>`] trait. The errors of conversion are
/// ignored, and 0.0 is used for value conversion of which has failed.
pub fn plot_auto_to_file<T, U: Table<Item = T>>(
    data: TableSlice<'_, U>,
    path: &Path,
) -> Result<Vec<String>, PlotError<impl Error + use<T, U>>>
where
    f64: TryFrom<T>,
    T: Clone,
{
    plot_slice_default_with_type(data, PlotType::Curve, path.to_owned())
}

/// Plots the data from the slice, lineary approiximating each data series (y = ax + b).
///
/// Requires a TableSlice to be at least of width 2 and height 1. Interprets the first column as x
/// values, and the rest as y values. Plots y(x) for each column of y values on the same plot. The
/// data is converted to floats with the [`TryFrom<T>`] trait. The errors of conversion are
/// ignored, and 0.0 is used for value conversion of which has failed.
pub fn plot_linear_to_file<T, U: Table<Item = T>>(
    data: TableSlice<'_, U>,
    path: &Path,
) -> Result<Vec<String>, PlotError<impl Error + use<T, U>>>
where
    f64: TryFrom<T>,
    T: Clone,
{
    plot_slice_default_with_type(data, PlotType::Linear, path.to_owned())
}

fn fix_ranges(
    options: &mut PlotOptions,
    data: &mut [PlotData],
) -> Option<(Range<f64>, Range<f64>)> {
    let mut x_range = match &options.limits_x {
        PlotLimits::Custom(range) => range.clone(),
        PlotLimits::MinMax | PlotLimits::MinMaxOrZero => {
            let mut data = data.iter().map(|d| d.x_range());
            let mut x_range = data.next()??;

            for range in data {
                let range = range?;
                if range.start < x_range.start {
                    x_range.start = range.start;
                }
                if range.end > x_range.end {
                    x_range.end = range.end;
                }
            }
            x_range
        }
    };

    if matches!(options.limits_x, PlotLimits::MinMaxOrZero) {
        if x_range.start > 0.0 {
            x_range.start = 0.0
        }
        if x_range.end < 0.0 {
            x_range.end = 0.0
        }
    }

    for data in &mut *data {
        data.set_x_range(x_range.clone());
    }

    let mut y_range = match &options.limits_y {
        PlotLimits::Custom(range) => range.clone(),
        PlotLimits::MinMax | PlotLimits::MinMaxOrZero => {
            let mut data = data.iter().map(|d| d.y_range());
            let mut y_range = data.next()??;

            for range in data {
                let range = range?;
                if range.start < y_range.start {
                    y_range.start = range.start;
                }
                if range.end > y_range.end {
                    y_range.end = range.end;
                }
            }
            y_range
        }
    };

    if matches!(options.limits_y, PlotLimits::MinMaxOrZero) {
        if y_range.start > 0.0 {
            y_range.start = 0.0
        }
        if y_range.end < 0.0 {
            y_range.end = 0.0
        }
    }

    options.limits_x = PlotLimits::Custom(x_range.clone());
    options.limits_y = PlotLimits::Custom(y_range.clone());

    Some((x_range, y_range))
}

type FloatChartContext<'a, DB> = ChartContext<'a, DB, Cartesian2d<RangedCoordf64, RangedCoordf64>>;
fn prepare_chart<'a, 'b, DB, DE>(
    mut chart: ChartBuilder<'a, 'b, DB>,
    options: &mut PlotOptions,
    data: &mut [PlotData],
) -> Result<FloatChartContext<'a, DB>, PlotError<DE>>
where
    DB: DrawingBackend<ErrorType = DE>,
    DE: Error + Send + Sync,
{
    let (x_range, y_range) = fix_ranges(options, data).ok_or(PlotError::SizeError)?;

    let mut chart = chart
        .margin(10)
        .caption(options.label.clone(), ("sans-serif", 16))
        .set_label_area_size(LabelAreaPosition::Left, 60)
        .set_label_area_size(LabelAreaPosition::Bottom, 40)
        .build_cartesian_2d(x_range, y_range)?;

    chart
        .configure_mesh()
        .disable_x_mesh()
        .disable_y_mesh()
        .x_labels(10)
        .y_labels(10)
        .max_light_lines(4)
        .y_desc("y values")
        .x_desc("x values")
        .draw()?;

    Ok(chart)
}

/// Prepares f64 data for plotting from a table slice. Interprets the first column as x
/// values, and the rest as y values. Returns None if the slice is less than 2x1. The errors of conversion are
/// ignored, and 0.0 is used for value conversion of which has failed.
pub fn prepare_data<T>(data: TableSlice<'_, impl Table<Item = T>>) -> Option<Vec<PlotData>>
where
    f64: TryFrom<T>,
    T: Clone,
{
    let mut data_cols = data.cols();

    let x: Vec<f64> = data_cols
        .next()?
        .into_iter()
        .map(|x: Option<&T>| {
            x.map(|x: &T| x.clone().try_into().unwrap_or_default())
                .unwrap_or_default()
        })
        .collect();

    let data: Vec<_> = data_cols
        .map(|c| {
            PlotData::from_iters(
                x.iter().copied(),
                c.into_iter().map(|x: Option<&T>| {
                    x.map(|x: &T| -> f64 { x.clone().try_into().unwrap_or_default() })
                        .unwrap_or_default()
                }),
            )
        })
        .collect::<Vec<_>>();

    Some(data)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test_util::*;

    #[test]
    fn plot_to_png_file() -> Result<(), Box<dyn Error>> {
        let data = normal_float_data_table();
        let file = tempfile::Builder::new().suffix(".png").tempfile()?;

        plot_segments_to_file(data.full_slice(), file.path())?;

        Ok(())
    }

    #[test]
    #[ignore = "only for manual output inspenction"]
    /// Saves the plot to a file test_plot.png, which is not cleaned up and can be viewed after the test is run
    fn plot_to_real_png_file() -> Result<(), Box<dyn Error>> {
        let data = normal_float_data_table();

        let path = std::path::PathBuf::from(format!("{TEST_OUTPUT_PATH}test_plot.png"));

        dbg!(plot_segments_to_file(data.full_slice(), &path)?);

        Ok(())
    }

    #[test]
    #[ignore = "only for manual output inspenction"]
    /// Saves the plot to a file test_plot_linear.png, which is not cleaned up and can be viewed after the test is run
    fn plot_linear_to_real_png_file() -> Result<(), Box<dyn Error>> {
        let data = normal_float_data_table();

        let path = std::path::PathBuf::from(format!("{TEST_OUTPUT_PATH}test_plot_linear.png"));
        dbg!(plot_linear_to_file(data.full_slice(), &path)?);

        Ok(())
    }

    #[test]
    #[ignore = "only for manual output inspenction"]
    /// Saves the plot to a file test_plot_auto.png, which is not cleaned up and can be viewed after the test is run
    fn plot_auto_to_real_png_file() -> Result<(), Box<dyn Error>> {
        let data = normal_float_data_table();

        let path = std::path::PathBuf::from(format!("{TEST_OUTPUT_PATH}test_plot_auto.png"));
        dbg!(plot_auto_to_file(data.full_slice(), &path)?);

        Ok(())
    }
}
