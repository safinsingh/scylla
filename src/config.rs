use crate::checks::{
	http::HttpCheck, tcp::TcpCheck, udp::UdpCheck, Service, SvcMeta,
};
use anyhow::{Context as _, Result};
use chrono::{DateTime, Utc};
use rand::Rng;
use serde::{
	de::{self, Visitor},
	Deserialize, Deserializer, Serialize, Serializer,
};
use std::{
	collections::HashMap,
	fmt,
	net::{Ipv4Addr, SocketAddrV4},
	path::PathBuf,
	str::FromStr,
	sync::Arc,
	time::Duration,
};
use tokio::sync::Mutex;

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
		// If there was a way to do this without cloning everywhere, I'd be open
		// to suggestions...
		let inner: Box<dyn Service> = match svc.ty {
			ServiceConfigTy::Tcp { port } => Box::new(TcpCheck {
				remote: get_sock_addr(team_meta, vm_meta, port)?,
			}),
			ServiceConfigTy::Ssh { port } => Box::new(TcpCheck {
				remote: get_sock_addr(team_meta, vm_meta, port.unwrap_or(22))?,
			}),
			ServiceConfigTy::Udp { port, bind_port } => Box::new(UdpCheck {
				remote: get_sock_addr(team_meta, vm_meta, port)?,
				socket_addr: SocketAddrV4::new(
					Ipv4Addr::new(0, 0, 0, 0),
					bind_port,
				),
			}),
			ServiceConfigTy::Http {
				port,
				ref method,
				ref content_hash,
			} => Box::new(HttpCheck {
				remote: get_sock_addr(team_meta, vm_meta, port.unwrap_or(80))?,
				method: method.0.clone(),
				ssl: false,
				content_hash: content_hash.clone(),
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

#[derive(Debug, Clone)]
pub struct HttpMethod(pub reqwest::Method);

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum ServiceConfigTy {
	Tcp {
		port: u16,
	},
	Udp {
		port: u16,
		bind_port: u16,
	},
	Ssh {
		port: Option<u16>,
	},
	Http {
		port: Option<u16>,
		method: HttpMethod,
		content_hash: Option<String>,
	},
}

impl Serialize for HttpMethod {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		serializer.serialize_str(self.0.as_str())
	}
}

struct HttpMethodVisitor;
impl<'de> Visitor<'de> for HttpMethodVisitor {
	type Value = HttpMethod;

	fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
		write!(
			formatter,
			"one of: GET, POST, PUT, DELETE, HEAD, OPTIONS, CONNECT, PATCH, \
			 TRACE"
		)
	}

	fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
	where
		E: de::Error,
	{
		Ok(HttpMethod(
			reqwest::Method::from_str(v)
				.map_err(|_| E::custom("Invalid method"))?,
		))
	}

	fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
	where
		E: de::Error,
	{
		Ok(HttpMethod(
			reqwest::Method::from_str(&v)
				.map_err(|_| E::custom("Invalid method"))?,
		))
	}

	fn visit_borrowed_bytes<E>(self, v: &'de [u8]) -> Result<Self::Value, E>
	where
		E: de::Error,
	{
		Ok(HttpMethod(
			reqwest::Method::from_bytes(v)
				.map_err(|_| E::custom("Invalid method"))?,
		))
	}

	fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E>
	where
		E: de::Error,
	{
		Ok(HttpMethod(
			reqwest::Method::from_bytes(&v)
				.map_err(|_| E::custom("Invalid method"))?,
		))
	}
}

impl<'de> Deserialize<'de> for HttpMethod {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		deserializer.deserialize_str(HttpMethodVisitor)
	}
}
