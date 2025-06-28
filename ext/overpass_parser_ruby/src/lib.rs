use magnus::{
    eval, function, method, prelude::*, DataTypeFunctions, Error, ExceptionClass, TypedData,
};
use overpass_parser_rust::{
    overpass_parser::{self, request::Request},
    sql_dialect,
};

fn parse(query: String) -> Result<RequestWrapper, magnus::Error> {
    match overpass_parser::parse_query(query.as_str()) {
        Ok(request) => Ok(RequestWrapper::new(request)),
        Err(e) => {
            let error_class: ExceptionClass = eval("OverpassParserRuby::ParsingError").unwrap();
            Err(Error::new(
                error_class,
                format!("Failed to parse query: {}", e),
            ))
        }
    }
}

#[derive(TypedData)]
#[magnus(class = "OverpassParserRuby::Request", free_immediately, size)]
struct RequestWrapper {
    inner: Request,
}

impl DataTypeFunctions for RequestWrapper {}

impl RequestWrapper {
    fn new(request: Request) -> Self {
        Self { inner: request }
    }

    fn to_sql(&self, dialect: String, srid: String) -> Result<String, magnus::Error> {
        let sql_dialect: Box<dyn sql_dialect::sql_dialect::SqlDialect> = match dialect.as_str() {
            "postgres" => Box::new(sql_dialect::postgres::postgres::Postgres::default()),
            "duckdb" => Box::new(sql_dialect::duckdb::duckdb::Duckdb),
            _ => {
                return Err(magnus::Error::new(
                    magnus::exception::runtime_error(),
                    "Unsupported SQL dialect".to_string(),
                ));
            }
        };
        Ok(self.inner.to_sql(&sql_dialect, srid.as_str(), None))
    }
}

fn init() {
    let module = magnus::define_module("OverpassParserRuby").unwrap();

    module
        .define_singleton_method("parse", function!(parse, 1))
        .unwrap();

    let request_class = module
        .define_class("Request", magnus::class::object())
        .unwrap();
    request_class
        .define_method("to_sql", method!(RequestWrapper::to_sql, 2))
        .unwrap();

    let runtime_error_class = eval("RuntimeError").unwrap();
    module
        .define_class("ParsingError", runtime_error_class)
        .unwrap();
}

#[no_mangle]
pub extern "C" fn Init_overpass_parser_ruby() {
    init()
}

#[no_mangle]
pub extern "C" fn Init_liboverpass_parser_ruby() {
    init()
}
