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
}

impl ReplicaInfo {
    pub fn new(role: Role, connected_slaves: usize) -> Self {
        Self {
            role,
            connected_slaves,
        }
    }

    pub fn default() -> Self {
        Self::new(Role::Master, 0)
    }

    pub fn replica() -> Self {
        Self::new(Role::Slave, 0)
    }

    #[inline]
    fn format(key: &str, val: &str) -> String {
        [key, ":", val].concat()
    }

    pub fn to_string(&self) -> String {
        ReplicaInfo::format("role", self.role.to_str())
    }
}
