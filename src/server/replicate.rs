use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};

fn gen_master_replid() -> String {
    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(40)
        .map(char::from)
        .collect()
}

#[inline]
fn format(key: &str, val: &str) -> String {
    [key, ":", val].concat()
}

#[derive(Debug, Clone)]
pub enum Role {
    Master,
    Slave,
}

impl Role {
    fn to_str(&self) -> &str {
        match self {
            Self::Master => "master",
            Self::Slave => "slave",
        }
    }
}

#[derive(Debug, Clone)]
pub struct ReplicaInfo {
    pub role: Role,
    pub connected_slaves: usize,
    pub master_replid: String,
    pub master_repl_offset: usize,
}

impl ReplicaInfo {
    pub fn new(
        role: Role,
        connected_slaves: usize,
        master_replid: String,
        master_repl_offset: usize,
    ) -> Self {
        Self {
            role,
            connected_slaves,
            master_replid,
            master_repl_offset,
        }
    }

    pub fn default() -> Self {
        Self::new(Role::Master, 0, gen_master_replid(), 0)
    }

    // fake it till you make it
    pub fn fake_replica() -> Self {
        Self::new(Role::Slave, 0, gen_master_replid(), 0)
    }

    pub fn replica(master_replid: String, master_repl_offset: usize) -> Self {
        Self::new(Role::Slave, 0, master_replid, master_repl_offset)
    }

    pub fn to_string(&self) -> String {
        let role = format("role", self.role.to_str());
        let master_replid = format("master_replid", &self.master_replid);
        let master_repl_offset = format("master_repl_offset", &self.master_repl_offset.to_string());
        [&role, "\r\n", &master_replid, "\r\n", &master_repl_offset].concat()
    }
}
