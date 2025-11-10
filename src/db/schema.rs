// FalkorDB Schema definitions
pub const CREATE_SCHEMA: &str = r#"
// Create constraints and indexes
CREATE CONSTRAINT ON (u:URL) ASSERT u.url IS UNIQUE;
CREATE CONSTRAINT ON (d:Domain) ASSERT d.name IS UNIQUE;
CREATE CONSTRAINT ON (b:Browser) ASSERT b.name IS UNIQUE;

CREATE INDEX ON :URL(visit_time);
CREATE INDEX ON :URL(visit_count);
CREATE INDEX ON :Domain(name);
CREATE INDEX ON :Browser(name);
"#;

// Schema documentation
pub const SCHEMA_DOC: &str = r#"
Graph Schema for Browser History:

Nodes:
- URL: Represents a visited URL
  Properties: url, title, visit_time, visit_count, last_visit

- Domain: Represents a website domain
  Properties: name, first_visit, last_visit, total_visits

- Browser: Represents a browser application
  Properties: name, profile

- Session: Represents a browsing session
  Properties: id, start_time, end_time, browser

Relationships:
- (URL)-[:BELONGS_TO]->(Domain)
- (URL)-[:VISITED_WITH]->(Browser)
- (URL)-[:FOLLOWED_BY]->(URL) - temporal sequence
- (URL)-[:IN_SESSION]->(Session)
- (Session)-[:USES]->(Browser)
"#;