use crate::storage::lmdb_sys::{
    Database, DatabaseOptions, EnvOptions, Environment, LmdbError, PutOptions, Transaction,
};
use log::info;
use std::sync::Arc;
use std::{fs, thread};
use tempdir::TempDir;

macro_rules! chk {
    ($stmt:expr) => {
        $stmt.unwrap_or_else(|e| panic!("{}", e.to_string()))
    };
}

#[test]
fn test_cursor_duplicate_keys() {
    let tmp_dir = chk!(TempDir::new("example"));
    if tmp_dir.path().exists() {
        chk!(fs::remove_dir_all(tmp_dir.path()));
    }
    chk!(fs::create_dir(tmp_dir.path()));

    let mut env_opt = EnvOptions::default();
    env_opt.no_sync = true;
    env_opt.max_dbs = Some(10);
    env_opt.map_size = Some(1024 * 1024 * 1024);
    env_opt.writable_mem_map = true;

    let env = Arc::new(chk!(Environment::new(
        tmp_dir.path().to_str().unwrap().to_string(),
        env_opt
    )));
    let mut tx = chk!(Transaction::begin(&env, false));

    let mut db_opt = DatabaseOptions::default();
    db_opt.allow_duplicate_keys = true;
    let db = chk!(Database::open(&env, &tx, "test".to_string(), Some(db_opt)));

    for k in 1..3 {
        for i in 'a'..'s' {
            chk!(tx.put(
                &db,
                format!("key_{}", k).as_bytes(),
                format!("val_{}", i).as_bytes(),
                PutOptions::default(),
            ));
        }
    }

    let cursor = chk!(tx.open_cursor(&db));

    let r = chk!(cursor.seek("key_100".as_bytes()));
    assert!(!r);

    let r = chk!(cursor.seek("key_1".as_bytes()));
    assert!(r);

    for i in 'a'..='z' {
        let r = chk!(cursor.read()).unwrap();
        if r.0 != "key_1".as_bytes() {
            break;
        }

        assert_eq!(r.0, "key_1".as_bytes());
        assert_eq!(r.1, format!("val_{}", i).as_bytes());
        let _r = chk!(cursor.next());
    }

    for i in 'a'..='z' {
        let r = chk!(cursor.read()).unwrap();
        assert_eq!(r.0, "key_2".as_bytes());
        assert_eq!(r.1, format!("val_{}", i).as_bytes());
        let r = chk!(cursor.next());

        if !r {
            break;
        }
    }
}

fn create_env() -> (Environment, Database) {
    let tmp_dir = chk!(TempDir::new("concurrent"));
    if tmp_dir.path().exists() {
        chk!(fs::remove_dir_all(tmp_dir.path()));
    }
    chk!(fs::create_dir(tmp_dir.path()));

    let mut env_opt = EnvOptions::default();
    env_opt.no_sync = true;
    env_opt.max_dbs = Some(10);
    env_opt.max_readers = Some(10);
    env_opt.map_size = Some(1024 * 1024 * 1024);
    env_opt.writable_mem_map = false;
    env_opt.no_sync = true;
    env_opt.no_locking = true;
    env_opt.no_thread_local_storage = true;

    let mut env = chk!(Environment::new(
        tmp_dir.path().to_str().unwrap().to_string(),
        env_opt
    ));

    let mut tx = chk!(env.tx_begin(false));
    let mut db_opt = DatabaseOptions::default();
    db_opt.allow_duplicate_keys = false;
    let db = chk!(Database::open(&env, &tx, "test".to_string(), Some(db_opt)));
    chk!(tx.commit());

    (env, db)
}

#[test]
fn test_concurrent_tx() {
    //  log4rs::init_file("./log4rs.yaml", Default::default())
    //      .unwrap_or_else(|_e| panic!("Unable to find log4rs config file"));

    let mut e1 = create_env();
    let mut e2 = create_env();

    let t1 = thread::spawn(move || -> Result<(), LmdbError> {
        for i in 1..=1_000_u64 {
            let mut tx = chk!(e1.0.tx_begin(false));
            tx.put(
                &e1.1,
                &i.to_le_bytes(),
                &i.to_le_bytes(),
                PutOptions::default(),
            )?;
            chk!(tx.commit());
            if i % 10000 == 0 {
                info!("Writer 1: {}", i)
            }
        }
        Ok(())
    });

    let t2 = thread::spawn(move || -> Result<(), LmdbError> {
        for i in 1..=1_000_u64 {
            let mut tx = chk!(e2.0.tx_begin(false));
            tx.put(
                &e2.1,
                &i.to_le_bytes(),
                &i.to_le_bytes(),
                PutOptions::default(),
            )?;
            chk!(tx.commit());
            if i % 10000 == 0 {
                info!("Writer 2: {}", i)
            }
        }
        Ok(())
    });

    let r1 = t1.join();
    assert!(r1.is_ok());
    let r2 = t2.join();
    assert!(r2.is_ok());
}