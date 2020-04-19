#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DcNetworkExternal {
    pub name: String
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DcNetwork {
    #[serde(skip_serializing)]
    pub name: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub external: Option<DcNetworkExternal>
}

impl DcNetwork {
    pub fn new_external(name: String) -> DcNetwork {
        let external = DcNetworkExternal{name};
        let network = DcNetwork {
            name: "default".to_owned(),
            external: Some(external)
        };
        network
    }
}

