mod common;

use std::{
    collections::{BTreeMap, BTreeSet},
    mem,
    ops::Deref,
    sync::{RwLock, RwLockWriteGuard},
};

use bincode::{Decode, Encode};
use blake2::{
    digest::{consts::U32, FixedOutput},
    Blake2b, Digest,
};
use tenderdash_abci::{proto, start_server, Application, Error, RequestDispatcher};
use tracing::{debug, error};
use tracing_subscriber::filter::LevelFilter;

const SOCKET: &str = "/tmp/abci.sock";

#[cfg(feature = "docker-tests")]
#[test]
fn test_kvstore() {
    use std::{fs, os::unix::prelude::PermissionsExt};
    tracing_subscriber::fmt()
        .with_max_level(LevelFilter::DEBUG)
        .init();

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

    let mut state_reference = BTreeMap::new();
    state_reference.insert("ayy".to_owned(), "lmao".to_owned());

    let bind_address = format!("unix://{}", SOCKET);
    let app = TestDispatcher::new(abci_app);
    let server = start_server(&bind_address, app).expect("server failed");

    let perms = fs::Permissions::from_mode(0o777);
    fs::set_permissions(SOCKET, perms).expect("set perms");

    let socket_uri = bind_address.to_string();
    let _td = common::docker::TenderdashDocker::new("tenderdash", "fix-docker-init", &socket_uri);

    assert!(matches!(server.handle_connection(), Ok(())));
    drop(server);

    let kvstore_app = kvstore.into_inner().expect("kvstore lock is poisoned");
    assert_eq!(kvstore_app.persisted_state, state_reference);
    assert_eq!(kvstore_app.last_block_height, 1);
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
    fn handle(
        &self,
        request: proto::abci::Request,
    ) -> Result<Option<proto::abci::Response>, Error> {
        debug!("Incoming request: {:?}", request);
        let value = match request.value.unwrap() {
            proto::abci::request::Value::Echo(req) => {
                proto::abci::response::Value::Echo(self.abci_app.echo(req))
            },
            proto::abci::request::Value::Flush(req) => {
                proto::abci::response::Value::Flush(self.abci_app.flush(req))
            },
            proto::abci::request::Value::Info(req) => {
                proto::abci::response::Value::Info(self.abci_app.info(req))
            },
            proto::abci::request::Value::InitChain(req) => {
                proto::abci::response::Value::InitChain(self.abci_app.init_chain(req))
            },
            proto::abci::request::Value::Query(req) => {
                proto::abci::response::Value::Query(self.abci_app.query(req))
            },
            proto::abci::request::Value::CheckTx(req) => {
                proto::abci::response::Value::CheckTx(self.abci_app.check_tx(req))
            },
            proto::abci::request::Value::OfferSnapshot(req) => {
                proto::abci::response::Value::OfferSnapshot(self.abci_app.offer_snapshot(req))
            },
            proto::abci::request::Value::LoadSnapshotChunk(req) => {
                proto::abci::response::Value::LoadSnapshotChunk(
                    self.abci_app.load_snapshot_chunk(req),
                )
            },
            proto::abci::request::Value::ApplySnapshotChunk(req) => {
                proto::abci::response::Value::ApplySnapshotChunk(
                    self.abci_app.apply_snapshot_chunk(req),
                )
            },
            proto::abci::request::Value::ListSnapshots(req) => {
                proto::abci::response::Value::ListSnapshots(self.abci_app.list_snapshots(req))
            },
            proto::abci::request::Value::PrepareProposal(req) => {
                proto::abci::response::Value::PrepareProposal(self.abci_app.prepare_proposal(req))
            },
            proto::abci::request::Value::ProcessProposal(req) => {
                proto::abci::response::Value::ProcessProposal(self.abci_app.process_proposal(req))
            },
            proto::abci::request::Value::FinalizeBlock(req) => {
                proto::abci::response::Value::FinalizeBlock(self.abci_app.finalize_block(req));
                // Shudown ABCI application after one block
                return Ok(None);
            },
            proto::abci::request::Value::ExtendVote(req) => {
                proto::abci::response::Value::ExtendVote(self.abci_app.extend_vote(req))
            },
            proto::abci::request::Value::VerifyVoteExtension(req) => {
                proto::abci::response::Value::VerifyVoteExtension(
                    self.abci_app.verify_vote_extension(req),
                )
            },
        };

        debug!("Response: {:?}", value);
        Ok(Some(proto::abci::Response { value: Some(value) }))
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
    fn info(&self, _request: proto::abci::RequestInfo) -> proto::abci::ResponseInfo {
        let kvstore_lock = self.lock_kvstore();

        proto::abci::ResponseInfo {
            data: "kvstore-rs".to_string(),
            version: "0.1.0".to_string(),
            app_version: 1,
            last_block_height: kvstore_lock.last_block_height() as i64,
            last_block_app_hash: kvstore_lock.calculate_persisted_state_hash().to_vec(),
        }
    }

    fn init_chain(
        &self,
        _request: proto::abci::RequestInitChain,
    ) -> proto::abci::ResponseInitChain {
        // Do nothing special as we're working with a simple example
        proto::abci::ResponseInitChain {
            app_hash: self
                .lock_kvstore()
                .calculate_persisted_state_hash()
                .to_vec(),
            ..Default::default()
        }
    }

    fn prepare_proposal(
        &self,
        request: proto::abci::RequestPrepareProposal,
    ) -> proto::abci::ResponsePrepareProposal {
        let mut kvstore_lock = self.lock_kvstore();
        assert_block_height(request.height, &kvstore_lock);

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

        let tx_records_encoded: Option<Vec<proto::abci::TxRecord>> = td_proposed_transactions
            .iter()
            .map(|tx| {
                Some(proto::abci::TxRecord {
                    tx: bincode::encode_to_vec(tx, bincode::config::standard()).ok()?,
                    action: proto::abci::tx_record::TxAction::Unmodified.into(),
                })
            })
            .chain(node_proposed_transactions.map(|tx| {
                Some(proto::abci::TxRecord {
                    tx: bincode::encode_to_vec(tx, bincode::config::standard()).ok()?,
                    action: proto::abci::tx_record::TxAction::Added.into(),
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

        proto::abci::ResponsePrepareProposal {
            tx_records,
            tx_results,
            app_hash: kvstore_lock.calculate_uncommited_state_hash().to_vec(),
            ..Default::default()
        }
    }

    fn process_proposal(
        &self,
        request: proto::abci::RequestProcessProposal,
    ) -> proto::abci::ResponseProcessProposal {
        let mut kvstore_lock = self.lock_kvstore();

        assert_block_height(request.height, &kvstore_lock);

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

        proto::abci::ResponseProcessProposal {
            status: proto::abci::response_process_proposal::ProposalStatus::Accept.into(),
            tx_results,
            app_hash,
            ..Default::default()
        }
    }

    fn extend_vote(
        &self,
        request: proto::abci::RequestExtendVote,
    ) -> proto::abci::ResponseExtendVote {
        // request.height
        let height = request.height.to_be_bytes().to_vec();
        proto::abci::ResponseExtendVote {
            vote_extensions: vec![proto::abci::ExtendVoteExtension {
                r#type: proto::types::VoteExtensionType::ThresholdRecover as i32,
                extension: height,
            }],
        }
    }

    fn verify_vote_extension(
        &self,
        request: proto::abci::RequestVerifyVoteExtension,
    ) -> proto::abci::ResponseVerifyVoteExtension {
        let height = request.height.to_be_bytes().to_vec();
        let ext = request
            .vote_extensions
            .first()
            .expect("missing vote extension");

        let status = match ext.extension == height {
            true => proto::abci::response_verify_vote_extension::VerifyStatus::Accept as i32,
            false => proto::abci::response_verify_vote_extension::VerifyStatus::Reject as i32,
        };

        proto::abci::ResponseVerifyVoteExtension { status }
    }

    fn finalize_block(
        &self,
        request: proto::abci::RequestFinalizeBlock,
    ) -> proto::abci::ResponseFinalizeBlock {
        let mut kvstore_lock = self.lock_kvstore();

        assert_block_height(request.height, &kvstore_lock);

        kvstore_lock.commit();

        Default::default()
    }
}

fn decode_transaction(bytes: impl AsRef<[u8]>) -> Option<Operation> {
    bincode::decode_from_slice(bytes.as_ref(), bincode::config::standard())
        .map(|decoded| decoded.0)
        .ok()
}

fn tx_results_accept(len: usize) -> Vec<proto::abci::ExecTxResult> {
    let mut tx_results = Vec::<proto::abci::ExecTxResult>::new();

    for _ in 0..len {
        tx_results.push(proto::abci::ExecTxResult {
            code: 0,
            ..Default::default()
        });
    }

    tx_results
}

/// Check if the node is up to date and ready for the next block
fn assert_block_height(height: i64, kvstore: &impl Deref<Target = KVStore>) {
    if height != (kvstore.last_block_height() + 1) as i64 {
        error!(
            "Proposed block height is {} when kvstore is on {}",
            height,
            kvstore.last_block_height()
        );
        panic!("non-recoverable, aborting");
    }
}
