extern crate console_error_panic_hook;
use log::*;
use pachadb_core::*;
use pachadb_nanolog::parser::Parser;
use pachadb_nanolog::{atom, rule};
use pachadb_nanolog::{engine::*, sym};
use std::panic;
use worker::kv::KvStore;
use worker::*;

#[event(queue)]
async fn handle_event(batch: MessageBatch<Uri>, env: Env, _ctx: Context) -> Result<()> {
    panic::set_hook(Box::new(console_error_panic_hook::hook));
    wasm_logger::init(wasm_logger::Config::default());

    Ok(())
}

#[event(fetch)]
async fn handle_request(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    panic::set_hook(Box::new(console_error_panic_hook::hook));
    wasm_logger::init(wasm_logger::Config::default());

    let router = Router::new();

    router
        .post_async("/", |mut req, ctx| async move {
            let query_req: QueryReq = req.json().await?;

            let query = Parser.parse(&query_req.query).unwrap();
            info!("Executing {:?}", query);

            let scans: Vec<Scan> = query
                .body
                .iter()
                .flat_map(|atom| match &atom.relation {
                    Term::Var(_) => {
                        let entity = atom.args.get(0).unwrap();
                        let value = atom.args.get(1).unwrap();
                        match (entity, value) {
                            (Term::Var(_), Term::Sym(v)) => vec![Scan::Value(v.clone())],
                            (Term::Sym(e), Term::Var(_)) => vec![Scan::Entity(e.clone())],
                            (Term::Sym(e), Term::Sym(v)) => {
                                vec![Scan::EntityValue(e.clone(), v.clone())]
                            }
                            (Term::Var(_), Term::Var(_)) => vec![],
                        }
                    }
                    Term::Sym(f) => {
                        let entity = atom.args.get(0).unwrap();
                        let value = atom.args.get(1).unwrap();
                        match (entity, value) {
                            (Term::Var(_), Term::Sym(v)) => {
                                vec![Scan::FieldValue(f.clone(), v.clone())]
                            }
                            (Term::Sym(e), Term::Var(_)) => {
                                vec![Scan::EntityField(e.clone(), f.clone())]
                            }
                            (Term::Sym(_), Term::Sym(_)) => vec![],
                            (Term::Var(_), Term::Var(_)) => vec![Scan::Field(f.clone())],
                        }
                    }
                })
                .collect();
            info!("Performing {} scans...", scans.len());

            let mut facts = {
                let scanner = Scanner {
                    index_by_entity: ctx.env.kv("pachadb-facts-index-by-entity")?,
                    index_by_entity_field: ctx.env.kv("pachadb-facts-index-by-entity-field")?,
                    // TODO(@ostera): we don't have this one!
                    index_by_entity_value: ctx.env.kv("pachadb-facts-index-by-entity-field")?,
                    index_by_field: ctx.env.kv("pachadb-facts-index-by-field")?,
                    index_by_field_value: ctx.env.kv("pachadb-facts-index-by-field-value")?,
                    index_by_value: ctx.env.kv("pachadb-facts-index-by-value")?,
                };

                let mut facts = vec![];
                for scan in scans {
                    facts.extend(scanner.fetch(scan).await?);
                }
                facts
            };
            facts.push(query);

            let result = Solver.solve(facts);
            info!("Result {:?}", result);

            Response::from_json(&result)
        })
        .run(req, env)
        .await
}

pub enum Scan {
    Entity(String),
    EntityField(String, String),
    Field(String),
    FieldValue(String, String),
    EntityValue(String, String),
    Value(String),
}

pub struct Scanner {
    index_by_entity: KvStore,
    index_by_entity_field: KvStore,
    index_by_entity_value: KvStore,
    index_by_field: KvStore,
    index_by_field_value: KvStore,
    index_by_value: KvStore,
}

impl Scanner {
    pub async fn fetch(&self, scan: Scan) -> Result<Vec<Rule>> {
        let (kv, prefix) = match scan {
            Scan::Entity(prefix) => {
                info!("Scanning index_by_entity");
                (&self.index_by_entity, prefix)
            }
            Scan::EntityField(e, f) => {
                info!("Scanning index_by_entity_field");
                (&self.index_by_entity_field, format!("{}/{}", e, f))
            }
            Scan::Field(f) => {
                info!("Scanning index_by_field");
                (&self.index_by_field, f)
            }
            Scan::FieldValue(f, v) => {
                info!("Scanning index_by_field_value");
                (&self.index_by_field_value, format!("{}/{}", f, v))
            }
            Scan::EntityValue(e, v) => {
                info!("Scanning index_by_entity_value");
                (&self.index_by_entity_value, format!("{}/{}", e, v))
            }
            Scan::Value(v) => {
                info!("Scanning index_by_value");
                (&self.index_by_value, v)
            }
        };
        info!("Using prefix {}", &prefix);

        let iter = kv.list().prefix(prefix).execute().await?;

        let mut rules = vec![];
        for key in iter.keys {
            info!("Fetching fact {}", &key.name);
            let fact: Fact = kv.get(&key.name).json().await?.unwrap();
            let rule = rule!(
                atom!(sym!(fact.entity.0), sym!(fact.field.0), sym!(fact.value)),
                vec![]
            );
            rules.push(rule);
        }

        Ok(rules)
    }
}
