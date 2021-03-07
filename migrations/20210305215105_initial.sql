CREATE TABLE teams (
	team_id VARCHAR PRIMARY KEY,
	pass VARCHAR NOT NULL
);

CREATE TABLE vms (
	vm_id VARCHAR,
	team_id VARCHAR,
	CONSTRAINT unique_vm UNIQUE (vm_id, team_id),
	CONSTRAINT fk_team_id FOREIGN KEY (team_id) REFERENCES teams(team_id) ON DELETE CASCADE
);

CREATE TABLE services (
	svc_id VARCHAR,
	vm_id VARCHAR,
	team_id VARCHAR,
	check_count INTEGER NOT NULL DEFAULT 0,
	uptime_score INTEGER NOT NULL DEFAULT 0,
	recurring_down INTEGER NOT NULL DEFAULT 0,
	sla_count INTEGER NOT NULL DEFAULT 0,
	CONSTRAINT unique_svc UNIQUE (svc_id, vm_id, team_id),
	CONSTRAINT fk_vm_team_id FOREIGN KEY (vm_id, team_id) REFERENCES vms(vm_id, team_id) ON DELETE CASCADE
);
