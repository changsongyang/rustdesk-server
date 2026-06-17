-- RustDesk Pro Server Database Initialization Script
-- Version: 1.0.0

-- Users table
CREATE TABLE IF NOT EXISTS users (
    id TEXT PRIMARY KEY NOT NULL,
    username TEXT NOT NULL UNIQUE,
    email TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    role TEXT NOT NULL DEFAULT 'viewer',
    organization_id TEXT,
    created_at TEXT NOT NULL,
    last_login TEXT,
    is_active INTEGER NOT NULL DEFAULT 1
);

CREATE INDEX IF NOT EXISTS idx_users_username ON users (username);
CREATE INDEX IF NOT EXISTS idx_users_email ON users (email);
CREATE INDEX IF NOT EXISTS idx_users_organization ON users (organization_id);

-- Devices table
CREATE TABLE IF NOT EXISTS devices (
    id TEXT PRIMARY KEY NOT NULL,
    device_id TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    hostname TEXT,
    os_type TEXT,
    os_version TEXT,
    ip_address TEXT,
    status TEXT NOT NULL DEFAULT 'unknown',
    user_id TEXT,
    organization_id TEXT,
    approved INTEGER NOT NULL DEFAULT 0,
    approved_by TEXT,
    approved_at TEXT,
    last_online TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users(id)
);

CREATE INDEX IF NOT EXISTS idx_devices_device_id ON devices (device_id);
CREATE INDEX IF NOT EXISTS idx_devices_user_id ON devices (user_id);
CREATE INDEX IF NOT EXISTS idx_devices_organization ON devices (organization_id);
CREATE INDEX IF NOT EXISTS idx_devices_status ON devices (status);
-- Composite indexes for common query patterns
CREATE INDEX IF NOT EXISTS idx_devices_org_status ON devices (organization_id, status);     
CREATE INDEX IF NOT EXISTS idx_devices_org_approved ON devices (organization_id, approved); 

-- Audit logs table
CREATE TABLE IF NOT EXISTS audit_logs (
    id TEXT PRIMARY KEY NOT NULL,
    log_type TEXT NOT NULL,
    action TEXT NOT NULL,
    user_id TEXT,
    username TEXT,
    device_id TEXT,
    device_name TEXT,
    ip_address TEXT,
    user_agent TEXT,
    details TEXT,
    created_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_audit_logs_type ON audit_logs (log_type);
CREATE INDEX IF NOT EXISTS idx_audit_logs_user_id ON audit_logs (user_id);
CREATE INDEX IF NOT EXISTS idx_audit_logs_device_id ON audit_logs (device_id);
CREATE INDEX IF NOT EXISTS idx_audit_logs_created_at ON audit_logs (created_at);
-- Composite indexes for common query patterns
CREATE INDEX IF NOT EXISTS idx_audit_logs_type_created ON audit_logs (log_type, created_at);
CREATE INDEX IF NOT EXISTS idx_audit_logs_user_created ON audit_logs (user_id, created_at); 

-- Organizations table (for multi-tenancy support)
CREATE TABLE IF NOT EXISTS organizations (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL UNIQUE,
    description TEXT,
    max_devices INTEGER DEFAULT 10,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    is_active INTEGER NOT NULL DEFAULT 1
);

-- Sessions table (for connection tracking)
CREATE TABLE IF NOT EXISTS sessions (
    id TEXT PRIMARY KEY NOT NULL,
    device_id TEXT NOT NULL,
    user_id TEXT,
    start_time TEXT NOT NULL,
    end_time TEXT,
    duration INTEGER,
    ip_address TEXT,
    status TEXT NOT NULL DEFAULT 'active',
    FOREIGN KEY (device_id) REFERENCES devices(id),
    FOREIGN KEY (user_id) REFERENCES users(id)
);

CREATE INDEX IF NOT EXISTS idx_sessions_device_id ON sessions (device_id);
CREATE INDEX IF NOT EXISTS idx_sessions_user_id ON sessions (user_id);
CREATE INDEX IF NOT EXISTS idx_sessions_start_time ON sessions (start_time);

-- License table
CREATE TABLE IF NOT EXISTS license (
    id TEXT PRIMARY KEY NOT NULL,
    license_type TEXT NOT NULL,
    valid_until TEXT NOT NULL,
    max_devices INTEGER,
    issued_at TEXT NOT NULL,
    is_trial INTEGER NOT NULL DEFAULT 0,
    is_active INTEGER NOT NULL DEFAULT 1
);

-- Insert default organization
INSERT OR IGNORE INTO organizations (id, name, description, max_devices, created_at, updated_at, is_active)
VALUES (
    'org-001',
    'Default Organization',
    'Default organization for RustDesk Pro Server',
    100,
    datetime('now'),
    datetime('now'),
    1
);
