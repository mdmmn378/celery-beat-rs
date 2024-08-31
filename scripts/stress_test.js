import http from 'k6/http';
import { check, sleep } from 'k6';
import { Counter } from 'k6/metrics';

// Custom metrics
export let errorCount = new Counter('errors');

export let options = {
    stages: [
        { duration: '30s', target: 500 }, // Ramp up to 50 users
        // { duration: '1m', target: 50 },  // Stay at 50 users
        // { duration: '30s', target: 0 },  // Ramp down to 0 users
    ],
};

export default function () {
    let healthRes = http.get('http://localhost:8080/api/v1/health/');
    check(healthRes, {
        'status is 200': (r) => r.status === 200,
    }) || errorCount.add(1);

    // let submitTaskRes = http.post('http://localhost:8080/api/v1/submit-task', JSON.stringify({
    //     "task_name": "src-py.main.add",
    //     "args": [100, 212],
    //     "kwargs": { "100": 2, "name": "mamun" }
    // }), {
    //     headers: { 'Content-Type': 'application/json' },
    // });

    // check(submitTaskRes, {
    //     'status is 200': (r) => r.status === 200,
    // }) || errorCount.add(1);

    sleep(0.001);
}
