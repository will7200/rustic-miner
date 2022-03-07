use std::fmt::format;
use std::sync::{Arc, RwLock};
use crossbeam::channel::{bounded, Receiver, Sender};
use log::{debug, error, info};
use websocket::{ClientBuilder, OwnedMessage, WebSocketResult};
use websocket::futures::future::err;
use websocket::native_tls::TlsConnector;
use derohe::block;
use derohe::pow::astrobwt;
use derohe::rpc::daemon_rpc;
use derohe::rpc::daemon_rpc::GetBlockTemplateResult;
use crate::{AtomicCell, ShardedLock};

pub mod address {
    use log::debug;

    use derohe::rpc;

    pub fn validate(s: &str) -> Result<(), String> {
        match rpc::address::Address::from_string(s) {
            Ok(_) => Ok(()),
            Err(e) => return Err(format!("{}{}", "invalid address, reason: ", e))
        }
    }
}

pub mod difficulty {
    pub fn check_pow_hash(pow_hash: [u8; 32], difficulty: u64) -> bool {
        let big_pow_hash = num_bigint::BigInt::from_bytes_le(num_bigint::Sign::Plus, &pow_hash);
        let big_difficultly = num_bigint::BigInt::from(difficulty);
        if big_pow_hash <= big_difficultly {
            return true;
        }
        return false;
    }
}

#[derive(Debug)]
pub struct Job {
    pub(crate) ijob: daemon_rpc::GetBlockTemplateResult,
    pub(crate) count: u64,
}

#[derive(Debug)]
pub struct Miner {
    id: usize,
    job: Arc<RwLock<Job>>,
    counter: Arc<AtomicCell<i64>>,
}

impl Miner {
    pub fn new(id: usize, job: Arc<RwLock<Job>>, counter: Arc<AtomicCell<i64>>) -> Miner {
        Miner {
            id,
            job,
            counter,
        }
    }

    pub fn start(self) {
        let mut local_job: daemon_rpc::GetBlockTemplateResult;
        let mut local_job_count: u64;
        let mut real_job_count: u64;
        let mut work: [u8; block::MINIBLOCK_SIZE] = [(0 as u8); block::MINIBLOCK_SIZE];
        let mut diff: u64;
        let mut i: u32 = 0;
        loop {
            {
                let job = self.job.read().unwrap();
                local_job = job.ijob.clone();
                local_job_count = job.count;
                real_job_count = job.count;
            }
            if local_job_count == 0 {
                continue;
            }
            // info!("thread {}, {}", self.id, local_job.JobID);


            let val = hex::decode(local_job.Blockhashing_blob.clone());
            let n = match val.as_ref() {
                Ok(val) => {
                    if val.len() != block::MINIBLOCK_SIZE {
                        error!("unable to decode blockwork {}", local_job.Blockhashing_blob);
                        continue;
                    }
                    val
                }
                Err(e) => {
                    error!("unable to decode blockwork {}", local_job.Blockhashing_blob);
                    continue;
                }
            };
            for i in 0..block::MINIBLOCK_SIZE {
                work[i] = n[i]
            }
            work[block::MINIBLOCK_SIZE - 1] = (self.id as u8);
            diff = local_job.Difficultyuint64;
            if work[0] & 0xf != 1 {
                error!("Unknown version, please check for updates, version={}", work[0]&0xf);
                continue;
            }

            while real_job_count == local_job_count {
                i = i + 1;
                work[block::MINIBLOCK_SIZE - 5] = (i >> 24) as u8;
                work[block::MINIBLOCK_SIZE - 4] = (i >> 16) as u8;
                work[block::MINIBLOCK_SIZE - 3] = (i >> 8) as u8;
                work[block::MINIBLOCK_SIZE - 2] = (i) as u8;

                let powhash = astrobwt::pow16(work.as_ref());
                self.counter.fetch_add(1);
                if difficulty::check_pow_hash(powhash, diff) == true {
                    info!("Succecssfully found DERO Miniblock, difficulty={}, height={}", diff, local_job.Height);
                    // TODO: submit job
                }
                {
                    let job = self.job.read().unwrap();
                    real_job_count = job.count
                }
            }
            // info!("thread {}, next job", self.id)
        }
    }
}

pub enum MinerError {
    WebSocketError(String)
}

#[derive(Debug)]
pub struct WorkGatherer {
    wallet_address: String,
    daemon_rpc_address: String,
    sender: Sender<MinerError>,
    receiver: Receiver<MinerError>,
    job: Arc<RwLock<Job>>,
}

impl WorkGatherer {
    pub fn new(wallet_address: String, daemon_rpc_address: String, job: Arc<RwLock<Job>>) -> Self {
        let (sender, receiver) = bounded(10);
        WorkGatherer {
            wallet_address,
            daemon_rpc_address,
            sender,
            receiver,
            job,
        }
    }

    pub fn receiver(&self) -> Receiver<MinerError> {
        self.receiver.clone()
    }

    pub fn get_work(self) {
        let address = format!("wss://{}/ws/{}", self.daemon_rpc_address, self.wallet_address);
        info!("Connecting to {}", address);
        let mut client_b = ClientBuilder::new(&*address)
            .unwrap();
        let response_client_connect = client_b.connect_secure(Option::from(TlsConnector::builder().danger_accept_invalid_certs(true).build().unwrap()));

        let mut client = match response_client_connect {
            Ok(client) => client,
            Err(e) => {
                self.sender.send(MinerError::WebSocketError(format!("{}", e)));
                return;
            }
        };

        loop {
            let response = client.recv_message();
            let message = match response {
                Ok(o) => match o {
                    OwnedMessage::Text(val) => val,
                    _ => {
                        self.sender.send(MinerError::WebSocketError(String::from("Received wrong type")));
                        return;
                    }
                },
                Err(e) => {
                    self.sender.send(MinerError::WebSocketError(format!("{}", e)));
                    return;
                }
            };
            let job: daemon_rpc::GetBlockTemplateResult = serde_json::from_str(&*message).unwrap();
            debug!("{:#?}", job);
            {
                let mut njob = self.job.write().unwrap();
                *njob = Job {
                    ijob: job,
                    count: njob.count + 1,
                }
            }
        }
    }
}