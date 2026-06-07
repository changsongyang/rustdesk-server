#!/usr/bin/env python3
"""
RustDesk Pro Server 性能基准测试脚本

用法:
    python benchmark.py --url http://localhost:8080 --users 10 --duration 60
"""

import argparse
import time
import json
import statistics
from concurrent.futures import ThreadPoolExecutor, as_completed
from dataclasses import dataclass
from typing import List, Dict, Optional
import requests

@dataclass
class BenchmarkResult:
    endpoint: str
    total_requests: int
    successful: int
    failed: int
    min_time: float
    max_time: float
    avg_time: float
    median_time: float
    p95_time: float
    p99_time: float
    requests_per_second: float

class Benchmark:
    def __init__(self, base_url: str, username: str = 'admin', password: str = 'admin123'):
        self.base_url = base_url.rstrip('/')
        self.username = username
        self.password = password
        self.token: Optional[str] = None
        self.results: List[BenchmarkResult] = []

    def login(self) -> bool:
        """登录获取令牌"""
        try:
            resp = requests.post(f'{self.base_url}/api/auth/login', json={
                'username': self.username,
                'password': self.password
            }, timeout=10)
            if resp.status_code == 200:
                self.token = resp.json().get('token')
                return True
        except Exception as e:
            print(f'登录失败: {e}')
        return False

    def make_request(self, method: str, endpoint: str, **kwargs) -> tuple:
        """发送HTTP请求并返回响应时间和状态码"""
        url = f'{self.base_url}{endpoint}'
        headers = kwargs.pop('headers', {})
        if self.token:
            headers['Authorization'] = f'Bearer {self.token}'
        
        start = time.time()
        try:
            resp = requests.request(method, url, headers=headers, timeout=10, **kwargs)
            duration = time.time() - start
            return duration, resp.status_code
        except Exception as e:
            return time.time() - start, 0

    def benchmark_endpoint(self, name: str, method: str, endpoint: str, **kwargs) -> BenchmarkResult:
        """测试单个端点"""
        times = []
        successful = 0
        failed = 0
        
        for _ in range(100):
            duration, status = self.make_request(method, endpoint, **kwargs)
            times.append(duration)
            if 200 <= status < 300:
                successful += 1
            else:
                failed += 1
        
        times.sort()
        return BenchmarkResult(
            endpoint=name,
            total_requests=len(times),
            successful=successful,
            failed=failed,
            min_time=min(times) * 1000,
            max_time=max(times) * 1000,
            avg_time=statistics.mean(times) * 1000,
            median_time=statistics.median(times) * 1000,
            p95_time=times[int(len(times) * 0.95)] * 1000,
            p99_time=times[int(len(times) * 0.99)] * 1000,
            requests_per_second=1 / statistics.mean(times) if times else 0
        )

    def run(self, workers: int = 10, duration: int = 60):
        """运行基准测试"""
        print(f'\n{"="*60}')
        print('RustDesk Pro Server 性能基准测试')
        print(f'{"="*60}')
        print(f'服务器: {self.base_url}')
        print(f'并发数: {workers}')
        print(f'持续时间: {duration}秒')
        print(f'{"="*60}\n')

        # 登录
        if not self.login():
            print('登录失败，无法继续测试')
            return

        print('登录成功!\n')

        # 定义测试端点
        endpoints = [
            ('GET /health', 'GET', '/health'),
            ('GET /api/users', 'GET', '/api/users'),
            ('GET /api/devices', 'GET', '/api/devices'),
            ('GET /api/audit', 'GET', '/api/audit?limit=100'),
            ('POST /api/license/validate', 'POST', '/api/license/validate', 
             {'json': {'key': 'TEST-KEY'}}),
        ]

        # 并发测试
        print('开始并发测试...\n')
        start_time = time.time()
        
        with ThreadPoolExecutor(max_workers=workers) as executor:
            futures = []
            for _ in range(duration):
                for name, method, endpoint, *extra in endpoints:
                    futures.append(executor.submit(
                        self.benchmark_endpoint, name, method, endpoint, *(extra or [{}])
                    ))
                time.sleep(1)

            for future in as_completed(futures):
                try:
                    result = future.result()
                    self.results.append(result)
                except Exception as e:
                    print(f'测试执行出错: {e}')

        total_time = time.time() - start_time

        # 输出结果
        self.print_results()
        self.export_results()

    def print_results(self):
        """打印测试结果"""
        print(f'\n{"="*60}')
        print('测试结果')
        print(f'{"="*60}\n')

        for result in self.results:
            print(f'端点: {result.endpoint}')
            print(f'  总请求数: {result.total_requests}')
            print(f'  成功率: {result.successful / result.total_requests * 100:.2f}%')
            print(f'  失败数: {result.failed}')
            print(f'  最小响应时间: {result.min_time:.2f} ms')
            print(f'  最大响应时间: {result.max_time:.2f} ms')
            print(f'  平均响应时间: {result.avg_time:.2f} ms')
            print(f'  中位数响应时间: {result.median_time:.2f} ms')
            print(f'  P95响应时间: {result.p95_time:.2f} ms')
            print(f'  P99响应时间: {result.p99_time:.2f} ms')
            print(f'  QPS: {result.requests_per_second:.2f}')
            print()

    def export_results(self):
        """导出结果到JSON文件"""
        data = {
            'timestamp': time.strftime('%Y-%m-%d %H:%M:%S'),
            'base_url': self.base_url,
            'results': [
                {
                    'endpoint': r.endpoint,
                    'total_requests': r.total_requests,
                    'successful': r.successful,
                    'failed': r.failed,
                    'min_time_ms': round(r.min_time, 2),
                    'max_time_ms': round(r.max_time, 2),
                    'avg_time_ms': round(r.avg_time, 2),
                    'median_time_ms': round(r.median_time, 2),
                    'p95_time_ms': round(r.p95_time, 2),
                    'p99_time_ms': round(r.p99_time, 2),
                    'requests_per_second': round(r.requests_per_second, 2),
                }
                for r in self.results
            ]
        }
        
        filename = f'benchmark_results_{int(time.time())}.json'
        with open(filename, 'w', encoding='utf-8') as f:
            json.dump(data, f, indent=2, ensure_ascii=False)
        print(f'结果已导出到: {filename}')

def main():
    parser = argparse.ArgumentParser(description='RustDesk Pro Server 性能基准测试')
    parser.add_argument('--url', default='http://localhost:8080', help='服务器URL')
    parser.add_argument('--username', default='admin', help='用户名')
    parser.add_argument('--password', default='admin123', help='密码')
    parser.add_argument('--workers', type=int, default=10, help='并发数')
    parser.add_argument('--duration', type=int, default=60, help='持续时间(秒)')

    args = parser.parse_args()

    benchmark = Benchmark(args.url, args.username, args.password)
    benchmark.run(workers=args.workers, duration=args.duration)

if __name__ == '__main__':
    main()
