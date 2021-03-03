use std::io;
use std::path;
use std::ffi::OsString;

fn main() {
    match cli() {
        Err(err) => {
            eprintln!("Error: {:?}", err);
            std::process::exit(1);
        },
        _ => {},
    }
}

#[derive(Debug)]
enum ArqError {
    Io(io::Error),
    EvaluateQuery(oxigraph::sparql::EvaluationError),
    ParseQuery(oxigraph::sparql::ParseError),
    FileNotFound(String, io::Error),
    UnknownGraphFormat,
    UnsupportedGraphFormat(OsString),
}

impl From<io::Error> for ArqError {
    fn from(err: io::Error) -> ArqError {
        ArqError::Io(err)
    }
}

impl From<oxigraph::sparql::EvaluationError> for ArqError {
    fn from(err: oxigraph::sparql::EvaluationError) -> ArqError {
        ArqError::EvaluateQuery(err)
    }
}

impl From<oxigraph::sparql::ParseError> for ArqError {
    fn from(err: oxigraph::sparql::ParseError) -> ArqError {
        ArqError::ParseQuery(err)
    }
}

fn cli() -> Result<(), ArqError> {
    use clap::{App, Arg};
    use std::path::Path;
    use std::fs;
    use std::fs::File;
    use std::io::BufReader;
    use std::str::FromStr;
    use oxigraph::MemoryStore;
    use oxigraph::model::*;
    use oxigraph::sparql::{Query, QueryResults, QueryResultsFormat};
    use arq_lib::prefix_map::PrefixMap;

    let matches = App::new("arq")
        .version("0.2.0")
        .about("SparQL command processor.")
        .author("Kristoffer Andersson")
        .arg(Arg::with_name("data")
             .long("data")
             .short("d")
             .value_name("FILE")
             .takes_value(true)
             .help("The data to work on."))
        .arg(Arg::with_name("query")
             .long("query")
             .short("q")
             .value_name("FILE")
             .takes_value(true)
             .help("The query to process.")
             .required(true))
        .get_matches();

    let store = MemoryStore::new();
    if let Some(data) = matches.value_of("data") {
        let data_path = Path::new(data);
        let f = File::open(data_path)
            .map_err(|e| ArqError::FileNotFound(String::from_str(data).unwrap(), e))?;
        let f = BufReader::new(f);
        
        store.load_graph(f, guess_graph_format(&data_path)?, &GraphName::DefaultGraph, None)?;

    }

    let mut prefix_map = PrefixMap::new();

    let query = matches.value_of("query").unwrap();

    let query_str = fs::read_to_string(query)
        .map_err(|e| ArqError::FileNotFound(String::from_str(query).unwrap(), e))?;
    let query = Query::parse(&query_str, None)?;
    for line in query_str.lines() {
        prefix_map.scan_and_add(line);
    }

    store.query(query.clone())?.write(std::io::stdout(), QueryResultsFormat::Json)?;
    println!("");
    let mut result: Vec<Vec<String>> = Vec::new();
    if let QueryResults::Solutions(solutions) = store.query(query)? {
        let mut vars = Vec::with_capacity(solutions.variables().len());
        for var in solutions.variables() {
            vars.push((var.clone()).into_string());
        }
        result.push(vars);
        for solution in solutions {
            let solution = solution?;
            let mut row = Vec::new();
            for var in &result[0] {
                let col = match solution.get(var.as_str()) {
                    Some(s) => {
                        println!("s = {:?}", s);
                        match &s {
                            Term::Literal(s) => s.value().to_string(),
                            Term::NamedNode(node) => prefix_map.replace_with_prefix(node.as_str()),
                            _ => String::new(),
                        }
                    }
                    _ => String::new(),
                };
                row.push(col);
                // row.push(String::from_str(solution?.get(var.as_str())));
                println!("{:?}", solution.get(var.as_str()));
            }
            result.push(row);
        }
    }
    println!("result = {:?}", result);
    let mut widths = Vec::new();
    for col in &result[0] {
        widths.push(col.len());
    }

    for row in &result {
        for col in 0..row.len() {
            widths[col] = widths[col].max(row[col].len());
            println!("width = {}", row[col].len());
        }
    }

    println!("widths = {:?}", widths);
    let widths_total: usize = widths.iter().sum::<usize>() + 4 + (result[0].len() - 1)*3;
    println!("{}", "-".repeat(widths_total));
    let mut row = String::new();
    for col in 0..result[0].len() {
        row = format!("{}| {}{} ", row, result[0][col], " ".repeat(widths[col]-result[0][col].len()));
    }
    println!("{}|", row);
    println!("{}", "=".repeat(widths_total));
    for row in &result[1..] {
        let mut out = String::new();
        for col in 0..row.len() {
            out = format!("{}| {}{} ", out, row[col], " ".repeat(widths[col]-row[col].len()));
        }
        println!("{}|", out);
    }
    println!("{}", "-".repeat(widths_total));
    Ok(())
}

fn guess_graph_format(path: &path::Path) -> Result<oxigraph::io::GraphFormat, ArqError> {
    use oxigraph::io::GraphFormat;

    match path.extension() {
        None => Err(ArqError::UnknownGraphFormat),
        Some(ext) => {
            if ext == GraphFormat::Turtle.file_extension() {
                return Ok(GraphFormat::Turtle);
            } else if ext == GraphFormat::NTriples.file_extension() {
                return Ok(GraphFormat::NTriples);
            } else if ext == GraphFormat::RdfXml.file_extension() {
                return Ok(GraphFormat::RdfXml);
            } else {
                return Err(ArqError::UnsupportedGraphFormat(ext.to_owned()));
            }
        }
    }
}
