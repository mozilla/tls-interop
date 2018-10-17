#[cfg(test)]
use super::*;
#[cfg(test)]
use std::env;
#[cfg(test)]
use std::path::Path;

// Test the flattener
#[test]
fn flatten_unittest() {
    use flatten::flatten;

    let mut mat = vec![];
    let mut list1 = vec![];
    list1.push(vec![String::from("l1.1.0")]);
    list1.push(vec![String::from("l1.2.0"), String::from("l1.2.1")]);
    mat.push(list1);
    let mut list2 = vec![];
    list2.push(vec![String::from("l2.1.0")]);
    list2.push(vec![String::from("l2.2.1")]);
    list2.push(vec![String::from("l2.2.2")]);
    mat.push(list2);

    let flat = flatten(&mat);
    assert_eq!(6, flat.len());
}

#[test]
fn test_simple_connection() {
    let configs = vec![
        ConfigType::NssLoopback,
        ConfigType::BsslServer,
        ConfigType::OsslServer,
        ConfigType::BsslClient,
        ConfigType::OsslClient,
    ];
    for config in configs {
        inner_test_simple(config);
    }
}

#[test]
fn test_all_cases() {
    let configs = vec![
        ConfigType::NssLoopback,
        ConfigType::BsslServer,
        ConfigType::OsslServer,
        ConfigType::BsslClient,
        ConfigType::OsslClient,
    ];
    for config in configs {
        inner_test_all_cases(config);
    }
}

#[cfg(test)]
fn inner_test_simple(conf_type: ConfigType) {
    let config = prepare_config(conf_type);

    let c = get_simple_test_case();

    let mut results = Results::new();
    run_test_case_meta(&mut results, &config, &c);

    assert_eq!(results.failed, 0);
}

#[cfg(test)]
fn inner_test_all_cases(conf_type: ConfigType) {
    let config = prepare_config(conf_type);

    let mut f = fs::File::open("cases.json").unwrap();
    let mut s = String::from("");
    f.read_to_string(&mut s)
        .expect("Could not read config file.");
    let cases: TestCases = json::decode(&s).unwrap();

    let mut results = Results::new();
    for c in cases.cases {
        run_test_case_meta(&mut results, &config, &c);
    }
    assert_eq!(results.failed, 0);
}

#[cfg(test)]
fn get_simple_test_case() -> TestCase {
    TestCase {
        name: String::from("Simple-Connect"),
        server_key: None,
        client_params: None,
        server_params: None,
        shared_params: None,
        client: None,
        server: None,
    }
}

#[cfg(test)]
enum ConfigType {
    NssLoopback,
    BsslServer,
    BsslClient,
    OsslServer,
    OsslClient,
}

#[cfg(test)]
fn prepare_config(conf_type: ConfigType) -> TestConfig {
    let dirs = get_shim_paths();
    let nss_shim_path = &dirs[0];
    let boring_shim_path = &dirs[1];
    let boring_runner_path = &dirs[2];
    let ossl_shim_path = &dirs[3];

    assert!(
        Path::new(nss_shim_path).exists(),
        "nss_bogo_shim not found at {}",
        nss_shim_path
    );
    match conf_type {
        ConfigType::NssLoopback => {}
        ConfigType::BsslServer | ConfigType::BsslClient => assert!(
            Path::new(boring_shim_path).exists(),
            "bssl_shim not found at {}",
            boring_shim_path
        ),
        ConfigType::OsslServer | ConfigType::OsslClient => assert!(
            Path::new(ossl_shim_path).exists(),
            "ossl_shim not found at {}",
            ossl_shim_path
        ),
    }

    TestConfig {
        client_shim: match conf_type {
            ConfigType::BsslClient => boring_shim_path.clone(),
            ConfigType::OsslClient => ossl_shim_path.clone(),
            _ => nss_shim_path.clone(),
        },
        server_shim: match conf_type {
            ConfigType::BsslServer => boring_shim_path.clone(),
            ConfigType::OsslServer => ossl_shim_path.clone(),
            _ => nss_shim_path.clone(),
        },
        rootdir: boring_runner_path.clone(),
        client_writes_first: match conf_type {
            ConfigType::BsslServer => true,
            ConfigType::OsslServer => true,
            _ => false,
        },
        force_ipv4: match conf_type {
            ConfigType::OsslServer | ConfigType::OsslClient => true,
            _ => false,
        },
        cipher_map: {
            let mut map = config::CipherMap::new();
            map.init(CIPHER_MAP_FILE);
            map
        },
    }
}

// Reads shim paths from Environment, or returns defaults
// (../dist/ and ../boringssl/ and ../openssl/).
#[cfg(test)]
fn get_shim_paths() -> Vec<String> {
    let nss_shim_path = match env::var_os("NSS_SHIM_PATH") {
        Some(val) => val.into_string().unwrap(),
        None => String::from("../dist/Debug/bin/nss_bogo_shim"),
    };
    let boring_root_dir = match env::var_os("BORING_ROOT_DIR") {
        Some(val) => val.into_string().unwrap(),
        None => String::from("../boringssl/"),
    };
    let boring_shim_path = format!("{}build/ssl/test/bssl_shim", &boring_root_dir);
    let boring_runner_path = format!("{}ssl/test/runner/", &boring_root_dir);
    let ossl_shim_path = match env::var_os("OSSL_SHIM_PATH") {
        Some(val) => val.into_string().unwrap(),
        None => String::from("../openssl/test/ossl_shim/ossl_shim"),
    };

    vec![
        nss_shim_path,
        boring_shim_path,
        boring_runner_path,
        ossl_shim_path,
    ]
}
