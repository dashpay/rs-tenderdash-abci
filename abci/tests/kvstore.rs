mod common;

use std::{
    collections::{BTreeMap, BTreeSet},
    mem,
    path::Path,
    sync::{RwLock, RwLockWriteGuard},
};

use bincode::{Decode, Encode};
use blake2::{
    digest::{consts::U32, FixedOutput},
    Blake2b, Digest,
};
use tenderdash_abci::{error::Error, server::start_unix, Application, RequestDispatcher};
use tenderdash_proto::{
    abci::{self as proto, ExecTxResult},
    types,
};
use tracing::{debug, error};

const SOCKET: &str = "/tmp/abci.sock";
const INFO_CALLED_ERROR: &str = "info method called";

#[cfg(feature = "docker-tests")]
#[test]
fn test_kvstore() {
    use std::{fs, os::unix::prelude::PermissionsExt};

    tracing_subscriber::fmt::init();

    let kvstore = RwLock::new(KVStore::new());
    let abci_app = KVStoreABCI::new(&kvstore);

    // Alter kvstore's state to make some work for Tenderdash:
    kvstore
        .write()
        .expect("kvstore lock is poisoned")
        .pending_operations = [Operation::Insert {
        key: "ayy".to_owned(),
        value: "lmao".to_owned(),
    }]
    .into_iter()
    .collect();

    let socket = Path::new(SOCKET);
    let app = TestDispatcher::new(abci_app);
    let server = start_unix(socket, app).expect("server failed");

    let perms = fs::Permissions::from_mode(0o777);
    fs::set_permissions(socket, perms).expect("set perms");

    let socket_uri = format!("unix://{}", socket.to_str().unwrap());
    let _td = common::docker::TenderdashDocker::new("fix-docker-init", &socket_uri);

    match server.handle_connection() {
        Ok(_) => (),
        Err(e) => {
            assert!(e.to_string().contains(INFO_CALLED_ERROR));
        },
    };

    // TODO: check comitted state of kvstore
}

pub struct TestDispatcher<'a> {
    abci_app: KVStoreABCI<'a>,
}

impl<'a> TestDispatcher<'a> {
    fn new(abci_app: KVStoreABCI<'a>) -> Self {
        Self { abci_app }
    }
}

impl RequestDispatcher for TestDispatcher<'_> {
    fn handle(&self, request: proto::Request) -> Result<proto::Response, Error> {
        debug!("Incoming request: {:?}", request);
        let value = match request.value.unwrap() {
            proto::request::Value::Echo(req) => {
                proto::response::Value::Echo(self.abci_app.echo(req))
            },
            proto::request::Value::Flush(req) => {
                proto::response::Value::Flush(self.abci_app.flush(req))
            },
            proto::request::Value::Info(req) => {
                proto::response::Value::Info(self.abci_app.info(req))
            },
            proto::request::Value::InitChain(req) => {
                proto::response::Value::InitChain(self.abci_app.init_chain(req))
            },
            proto::request::Value::Query(req) => {
                proto::response::Value::Query(self.abci_app.query(req))
            },
            proto::request::Value::CheckTx(req) => {
                proto::response::Value::CheckTx(self.abci_app.check_tx(req))
            },
            proto::request::Value::OfferSnapshot(req) => {
                proto::response::Value::OfferSnapshot(self.abci_app.offer_snapshot(req))
            },
            proto::request::Value::LoadSnapshotChunk(req) => {
                proto::response::Value::LoadSnapshotChunk(
                    self.abci_app.load_snapshot_chunk(req),
                )
            },
            proto::request::Value::ApplySnapshotChunk(req) => {
                proto::response::Value::ApplySnapshotChunk(
                    self.abci_app.apply_snapshot_chunk(req),
                )
            },
            proto::request::Value::ListSnapshots(req) => {
                proto::response::Value::ListSnapshots(self.abci_app.list_snapshots(req))
            },
            proto::request::Value::PrepareProposal(req) => {
                proto::response::Value::PrepareProposal(self.abci_app.prepare_proposal(req))
            },
            proto::request::Value::ProcessProposal(req) => {
                proto::response::Value::ProcessProposal(self.abci_app.process_proposal(req))
            },
            proto::request::Value::FinalizeBlock(req) => {
                proto::response::Value::FinalizeBlock(self.abci_app.finalize_block(req));
                // Shudown ABCI application after one block
                return Err(Error::generic(INFO_CALLED_ERROR.to_string()));
            },
            proto::request::Value::ExtendVote(req) => {
                proto::response::Value::ExtendVote(self.abci_app.extend_vote(req))
            },
            proto::request::Value::VerifyVoteExtension(req) => {
                proto::response::Value::VerifyVoteExtension(
                    self.abci_app.verify_vote_extension(req),
                )
            },
        };
        debug!("Response: {:?}", value);
        Ok(proto::Response { value: Some(value) })
    }
}

/// An example storage.
///
/// For clarity it separates commited data (application data with associated
/// block height) and uncommited data. Tenderdash interaction details are
/// factored out as much as possible.
#[derive(Debug, Default)]
pub(crate) struct KVStore {
    persisted_state: BTreeMap<String, String>,
    last_block_height: u32,
    pub(crate) pending_operations: BTreeSet<Operation>,
}

impl KVStore {
    pub(crate) fn new() -> Self {
        Default::default()
    }

    pub(crate) fn commit(&mut self) {
        let pending_operations = mem::replace(&mut self.pending_operations, BTreeSet::new());
        pending_operations
            .into_iter()
            .for_each(|op| op.apply(&mut self.persisted_state));
        self.last_block_height += 1;
    }

    pub(crate) fn calculate_uncommited_state_hash(&self) -> [u8; 32] {
        let mut temp_state = self.persisted_state.clone();
        self.pending_operations
            .iter()
            .cloned()
            .for_each(|op| op.apply(&mut temp_state));

        simple_map_hash(&temp_state)
    }

    pub(crate) fn calculate_persisted_state_hash(&self) -> [u8; 32] {
        simple_map_hash(&self.persisted_state)
    }

    pub(crate) fn last_block_height(&self) -> u32 {
        self.last_block_height
    }
}

fn simple_map_hash<'a, K, V>(map: impl IntoIterator<Item = (&'a K, &'a V)>) -> [u8; 32]
where
    K: AsRef<[u8]> + 'a,
    V: AsRef<[u8]> + 'a,
{
    let mut hasher: Blake2b<U32> = Digest::new();
    map.into_iter()
        .map(|(k, v)| (k.as_ref(), v.as_ref()))
        .for_each(|(k, v)| {
            hasher.update(k.len().to_ne_bytes());
            hasher.update(k);
            hasher.update(v.len().to_ne_bytes());
            hasher.update(v);
        });
    hasher.finalize_fixed().into()
}

#[derive(Debug, Clone, Encode, Decode, Ord, PartialOrd, PartialEq, Eq)]
pub(crate) enum Operation {
    Insert { key: String, value: String },
    Delete(String),
}

impl Operation {
    fn apply(self, map: &mut BTreeMap<String, String>) {
        match self {
            Operation::Insert { key, value } => {
                map.insert(key, value);
            },
            Operation::Delete(key) => {
                map.remove(&key);
            },
        }
    }
}

#[derive(Debug)]
pub(crate) struct KVStoreABCI<'a> {
    kvstore: &'a RwLock<KVStore>,
}

impl<'a> KVStoreABCI<'a> {
    pub(crate) fn new(kvstore: &'a RwLock<KVStore>) -> Self {
        KVStoreABCI { kvstore }
    }

    fn lock_kvstore(&self) -> RwLockWriteGuard<KVStore> {
        self.kvstore.write().expect("kvstore lock is poisoned")
    }
}

impl Application for KVStoreABCI<'_> {
    fn info(&self, _request: proto::RequestInfo) -> proto::ResponseInfo {
        let kvstore_lock = self.lock_kvstore();

        proto::ResponseInfo {
            data: "kvstore-rs".to_string(),
            version: "0.1.0".to_string(),
            app_version: 1,
            last_block_height: kvstore_lock.last_block_height() as i64,
            last_block_app_hash: kvstore_lock.calculate_persisted_state_hash().to_vec(),
        }
    }

    fn init_chain(&self, _request: proto::RequestInitChain) -> proto::ResponseInitChain {
        // Do nothing special as we're working with a simple example
        proto::ResponseInitChain {
            app_hash: self
                .lock_kvstore()
                .calculate_persisted_state_hash()
                .to_vec(),
            ..Default::default()
        }
    }

    fn prepare_proposal(
        &self,
        request: proto::RequestPrepareProposal,
    ) -> proto::ResponsePrepareProposal {
        let mut kvstore_lock = self.lock_kvstore();
        // Check if the node is up to date and ready for the next block
        if request.height != (kvstore_lock.last_block_height() + 1) as i64 {
            error!(
                "Proposed block height is {} when kvstore is on {}",
                request.height,
                kvstore_lock.last_block_height()
            );
            return Default::default();
        }

        // Decode proposed transactions
        let Some(td_proposed_transactions) = request
            .txs
            .into_iter()
            .map(decode_transaction)
            .collect::<Option<BTreeSet<Operation>>>()
        else {
            error!("Cannot decode transactions");
            return Default::default();
        };

        // Mark transactions that should be added to the proposed transactions
        let pending_local_transactions = &kvstore_lock.pending_operations;
        let node_proposed_transactions =
            pending_local_transactions.difference(&td_proposed_transactions);

        let tx_records_encoded: Option<Vec<proto::TxRecord>> = td_proposed_transactions
            .iter()
            .map(|tx| {
                Some(proto::TxRecord {
                    tx: bincode::encode_to_vec(tx, bincode::config::standard()).ok()?,
                    action: proto::tx_record::TxAction::Unmodified.into(),
                })
            })
            .chain(node_proposed_transactions.map(|tx| {
                Some(proto::TxRecord {
                    tx: bincode::encode_to_vec(tx, bincode::config::standard()).ok()?,
                    action: proto::tx_record::TxAction::Added.into(),
                })
            }))
            .collect();

        let Some(tx_records) = tx_records_encoded else {
            error!("Cannot encode transactions");
            return Default::default()
        };

        // Put both local and proposed transactions into staging area
        let joined_transactions = pending_local_transactions.union(&td_proposed_transactions);

        let tx_results = tx_results_accept(joined_transactions.clone().count());

        kvstore_lock.pending_operations = joined_transactions.cloned().collect();

        proto::ResponsePrepareProposal {
            tx_records,
            tx_results,
            app_hash: kvstore_lock.calculate_uncommited_state_hash().to_vec(),
            ..Default::default()
        }
    }

    fn process_proposal(
        &self,
        request: proto::RequestProcessProposal,
    ) -> proto::ResponseProcessProposal {
        let mut kvstore_lock = self.lock_kvstore();

        // Check if the node is up to date and ready for the next block
        if request.height != (kvstore_lock.last_block_height() + 1) as i64 {
            error!(
                "Proposed block height is {} when kvstore is on {}",
                request.height,
                kvstore_lock.last_block_height()
            );
            return Default::default();
        }

        // Decode proposed transactions
        let Some(td_proposed_transactions) = request
            .txs
            .into_iter()
            .map(decode_transaction)
            .collect::<Option<BTreeSet<Operation>>>()
        else {
            error!("Cannot decode transactions");
            return Default::default();
        };

        let tx_results = tx_results_accept(td_proposed_transactions.len());

        // For simplicity just agree with proposed transactions:
        kvstore_lock.pending_operations = td_proposed_transactions;

        let app_hash = kvstore_lock.calculate_uncommited_state_hash().to_vec();

        proto::ResponseProcessProposal {
            status: proto::response_process_proposal::ProposalStatus::Accept.into(),
            tx_results: tx_results,
            app_hash,
            ..Default::default()
        }
    }

    fn extend_vote(&self, request: proto::RequestExtendVote) -> proto::ResponseExtendVote {
        // request.height
        let height = request.height.to_be_bytes().to_vec();
        proto::ResponseExtendVote {
            vote_extensions: vec![proto::ExtendVoteExtension {
                r#type: types::VoteExtensionType::ThresholdRecover as i32,
                extension: height,
            }],
        }
    }

    fn verify_vote_extension(
        &self,
        request: proto::RequestVerifyVoteExtension,
    ) -> proto::ResponseVerifyVoteExtension {
        let height = request.height.to_be_bytes().to_vec();
        let ext = request
            .vote_extensions
            .first()
            .expect("missing vote extension");

        let status = match ext.extension == height {
            true => proto::response_verify_vote_extension::VerifyStatus::Accept as i32,
            false => proto::response_verify_vote_extension::VerifyStatus::Reject as i32,
        };

        proto::ResponseVerifyVoteExtension { status }
    }

    fn finalize_block(
        &self,
        request: proto::RequestFinalizeBlock,
    ) -> proto::ResponseFinalizeBlock {
        let mut kvstore_lock = self.lock_kvstore();

        // Check if the node is up to date and ready for the next block
        if request.height != (kvstore_lock.last_block_height() + 1) as i64 {
            error!(
                "Proposed block height is {} when kvstore is on {}",
                request.height,
                kvstore_lock.last_block_height()
            );
            return Default::default();
        }

        kvstore_lock.commit();

        Default::default()
    }
}

fn decode_transaction(bytes: impl AsRef<[u8]>) -> Option<Operation> {
    bincode::decode_from_slice(bytes.as_ref(), bincode::config::standard())
        .map(|decoded| decoded.0)
        .ok()
}

fn tx_results_accept(len: usize) -> Vec<ExecTxResult> {
    let mut tx_results = Vec::<ExecTxResult>::new();

    for _ in 0..len {
        tx_results.push(proto::ExecTxResult {
            code: 0,
            ..Default::default()
        });
    }

    tx_results
}
