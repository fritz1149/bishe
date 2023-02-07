create table if not exists edge_domains (
    id varchar(64) primary key default '',
    name varchar(64) not null default '',
    is_cloud boolean not null default false
);

create table if not exists compute_nodes (
    id varchar(64) primary key default '',
    ip_addr varchar(64) not null default '0.0.0.0',
    slot integer not null default  0,
    edge_domain_id varchar(64) not null default ''
);

create table if not exists compute_node_edges (
    compute_node_id1 varchar(64),
    compute_node_id2 varchar(64),
    primary key(compute_node_id1, compute_node_id2)
)