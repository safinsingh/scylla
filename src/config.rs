use crate::checks::{tcp::TcpCheck, udp::UdpCheck, CheckMeta, Service};
use chrono::{DateTime, Utc};
use core::panic;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::{
	collections::HashMap, convert::TryInto, path::PathBuf, sync::Arc,
	time::Duration,
};

pub type SharedService = Box<dyn Service>;
const FIXME: &str = "172.30";

fn get_check_inner(
	svc: &ServiceConfig,
	team: (&String, &Team),
	bx: (&String, &ScyllaBox),
) -> (Arc<String>, CheckMeta) {
	(
		Arc::new(format!(
			"{}.{}.{}:{}",
			FIXME,
			team.1.subnet,
			bx.1.host,
			svc.port.unwrap()
		)),
		CheckMeta {
			team_name: team.0.to_owned(),
			vm_name: bx.0.to_owned(),
			svc_name: svc.id.to_owned(),
		},
	)
}

pub fn service_from_config(
	svc: &ServiceConfig,
	team: (&String, &Team),
	bx: (&String, &ScyllaBox),
) -> SharedService {
	let desugared = svc.desugar();
	let (host, meta) = get_check_inner(svc, team, bx);

	match desugared.ty {
		ServiceTy::TCP => Box::new(TcpCheck::new(host, meta)),
		ServiceTy::UDP => Box::new(UdpCheck::new(host, meta)),
		_ => unimplemented!(),
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Cfg {
	pub round: String,
	#[serde(with = "date_fmt")]
	pub start: DateTime<Utc>,
	pub boxes: HashMap<String, ScyllaBox>,
	pub checks: CheckSettings,
	pub teams: HashMap<String, Team>,
	#[serde(default = "Vec::new")]
	pub injects: Vec<Inject>,
	#[serde(rename = "patchServer")]
	pub patch_server: PathBuf,
}

impl Cfg {
	pub fn get_services(&self) -> Vec<Box<dyn Service>> {
		self.boxes
			.iter()
			.flat_map(|bx| {
				self.teams.iter().flat_map(move |team| {
					bx.1.services
						.iter()
						.map(move |svc| service_from_config(svc, team, bx))
				})
			})
			.collect::<Vec<_>>()
	}
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Team {
	subnet: u8,
	pub password: String,
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
	pub interval: usize,
	pub jitter: isize,
}

impl CheckSettings {
	pub fn get_interval(&self) -> Duration {
		Duration::from_secs(
			(self.interval as isize
				+ rand::thread_rng().gen_range(-self.jitter..self.jitter))
			.try_into()
			.unwrap(),
		)
	}
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ScyllaBox {
	host: u8,
	pub services: Vec<ServiceConfig>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ServiceConfig {
	pub id: String,
	#[serde(rename = "type")]
	ty: ServiceTy,
	port: Option<u16>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub enum ServiceTy {
	#[serde(rename = "ssh")]
	SSH,
	#[serde(rename = "http")]
	HTTP,
	#[serde(rename = "tcp")]
	TCP,
	#[serde(rename = "udp")]
	UDP,
}

impl ServiceConfig {
	fn new(&self, port: u16, ty: ServiceTy) -> Self {
		Self {
			port: Some(port),
			ty,
			id: self.id.to_owned(),
		}
	}

	fn desugar(&self) -> Self {
		match (self.ty, self.port) {
			(ServiceTy::SSH, p) => self.new(p.unwrap_or(22), ServiceTy::TCP),
			(ServiceTy::HTTP, p) => self.new(p.unwrap_or(80), ServiceTy::TCP),
			| (cfg @ ServiceTy::TCP, Some(p))
			| (cfg @ ServiceTy::UDP, Some(p)) => self.new(p, cfg),
			_ => panic!("TCP or UDP-style services must specify a port!"),
		}
	}
}
