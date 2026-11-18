//! [`DataConverter`] implementation.

use std::collections::BTreeSet;
use std::fs::File;
use std::io::{BufReader, BufWriter, Write};
use std::path::Path;

use calamine::{Data, Reader, Xlsx};
use fileprism_core::{Error, Job, Output};
use rust_xlsxwriter::Workbook;
use serde_json::{Map, Value};

use crate::formats::{normalize_format, supports_pair, DataFormat};

/// Tabular / structured data converter for JSON / CSV / YAML / XLSX.
#[derive(Debug, Default, Clone, Copy)]
pub struct DataConverter;

impl DataConverter {
    pub fn new() -> Self {
        Self
    }

    pub fn supports_pair(input: &str, output: &str) -> bool {
        let Some(inp) = normalize_format(input) else {
            return false;
        };
        let Some(out) = normalize_format(output) else {
            return false;
        };
        supports_pair(inp, out)
    }

    pub fn convert_job(job: &Job) -> Result<Output, Error> {
        // JobOptions (quality / width / height / strip_metadata) are image-oriented;
        // data conversions ignore them.
        let input_fmt = normalize_format(&job.input_format).ok_or_else(|| {
            Error::Unsupported(job.input_format.clone(), job.output_format.clone())
        })?;
        let output_fmt = normalize_format(&job.output_format).ok_or_else(|| {
            Error::Unsupported(job.input_format.clone(), job.output_format.clone())
        })?;
        if !supports_pair(input_fmt, output_fmt) {
            return Err(Error::Unsupported(
                job.input_format.clone(),
                job.output_format.clone(),
            ));
        }

        let input_path = Path::new(&job.input_path);
        if !input_path.is_file() {
            return Err(Error::InputNotFound(job.input_path.clone()));
        }

        if let Some(parent) = Path::new(&job.output_path).parent() {
            if !parent.as_os_str().is_empty() && !parent.exists() {
                std::fs::create_dir_all(parent).map_err(|e| {
                    Error::Io(format!("create output dir {}: {e}", parent.display()))
                })?;
            }
        }

        match (input_fmt, output_fmt) {
            (DataFormat::Json, DataFormat::Csv) => json_to_csv(input_path, Path::new(&job.output_path))?,
            (DataFormat::Csv, DataFormat::Json) => csv_to_json(input_path, Path::new(&job.output_path))?,
            (DataFormat::Json, DataFormat::Yaml) => json_to_yaml(input_path, Path::new(&job.output_path))?,
            (DataFormat::Yaml, DataFormat::Json) => yaml_to_json(input_path, Path::new(&job.output_path))?,
            (DataFormat::Csv, DataFormat::Xlsx) => csv_to_xlsx(input_path, Path::new(&job.output_path))?,
            (DataFormat::Xlsx, DataFormat::Csv) => xlsx_to_csv(input_path, Path::new(&job.output_path))?,
            _ => {
                return Err(Error::Unsupported(
                    job.input_format.clone(),
                    job.output_format.clone(),
                ));
            }
        }

        let bytes_written = std::fs::metadata(&job.output_path)
            .map(|m| m.len())
            .map_err(|e| Error::Io(e.to_string()))?;

        Ok(Output {
            job_id: job.id.clone(),
            output_path: job.output_path.clone(),
            bytes_written,
        })
    }
}

fn read_text(path: &Path) -> Result<String, Error> {
    std::fs::read_to_string(path).map_err(|e| Error::Io(e.to_string()))
}

fn json_to_csv(input: &Path, output: &Path) -> Result<(), Error> {
    let text = read_text(input)?;
    let value: Value = serde_json::from_str(&text)
        .map_err(|e| Error::Other(format!("json parse: {e}")))?;
    let rows = json_array_of_objects(&value)?;
    write_csv_rows(output, &rows)
}

fn csv_to_json(input: &Path, output: &Path) -> Result<(), Error> {
    let rows = read_csv_rows(input)?;
    let array = Value::Array(
        rows.into_iter()
            .map(|obj| Value::Object(obj))
            .collect(),
    );
    write_json(output, &array)
}

fn json_to_yaml(input: &Path, output: &Path) -> Result<(), Error> {
    let text = read_text(input)?;
    let value: Value = serde_json::from_str(&text)
        .map_err(|e| Error::Other(format!("json parse: {e}")))?;
    let yaml = serde_yaml::to_string(&value)
        .map_err(|e| Error::Other(format!("yaml encode: {e}")))?;
    std::fs::write(output, yaml).map_err(|e| Error::Io(e.to_string()))
}

fn yaml_to_json(input: &Path, output: &Path) -> Result<(), Error> {
    let text = read_text(input)?;
    let value: Value = serde_yaml::from_str(&text)
        .map_err(|e| Error::Other(format!("yaml parse: {e}")))?;
    write_json(output, &value)
}

fn csv_to_xlsx(input: &Path, output: &Path) -> Result<(), Error> {
    let file = File::open(input).map_err(|e| Error::Io(e.to_string()))?;
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .flexible(true)
        .from_reader(BufReader::new(file));

    let mut workbook = Workbook::new();
    let worksheet = workbook
        .add_worksheet()
        .set_name("Sheet1")
        .map_err(|e| Error::Other(format!("xlsx sheet: {e}")))?;

    for (row_idx, record) in reader.records().enumerate() {
        let record = record.map_err(|e| Error::Other(format!("csv read: {e}")))?;
        let excel_row = u32::try_from(row_idx)
            .map_err(|_| Error::Other("csv row count exceeds xlsx limit".into()))?;
        for (col_idx, field) in record.iter().enumerate() {
            let excel_col = u16::try_from(col_idx)
                .map_err(|_| Error::Other("csv column count exceeds xlsx limit".into()))?;
            worksheet
                .write_string(excel_row, excel_col, field)
                .map_err(|e| Error::Other(format!("xlsx write: {e}")))?;
        }
    }

    workbook
        .save(output)
        .map_err(|e| Error::Other(format!("xlsx save: {e}")))
}

fn xlsx_to_csv(input: &Path, output: &Path) -> Result<(), Error> {
    let mut workbook: Xlsx<_> =
        calamine::open_workbook(input).map_err(|e| Error::Other(format!("xlsx open: {e}")))?;

    let sheet_name = workbook
        .sheet_names()
        .first()
        .cloned()
        .ok_or_else(|| Error::Other("xlsx has no sheets".into()))?;

    let range = workbook
        .worksheet_range(&sheet_name)
        .map_err(|e| Error::Other(format!("xlsx read sheet '{sheet_name}': {e}")))?;

    let file = File::create(output).map_err(|e| Error::Io(e.to_string()))?;
    let mut writer = csv::Writer::from_writer(BufWriter::new(file));

    // First row is the header row; emit every row including it.
    for row in range.rows() {
        let fields: Vec<String> = row.iter().map(cell_to_string).collect();
        writer
            .write_record(&fields)
            .map_err(|e| Error::Other(format!("csv write: {e}")))?;
    }
    writer
        .flush()
        .map_err(|e| Error::Other(format!("csv flush: {e}")))
}

fn json_array_of_objects(value: &Value) -> Result<Vec<Map<String, Value>>, Error> {
    let Value::Array(items) = value else {
        return Err(Error::Validation(
            "JSON→CSV requires a JSON array of objects".into(),
        ));
    };
    let mut rows = Vec::with_capacity(items.len());
    for (i, item) in items.iter().enumerate() {
        match item {
            Value::Object(map) => rows.push(map.clone()),
            _ => {
                return Err(Error::Validation(format!(
                    "JSON→CSV item at index {i} is not an object"
                )));
            }
        }
    }
    Ok(rows)
}

fn write_csv_rows(output: &Path, rows: &[Map<String, Value>]) -> Result<(), Error> {
    let mut headers: BTreeSet<String> = BTreeSet::new();
    for row in rows {
        headers.extend(row.keys().cloned());
    }
    let headers: Vec<String> = headers.into_iter().collect();

    let file = File::create(output).map_err(|e| Error::Io(e.to_string()))?;
    let mut writer = csv::Writer::from_writer(BufWriter::new(file));
    writer
        .write_record(&headers)
        .map_err(|e| Error::Other(format!("csv write header: {e}")))?;

    for row in rows {
        let record: Vec<String> = headers
            .iter()
            .map(|h| json_cell_to_string(row.get(h)))
            .collect();
        writer
            .write_record(&record)
            .map_err(|e| Error::Other(format!("csv write row: {e}")))?;
    }
    writer
        .flush()
        .map_err(|e| Error::Other(format!("csv flush: {e}")))
}

fn read_csv_rows(input: &Path) -> Result<Vec<Map<String, Value>>, Error> {
    let file = File::open(input).map_err(|e| Error::Io(e.to_string()))?;
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(true)
        .flexible(true)
        .from_reader(BufReader::new(file));

    let headers: Vec<String> = reader
        .headers()
        .map_err(|e| Error::Other(format!("csv headers: {e}")))?
        .iter()
        .map(|s| s.to_string())
        .collect();

    let mut rows = Vec::new();
    for record in reader.records() {
        let record = record.map_err(|e| Error::Other(format!("csv read: {e}")))?;
        let mut obj = Map::new();
        for (i, header) in headers.iter().enumerate() {
            let raw = record.get(i).unwrap_or("");
            obj.insert(header.clone(), csv_field_to_json(raw));
        }
        rows.push(obj);
    }
    Ok(rows)
}

fn write_json(output: &Path, value: &Value) -> Result<(), Error> {
    let file = File::create(output).map_err(|e| Error::Io(e.to_string()))?;
    let mut writer = BufWriter::new(file);
    serde_json::to_writer_pretty(&mut writer, value)
        .map_err(|e| Error::Other(format!("json encode: {e}")))?;
    writer.write_all(b"\n").map_err(|e| Error::Io(e.to_string()))?;
    writer.flush().map_err(|e| Error::Io(e.to_string()))
}

fn json_cell_to_string(value: Option<&Value>) -> String {
    match value {
        None | Some(Value::Null) => String::new(),
        Some(Value::String(s)) => s.clone(),
        Some(Value::Bool(b)) => b.to_string(),
        Some(Value::Number(n)) => n.to_string(),
        Some(other) => other.to_string(),
    }
}

fn csv_field_to_json(raw: &str) -> Value {
    if raw.is_empty() {
        return Value::String(String::new());
    }
    if let Ok(n) = raw.parse::<i64>() {
        return Value::Number(n.into());
    }
    if let Ok(n) = raw.parse::<f64>() {
        if let Some(num) = serde_json::Number::from_f64(n) {
            return Value::Number(num);
        }
    }
    if raw.eq_ignore_ascii_case("true") {
        return Value::Bool(true);
    }
    if raw.eq_ignore_ascii_case("false") {
        return Value::Bool(false);
    }
    Value::String(raw.to_string())
}

fn cell_to_string(cell: &Data) -> String {
    match cell {
        Data::Empty => String::new(),
        Data::String(s) => s.clone(),
        Data::Float(f) => {
            if f.fract() == 0.0 && *f >= i64::MIN as f64 && *f <= i64::MAX as f64 {
                format!("{}", *f as i64)
            } else {
                f.to_string()
            }
        }
        Data::Int(i) => i.to_string(),
        Data::Bool(b) => b.to_string(),
        Data::DateTime(dt) => dt.to_string(),
        Data::DateTimeIso(s) | Data::DurationIso(s) => s.clone(),
        Data::Error(e) => format!("#ERROR:{e}"),
    }
}
