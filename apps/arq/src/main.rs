
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use clap::{App, Arg};
    use std::path::Path;
    use std::fs;
    use std::fs::File;
    use std::io::BufReader;
    use oxigraph::MemoryStore;
    use oxigraph::io::GraphFormat;
    use oxigraph::model::*;
    use oxigraph::sparql::{Query, QueryResultsFormat};

    let matches = App::new("arq")
        .version("0.1.0")
        .about("SparQL command processor.")
        .author("Kristoffer Andersson")
        .arg(Arg::with_name("data")
             .long("data")
             .short("d")
             .value_name("FILE")
             .takes_value(true)
             .help("The data to work on.")
             .required(true))
        .arg(Arg::with_name("query")
             .long("query")
             .short("q")
             .value_name("FILE")
             .takes_value(true)
             .help("The query to process.")
             .required(true))
        .get_matches();

    let data = matches.value_of("data").unwrap();
    let data_path = Path::new(data);

    let query = matches.value_of("query").unwrap();


    println!("Processing '{}' on '{}'", query, data_path.display());
    assert_eq!(data_path.extension().unwrap(), GraphFormat::Turtle.file_extension());

    let store = MemoryStore::new();

    let f = File::open(data_path)?;
    let f = BufReader::new(f);
    store.load_graph(f, GraphFormat::Turtle, &GraphName::DefaultGraph, None)?;

    let query_str = fs::read_to_string(query)?;
    let query = Query::parse(&query_str, None)?;

    store.query(query)?.write(std::io::stdout(), QueryResultsFormat::Json)?;

    Ok(())
}
