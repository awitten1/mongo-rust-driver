#[cfg(not(feature = "sync"))]
mod atlas_connectivity;
mod auth_aws;
mod client;
mod coll;
mod cursor;
mod db;
#[cfg(not(feature = "sync"))]
mod documentation_examples;
mod index_management;
mod spec;
mod util;

pub(crate) use self::{
    spec::{run_single_test, run_spec_test, run_spec_test_with_path, RunOn, Serverless, Topology},
    util::{
        assert_matches,
        eq_matches,
        CmapEvent,
        CommandEvent,
        Event,
        EventClient,
        EventHandler,
        FailCommandOptions,
        FailPoint,
        FailPointMode,
        MatchErrExt,
        Matchable,
        SdamEvent,
        TestClient,
    },
};

use home::home_dir;
use lazy_static::lazy_static;

use self::util::TestLock;
use crate::{
    client::options::{ServerApi, ServerApiVersion},
    options::ClientOptions,
};
use std::{fs::read_to_string, str::FromStr};

const MAX_POOL_SIZE: u32 = 100;

lazy_static! {
    pub(crate) static ref CLIENT_OPTIONS: ClientOptions = client_options_for_uri(&DEFAULT_URI);
    pub(crate) static ref LOCK: TestLock = TestLock::new();
    pub(crate) static ref DEFAULT_URI: String = get_default_uri();
    pub(crate) static ref SERVER_API: Option<ServerApi> = match std::env::var("MONGODB_API_VERSION")
    {
        Ok(server_api_version) if !server_api_version.is_empty() => Some(ServerApi {
            version: ServerApiVersion::from_str(server_api_version.as_str()).unwrap(),
            deprecation_errors: None,
            strict: None,
        }),
        _ => None,
    };
    pub(crate) static ref SERVERLESS: bool =
        matches!(std::env::var("SERVERLESS"), Ok(s) if s == "serverless");
    pub(crate) static ref LOAD_BALANCED_SINGLE_URI: Option<String> =
        std::env::var("SINGLE_MONGOS_LB_URI").ok();
    pub(crate) static ref LOAD_BALANCED_MULTIPLE_URI: Option<String> =
        std::env::var("MULTI_MONGOS_LB_URI").ok();
}

pub(crate) fn client_options_for_uri(uri: &str) -> ClientOptions {
    let mut options = ClientOptions::parse_without_srv_resolution(uri).unwrap();
    options.max_pool_size = Some(MAX_POOL_SIZE);
    options.server_api = SERVER_API.clone();
    if LOAD_BALANCED_SINGLE_URI
        .as_ref()
        .map_or(false, |uri| !uri.is_empty())
    {
        options.test_options_mut().mock_service_id = true;
    }

    options
}

fn get_default_uri() -> String {
    if let Some(uri) = LOAD_BALANCED_SINGLE_URI.clone() {
        if !uri.is_empty() {
            return uri;
        }
    }
    if let Ok(uri) = std::env::var("MONGODB_URI") {
        return uri;
    }
    if let Some(mut home) = home_dir() {
        home.push(".mongodb_uri");
        if let Ok(uri) = read_to_string(home) {
            return uri;
        }
    }
    "mongodb://localhost:27017".to_string()
}
