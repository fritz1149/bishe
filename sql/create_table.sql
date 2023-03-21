drop table if exists edge_domains;
create table if not exists edge_domains (
    id varchar(64) primary key default '',
    name varchar(64) not null default '',
    is_cloud boolean not null default false,
    root_node_id varchar(64) not null default ''
);

drop table if exists compute_nodes;
create table if not exists compute_nodes (
    id varchar(64) primary key default '',
    ip_addr varchar(64) not null default '0.0.0.0',
    slot integer not null default  0,
    edge_domain_id varchar(64) not null default '',
    father_hostname varchar(64) default '',
    node_type varchar(64) check(node_type in ('leaf', 'non-leaf', 'cloud'))
                                         not null default 'cloud'
);

drop table if exists compute_node_edges;
create table if not exists compute_node_edges (
    compute_node_id1 varchar(64),
    compute_node_id2 varchar(64),
    primary key(compute_node_id1, compute_node_id2)
);

drop table if exists net_infos;
create table if not exists net_infos (
    origin_hostname varchar(64),
    target_hostname varchar(64),
    bandwidth Double,
    delay Double,
    primary key (origin_hostname, target_hostname)
)