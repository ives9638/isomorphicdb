// Copyright 2020 - 2021 Alex Dukhno
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::query_engine_old::QueryEngineOld;
use native_tls::Identity;
use postgre_sql::wire_protocol::PgWireAcceptor;
use std::{
    env,
    env::VarError,
    io::{self, Read},
    net::TcpListener,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    thread,
};
use storage::Database;

mod query_engine;
mod query_engine_old;
mod session;

#[derive(Default, Clone)]
pub struct NodeEngine;

impl NodeEngine {
    pub fn start(&self, database: Database) {
        let listener = TcpListener::bind("0.0.0.0:5432").expect("create listener");

        for stream in listener.incoming() {
            match stream {
                Err(_) => break,
                Ok(socket) => {
                    let db = database.clone();
                    thread::spawn(move || -> io::Result<()> {
                        let acceptor: PgWireAcceptor<Identity> =
                            match (pfx_certificate_path(), pfx_certificate_password()) {
                                (Ok(path), Ok(pass)) => {
                                    let mut buff = vec![];
                                    let mut file = std::fs::File::open(path).unwrap();
                                    file.read_to_end(&mut buff)?;
                                    PgWireAcceptor::new(Some(Identity::from_pkcs12(&buff, &pass).unwrap()))
                                }
                                _ => PgWireAcceptor::new(None),
                            };

                        let connection = acceptor.accept(socket).unwrap();

                        let arc = Arc::new(Mutex::new(connection));
                        let mut query_engine = QueryEngineOld::new(arc.clone(), db);
                        log::debug!("ready to handle query");

                        loop {
                            let mut guard = arc.lock().unwrap();
                            let result = guard.receive();
                            drop(guard);
                            log::debug!("{:?}", result);
                            match result {
                                Err(e) => {
                                    log::error!("UNEXPECTED ERROR: {:?}", e);
                                    return Err(e);
                                }
                                Ok(Err(e)) => {
                                    log::error!("UNEXPECTED ERROR: {:?}", e);
                                    return Err(io::ErrorKind::InvalidInput.into());
                                }
                                Ok(Ok(client_request)) => match query_engine.execute(client_request) {
                                    Ok(()) => {}
                                    Err(_) => {
                                        break Ok(());
                                    }
                                },
                            }
                        }
                    });
                }
            }
        }
    }
}

fn pfx_certificate_path() -> Result<PathBuf, VarError> {
    let file = env::var("PFX_CERTIFICATE_FILE")?;
    let path = Path::new(&file);
    if path.is_absolute() {
        return Ok(path.to_path_buf());
    }

    let current_dir = env::current_dir().unwrap();
    Ok(current_dir.as_path().join(path))
}

fn pfx_certificate_password() -> Result<String, VarError> {
    env::var("PFX_CERTIFICATE_PASSWORD")
}
