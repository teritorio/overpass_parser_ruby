use std::{cell::RefCell, collections::HashMap};

use magnus::{
    block::Proc, eval, function, method, prelude::*, r_hash::ForEach, DataTypeFunctions, Error,
    ExceptionClass, IntoValue, RArray, RHash, TypedData, Value,
};
use overpass_parser_rust::{
    overpass_parser::{
        self,
        request::Request,
        selectors::Selectors,
        subrequest::{QueryType, SubrequestType},
    },
    sql_dialect,
};

fn parse(query: String) -> Result<RequestWrapper, magnus::Error> {
    match overpass_parser::parse_query(query.as_str()) {
        Ok(request) => Ok(RequestWrapper::new(request)),
        Err(e) => {
            let error_class: ExceptionClass = eval("OverpassParserRuby::ParsingError").unwrap();
            Err(Error::new(
                error_class,
                format!("Failed to parse query: {e}"),
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

thread_local! {
    static RUBY_PROCS: RefCell<HashMap<u64, Proc>> = RefCell::new(HashMap::new());
}

static PROC_ID_COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

fn build_postgres_dialect(quote: Option<Proc>) -> sql_dialect::postgres::postgres::Postgres {
    // Store the proc in thread-local storage
    let proc_id = PROC_ID_COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    if quote.is_some() {
        RUBY_PROCS.with(|procs| {
            procs.borrow_mut().insert(proc_id, quote.unwrap());
        });
    }

    sql_dialect::postgres::postgres::Postgres {
        postgres_escape_literal: match quote {
            Some(_) => Some(Box::new(move |str| {
                RUBY_PROCS.with(|procs| {
                    if let Some(proc) = procs.borrow().get(&proc_id) {
                        proc.call::<(&str,), String>((str,)).unwrap()
                    } else {
                        panic!("Quote proc is None")
                    }
                })
            })),
            None => None,
        },
        ..sql_dialect::postgres::postgres::Postgres::default()
    }
}

impl RequestWrapper {
    fn new(request: Request) -> Self {
        Self { inner: request }
    }

    fn to_sql(
        &self,
        dialect: String,
        srid: u32,
        quote: Option<Proc>,
    ) -> Result<Vec<String>, magnus::Error> {
        let sql_dialect: &(dyn sql_dialect::sql_dialect::SqlDialect) = match dialect.as_str() {
            "postgres" => &build_postgres_dialect(quote),
            "duckdb" => &sql_dialect::duckdb::duckdb::Duckdb,
            _ => {
                return Err(magnus::Error::new(
                    magnus::exception::runtime_error(),
                    "Unsupported SQL dialect".to_string(),
                ));
            }
        };
        Ok(self
            .inner
            .to_sql(sql_dialect, srid.to_string().as_str(), None))
    }

    fn all_selectors_inner(&self, query_type: &QueryType) -> Result<Vec<Selectors>, magnus::Error> {
        match query_type {
            QueryType::QueryObjects(query_objects) => Ok(vec![query_objects.selectors.clone()]),
            QueryType::QueryUnion(query_union) => Ok(query_union
                .queries
                .iter()
                .map(|subquery| self.all_selectors_inner(subquery).ok().unwrap())
                .flatten()
                .collect()),
            QueryType::QueryRecurse(_) => Ok(vec![]),
        }
    }

    fn all_selectors(&self) -> Result<RArray, magnus::Error> {
        let r = RArray::new();
        self.inner
            .subrequest
            .queries
            .iter()
            .filter_map(|query| {
                if let SubrequestType::QueryType(ref query_type) = **query {
                    self.all_selectors_inner(query_type).ok()
                } else {
                    None
                }
            })
            .flatten()
            .filter(|selectors| !selectors.selectors.is_empty())
            .for_each(|selectors| {
                let _ = r.push(SelectorsWrapper::new(selectors).into_value());
            });
        Ok(r)
    }
}

#[derive(TypedData)]
#[magnus(class = "OverpassParserRuby::Selectors", free_immediately, size)]
struct SelectorsWrapper {
    inner: Selectors,
}

impl DataTypeFunctions for SelectorsWrapper {}

impl SelectorsWrapper {
    fn new(selectors: Selectors) -> Self {
        Self { inner: selectors }
    }

    fn matches(&self, rtags: RHash) -> Result<Option<Vec<&str>>, magnus::Error> {
        let mut owned_pairs: Vec<(String, String)> = Vec::new();
        rtags.foreach(|key: Value, value: Value| {
            let key_str = key.to_string().clone();
            let value_str = value.to_string();
            owned_pairs.push((key_str, value_str));
            Ok(ForEach::Continue)
        })?;

        let tags = owned_pairs
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect();

        Ok(self.inner.matches(&tags))
    }

    fn keys(&self) -> Result<Option<Vec<&str>>, magnus::Error> {
        Ok(self
            .inner
            .selectors
            .iter()
            .filter(|selector| !selector.not)
            .map(|selector| selector.key.as_ref())
            .collect::<Vec<&str>>()
            .into())
    }

    fn to_sql(
        &self,
        dialect: String,
        table: String,
        srid: u32,
        quote: Option<Proc>,
    ) -> Result<String, magnus::Error> {
        let sql_dialect: &(dyn sql_dialect::sql_dialect::SqlDialect) = match dialect.as_str() {
            "postgres" => &build_postgres_dialect(quote),
            "duckdb" => &sql_dialect::duckdb::duckdb::Duckdb,
            _ => {
                return Err(magnus::Error::new(
                    magnus::exception::runtime_error(),
                    "Unsupported SQL dialect".to_string(),
                ));
            }
        };
        Ok(self.inner.to_sql(sql_dialect, table.as_str(), srid.to_string().as_str()))
    }

    fn to_overpass(&self) -> Result<String, magnus::Error> {
        Ok(self.inner.to_overpass())
    }
}

fn init() {
    let module = magnus::define_module("OverpassParserRuby").unwrap();

    module
        .define_singleton_method("parse", function!(parse, 1))
        .unwrap();

    let selectors_class = module
        .define_class("Selectors", magnus::class::object())
        .unwrap();
    selectors_class
        .define_method("matches", method!(SelectorsWrapper::matches, 1))
        .unwrap();
    selectors_class
        .define_method("keys", method!(SelectorsWrapper::keys, 0))
        .unwrap();
    selectors_class
        .define_method("to_sql", method!(SelectorsWrapper::to_sql, 4))
        .unwrap();
    selectors_class
        .define_method("to_overpass", method!(SelectorsWrapper::to_overpass, 0))
        .unwrap();

    let request_class = module
        .define_class("Request", magnus::class::object())
        .unwrap();
    request_class
        .define_method("to_sql", method!(RequestWrapper::to_sql, 3))
        .unwrap();
    request_class
        .define_method("all_selectors", method!(RequestWrapper::all_selectors, 0))
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
