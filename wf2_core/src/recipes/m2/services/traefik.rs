use crate::context::Context;
use crate::dc_service::DcService;
use crate::recipes::m2::m2_vars::{M2Var, M2Vars};
use crate::recipes::m2::services::M2Service;

pub struct TraefikServiceV2;

impl TraefikServiceV2 {
    pub fn route_to_svc(name: String,domain:String,tls: bool, port: u8) -> Vec<String> {        
        let service_name = format!("{}_svc",name);
        let val = vec![
            format!("traefik.http.routers.{}.rule=Host(`{}`)", name, domain),
            format!("traefik.http.routers.{}.service={}",name,service_name.to_owned()).to_string(),
            format!("traefik.http.routers.{}.tls={}",name,tls),
            format!("traefik.http.services.{}.loadBalancer.server.port={}",service_name,port),
            "traefik.enable=true".to_owned()
        ];
        val
    }
}


pub struct TraefikService;

impl TraefikService {
    pub fn host_entry_label(domain: impl Into<String>, port: impl Into<u32>) -> Vec<String> {
        vec![
            TraefikService::host(domain.into()),
            TraefikService::port(port.into()),
        ]
    }
    pub fn host_only_entry_label(domain: impl Into<String>) -> Vec<String> {
        vec![TraefikService::host(domain.into())]
    }

    fn host(domain: String) -> String {
        format!("traefik.frontend.rule=Host:{}", domain)
    }

    fn port(port: u32) -> String {
        format!("traefik.port={}", port)
    }
}

impl M2Service for TraefikService {
    const NAME: &'static str = "traefik";
    const IMAGE: &'static str = "traefik:1.7";

    fn dc_service(&self, ctx: &Context, vars: &M2Vars) -> DcService {
        DcService::new(ctx.name(), Self::NAME, Self::IMAGE)
            .set_volumes(vec![
                "/var/run/docker.sock:/var/run/docker.sock".to_string(),
                format!(
                    "{}:/etc/traefik/traefik.toml",
                    vars.content[&M2Var::TraefikFile]
                ),
            ])
            .set_ports(vec!["80:80", "443:443", "8080:8080"])
            .set_command("--api --docker")
            .set_labels(vec![Self::TRAEFIK_DISABLE_LABEL])
            .build()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_host_entry() {
        let labels = TraefikService::host_entry_label("mail.jh", 8080_u32);
        assert_eq!(
            labels,
            vec!["traefik.frontend.rule=Host:mail.jh", "traefik.port=8080"]
        )
    }
    #[test]
    fn test_host_only_entry() {
        let labels = TraefikService::host_only_entry_label("mail.jh");
        assert_eq!(labels, vec!["traefik.frontend.rule=Host:mail.jh"])
    }
}
