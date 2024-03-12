#[derive(Debug)]
enum ReplicaType {
    Master,
    Slave,
}

impl ReplicaType {
    fn to_lower(&self) -> &str {
        match self {
            ReplicaType::Slave => "slave",
            ReplicaType::Master => "master",
        }
    }
}

#[derive(Debug)]
pub(super) struct Info {
    replica_type: ReplicaType,
}

impl Info {
    pub fn new() -> Self {
        Self {
            replica_type: ReplicaType::Master,
        }
    }

    fn format(key: &str, val: &str) -> String {
        format!("{}:{}", key, val)
    }

    pub fn replica(&self) -> String {
        Info::format("role", self.replica_type.to_lower())
    }
}
