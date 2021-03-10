use crate::checks::{tcp::TcpCheck, udp::UdpCheck, Service, SvcMeta};
use anyhow::{Context as _, Result};
use async_std::{
	net::{Ipv4Addr, SocketAddrV4},
	sync::Mutex,
};
use chrono::{DateTime, Utc};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf, sync::Arc, time::Duration};

const FIXME: &str = "172.30";

fn get_sock_addr(team: &Team, vm: &Vm, port: u16) -> Result<SocketAddrV4> {
	let host = format!("{}.{}.{}", FIXME, team.subnet, vm.host);

	Ok(SocketAddrV4::new(
		host.parse::<Ipv4Addr>().with_context(|| {
			format!(
				"Failed to parse team subnets and vm hosts as a valid IP \
				 address on host: {}",
				host
			)
		})?,
		port,
	))
}

#[derive(Debug)]
pub struct SharedService {
	pub inner: Box<dyn Service>,
	pub meta: Arc<SvcMeta>,
}

impl SharedService {
	pub fn from_config(
		svc: &ServiceConfig,
		(team_id, team_meta): (&String, &Team),
		(vm_id, vm_meta): (&String, &Vm),
	) -> Result<Self> {
		let inner: Box<dyn Service> = match svc.ty {
			ServiceConfigTy::Tcp { port } => Box::new(TcpCheck {
				sock: get_sock_addr(team_meta, vm_meta, port)?,
			}),
			ServiceConfigTy::Ssh { port } => Box::new(TcpCheck {
				sock: get_sock_addr(team_meta, vm_meta, port.unwrap_or(22))?,
			}),
			ServiceConfigTy::Udp { port } => Box::new(UdpCheck {
				sock: get_sock_addr(team_meta, vm_meta, port)?,
			}),
		};

		Ok(Self {
			inner,
			meta: Arc::new(SvcMeta {
				team_id: team_id.clone(),
				vm_id: vm_id.clone(),
				svc_id: svc.id.clone(),
			}),
		})
	}
}

mod date_fmt {
	// Modified from https://serde.rs/custom-date-format.html

	use chrono::{DateTime, TimeZone, Utc};
	use serde::{self, Deserialize, Deserializer, Serializer};

	const FORMAT: &'static str = "%m/%d/%Y %H:%M";
	pub fn serialize<S>(
		date: &DateTime<Utc>,
		serializer: S,
	) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		let s = format!("{}", date.format(FORMAT));
		serializer.serialize_str(&s)
	}

	pub fn deserialize<'de, D>(
		deserializer: D,
	) -> Result<DateTime<Utc>, D::Error>
	where
		D: Deserializer<'de>,
	{
		let s = String::deserialize(deserializer)?;
		Utc.datetime_from_str(&s, FORMAT)
			.map_err(serde::de::Error::custom)
	}
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Cfg {
	pub round: String,
	#[serde(with = "date_fmt")]
	pub start: DateTime<Utc>,
	pub boxes: HashMap<String, Vm>,
	pub checks: CheckSettings,
	pub teams: HashMap<String, Team>,
	#[serde(default = "Vec::new")]
	pub injects: Vec<Inject>,
	#[serde(rename = "patchServer")]
	pub patch_server: PathBuf,
	#[serde(skip)]
	pub _services: Mutex<Vec<SharedService>>,
	pub database: String,
	pub web: Web,
}

impl Cfg {
	pub fn set_services(mut self) -> Result<Self> {
		let mut __services = Vec::new();
		for vm in self.boxes.iter() {
			for team in self.teams.iter() {
				for svc in vm.1.services.iter() {
					__services.push(SharedService::from_config(svc, team, vm)?);
				}
			}
		}

		self._services = Mutex::new(__services);
		Ok(self)
	}
}

fn give_me_five() -> u8 { 5 }
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Team {
	#[serde(default = "give_me_five")]
	pub timeout: u8,
	pub subnet: u8,
	pub password: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Web {
	pub port: u16,
}

#[serde(rename_all = "camelCase")]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Inject {
	pub offset: usize,
	pub duration: usize,
	pub new_services: HashMap<String, Vec<ServiceConfig>>,
	pub meta: InjectMeta,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InjectMeta {
	pub title: String,
	pub description: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct CheckSettings {
	pub interval: u16,
	pub jitter: i16,
	pub timeout: u8,
}

impl CheckSettings {
	pub fn get_interval(&self) -> Duration {
		Duration::from_secs(
			(self.interval as i32
				+ rand::thread_rng().gen_range(-self.jitter..self.jitter)
					as i32) as u64,
		)
	}
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Vm {
	host: u8,
	pub services: Vec<ServiceConfig>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ServiceConfig {
	pub id: String,
	#[serde(flatten)]
	pub ty: ServiceConfigTy,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum ServiceConfigTy {
	Tcp { port: u16 },
	Udp { port: u16 },
	Ssh { port: Option<u16> },
}
