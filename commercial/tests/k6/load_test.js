import http from 'k6/http';
import { check, sleep } from 'k6';
import { Rate, Trend } from 'k6/metrics';

// Custom metrics
const errorRate = new Rate('errors');
const loginDuration = new Trend('login_duration');
const apiDuration = new Trend('api_duration');

// Test configuration
export const options = {
  stages: [
    { duration: '30s', target: 10 },   // Ramp up to 10 users
    { duration: '1m', target: 10 },    // Stay at 10 users
    { duration: '30s', target: 50 },   // Ramp up to 50 users
    { duration: '2m', target: 50 },    // Stay at 50 users
    { duration: '30s', target: 100 },  // Ramp up to 100 users
    { duration: '2m', target: 100 },   // Stay at 100 users
    { duration: '1m', target: 0 },     // Ramp down
  ],
  thresholds: {
    http_req_duration: ['p(95)<500', 'p(99)<1000'],
    http_req_failed: ['rate<0.01'],
    errors: ['rate<0.05'],
  },
};

const BASE_URL = __ENV.BASE_URL || 'http://localhost:8080';
const USERNAME = __ENV.USERNAME || 'admin';
const PASSWORD = __ENV.PASSWORD || 'admin123';

let authToken = '';

export function setup() {
  // Login to get token
  const loginRes = http.post(`${BASE_URL}/api/auth/login`, JSON.stringify({
    username: USERNAME,
    password: PASSWORD,
  }), {
    headers: { 'Content-Type': 'application/json' },
  });

  if (loginRes.status === 200) {
    const body = JSON.parse(loginRes.body);
    return { token: body.token };
  }
  return { token: '' };
}

export default function(data) {
  const headers = {
    'Content-Type': 'application/json',
    'Authorization': `Bearer ${data.token}`,
  };

  // Test health endpoint
  const healthRes = http.get(`${BASE_URL}/health`);
  check(healthRes, {
    'health check status is 200': (r) => r.status === 200,
  });

  // Test login endpoint
  const loginStart = Date.now();
  const loginRes = http.post(`${BASE_URL}/api/auth/login`, JSON.stringify({
    username: USERNAME,
    password: PASSWORD,
  }), {
    headers: { 'Content-Type': 'application/json' },
  });
  loginDuration.add(Date.now() - loginStart);

  check(loginRes, {
    'login status is 200': (r) => r.status === 200,
    'login has token': (r) => JSON.parse(r.body).token !== undefined,
  });

  if (loginRes.status !== 200) {
    errorRate.add(1);
    return;
  }

  // Test user list
  const usersStart = Date.now();
  const usersRes = http.get(`${BASE_URL}/api/users`, { headers });
  apiDuration.add(Date.now() - usersStart);

  check(usersRes, {
    'users list status is 200': (r) => r.status === 200,
  });
  errorRate.add(usersRes.status !== 200);

  // Test device list
  const devicesStart = Date.now();
  const devicesRes = http.get(`${BASE_URL}/api/devices`, { headers });
  apiDuration.add(Date.now() - devicesStart);

  check(devicesRes, {
    'devices list status is 200': (r) => r.status === 200,
  });
  errorRate.add(devicesRes.status !== 200);

  // Test audit logs
  const auditStart = Date.now();
  const auditRes = http.get(`${BASE_URL}/api/audit?limit=100`, { headers });
  apiDuration.add(Date.now() - auditStart);

  check(auditRes, {
    'audit logs status is 200': (r) => r.status === 200,
  });
  errorRate.add(auditRes.status !== 200);

  // Test license validation
  const licenseRes = http.post(`${BASE_URL}/api/license/validate`, JSON.stringify({
    key: 'RDPRO-TRIAL-ABC123',
  }), {
    headers: { 'Content-Type': 'application/json' },
  });

  check(licenseRes, {
    'license validate status is 200 or 400': (r) => r.status === 200 || r.status === 400,
  });

  // Simulate user think time
  sleep(1);
}

export function handleSummary(data) {
  return {
    stdout: textSummary(data, { indent: ' ', enableColors: true }),
    'summary.json': JSON.stringify(data, null, 2),
  };
}

function textSummary(data, options) {
  const { metrics } = data;
  
  let summary = '\n=== RustDesk Pro Server Performance Test Summary ===\n\n';
  
  summary += `Total Requests: ${metrics.http_reqs.values.count}\n`;
  summary += `Request Rate: ${metrics.http_reqs.values.rate.toFixed(2)} req/s\n\n`;
  
  summary += '--- HTTP Response Times ---\n';
  summary += `Average: ${(metrics.http_req_duration.values.mean * 1000).toFixed(2)} ms\n`;
  summary += `P50: ${(metrics.http_req_duration.values['p(50)'] * 1000).toFixed(2)} ms\n`;
  summary += `P95: ${(metrics.http_req_duration.values['p(95)'] * 1000).toFixed(2)} ms\n`;
  summary += `P99: ${(metrics.http_req_duration.values['p(99)'] * 1000).toFixed(2)} ms\n`;
  summary += `Max: ${(metrics.http_req_duration.values.max * 1000).toFixed(2)} ms\n\n`;
  
  summary += '--- Custom Metrics ---\n';
  summary += `Average Login Duration: ${(metrics.login_duration.values.mean * 1000).toFixed(2)} ms\n`;
  summary += `Average API Duration: ${(metrics.api_duration.values.mean * 1000).toFixed(2)} ms\n`;
  summary += `Error Rate: ${(metrics.errors.values.rate * 100).toFixed(2)}%\n`;
  
  return summary;
}
