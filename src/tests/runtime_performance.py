"""Measure real stdio round trips against a disposable local checkout and DB.

Run from the repository using its pinned rustc:
    python src/tests/runtime_performance.py RUNTIME_EXE --trace TRACE_JSON
Use --exercise for cancellation, overload and disconnect checks on a new runtime.
Use --soak for 1,000 read/cancel pairs and 100 reconnects after warm-up.
"""
import argparse
from contextlib import closing
import hashlib
import json
import os
from pathlib import Path
import queue
import re
import shutil
import sqlite3
import statistics
import subprocess
import tempfile
import threading
import time


GIT_GATE = r"""
use std::{env, fs, path::Path, process::Command, thread, time::{Duration, Instant}};

fn main() {
    let entered = Path::new(".git/runtime-perf-entered");
    let release = Path::new(".git/runtime-perf-release");
    if !entered.exists() {
        fs::write(entered, b"").unwrap();
        let until = Instant::now() + Duration::from_secs(3);
        while !release.exists() && Instant::now() < until {
            thread::sleep(Duration::from_millis(5));
        }
    }
    let result = Command::new(env!("CIRNET_PERF_GIT"))
        .args(env::args_os().skip(1))
        .status().unwrap();
    std::process::exit(result.code().unwrap_or(1));
}
"""

TRACE = re.compile(
    r"^cirnetrules-ipc id=(?P<id>[a-z0-9-]+) method=(?P<method>[a-z.]+)"
    r" queued_us=(?P<queue_us>\d+) dispatch_us=(?P<dispatch_us>\d+)"
    r"(?: storage_pending=\d+ serialize_us=(?P<serialize_us>\d+)"
    r" egress_us=(?P<egress_us>\d+) response_bytes=(?P<response_bytes>\d+))?"
)


class Runtime:
    def __init__(self, binary, data, wrapper_dir, read_stdout=True):
        env = {
            **os.environ,
            "PATH": str(wrapper_dir) + os.pathsep + os.environ["PATH"],
            "CIRNETRULES_TRACE_IPC": "1",
        }
        self.child = subprocess.Popen(
            [binary, "--stdio", "--data-dir", data], env=env,
            stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
        )
        self.completions = queue.Queue()
        self.sent = {}
        self.trace = []
        self.pipe_stalled = False
        self.reader = None
        self.tracer = threading.Thread(target=self.drain_trace)
        self.tracer.start()
        if read_stdout:
            self.resume_stdout()

    def __enter__(self):
        return self

    def __exit__(self, exc_type, _exc, _traceback):
        try:
            if exc_type is None and self.child.poll() is None:
                self.child.stdin.close()
                assert self.child.wait(timeout=12) == 0, "Runtime failed during EOF drain"
        finally:
            if self.child.poll() is None:
                self.child.kill()
                self.child.wait(timeout=12)
            if self.reader:
                self.reader.join()
            self.tracer.join()
            for stream in (self.child.stdin, self.child.stdout, self.child.stderr):
                stream.close()

    def resume_stdout(self):
        assert self.reader is None
        self.reader = threading.Thread(target=self.drain_stdout)
        self.reader.start()

    def drain_stdout(self):
        for line in self.child.stdout:
            self.completions.put((time.perf_counter(), json.loads(line)))

    def drain_trace(self):
        for line in self.child.stderr:
            if match := TRACE.match(line.decode("ascii", "ignore")):
                fields = match.groupdict()
                self.trace.append(fields)
            self.pipe_stalled |= b"Operator response pipe stalled" in line

    def send(self, rid, method, params=None):
        self.sent[rid] = time.perf_counter()
        frame = {"v": 3, "id": rid, "method": "cirnetrules." + method, "params": params or {}}
        self.child.stdin.write((json.dumps(frame) + "\n").encode())
        self.child.stdin.flush()

    def receive(self, rid=None):
        arrived, wire = self.completions.get(timeout=12)
        if rid is not None:
            assert rid == wire["id"], "Unexpected response ID"
        elapsed = (arrived - self.sent.pop(wire["id"])) * 1000
        return elapsed, wire

    def call(self, rid, method, params=None):
        self.send(rid, method, params)
        _, wire = self.receive(rid)
        assert wire["ok"], (method, wire["error"]["code"])
        return wire["result"]


def git_fixture(git, root):
    wrapper = root / "wrapper"
    wrapper.mkdir()
    source = wrapper / "git_fixture.rs"
    source.write_text(GIT_GATE, encoding="utf-8")
    subprocess.run(
        ["rustc", "--edition=2024", "-O", source, "-o", wrapper / git.name],
        env={**os.environ, "CIRNET_PERF_GIT": str(git)}, check=True,
    )
    repo = root / "checkout"
    repo.mkdir()
    subprocess.run([git, "init", "--quiet"], cwd=repo, check=True)
    subprocess.run(
        [git, "-c", "user.name=Fixture", "-c", "user.email=fixture@example.invalid",
         "-c", "commit.gpgSign=false", "commit", "--allow-empty", "--quiet", "-m", "Fixture"],
        cwd=repo, check=True,
    )
    return repo, wrapper


def gate_paths(repo):
    marker = repo / ".git" / "runtime-perf-entered"
    release = repo / ".git" / "runtime-perf-release"
    return marker, release


def enter_gate(runtime, repo, rid):
    marker, release = gate_paths(repo)
    marker.unlink(missing_ok=True)
    release.unlink(missing_ok=True)
    runtime.send(rid, "repo.preview", {"path": str(repo)})
    deadline = time.perf_counter() + 4
    while not marker.exists() and time.perf_counter() < deadline:
        time.sleep(.002)
    assert marker.exists(), "Runtime never entered the gated Git command"
    return release


def measure(binary, data, repo, wrapper, rounds, delay):
    gate_paths(repo)[1].touch()
    samples = []
    with Runtime(binary, data, wrapper) as runtime:
        token = runtime.call("setup-preview", "repo.preview", {"path": str(repo)})["token"]
        binding = runtime.call("setup-add", "repo.add", {"path": str(repo), "token": token})["binding_id"]
        cid = runtime.call("setup-chat", "conversation.create", {
            "key": "fixture-key", "title": "Fixture", "bindings": [binding],
        })["id"]
        runtime.call("setup-get", "conversation.get", {"id": cid})
        trace_ids = {}
        for index in range(rounds):
            requests = {
                f"slow-{index}": ("repo.preview", None),
                f"get-{index}": ("conversation.get", {"id": cid}),
                f"info-{index}": ("runtime.info", {}),
                f"control-{index}": ("operation.get", {"operation_id": "fixture"}),
            }
            slow = next(iter(requests))
            release = enter_gate(runtime, repo, slow)
            for rid, (method, params) in requests.items():
                trace_ids[rid] = trace_ids[hashlib.sha256(rid.encode()).hexdigest()[:16]] = rid
                if rid != slow:
                    runtime.send(rid, method, params)
            time.sleep(delay)
            release.touch()
            for position in range(4):
                elapsed, wire = runtime.receive()
                rid = wire["id"]
                assert rid in requests, "Unexpected response ID"
                if rid.startswith("get-"):
                    assert wire["ok"] and wire["result"]["conversation"]["id"] == cid
                elif rid.startswith("control-"):
                    assert not wire["ok"] and wire["error"]["code"] == "INVALID_INPUT"
                else:
                    assert wire["ok"]
                samples.append({"id": rid, "elapsed_ms": round(elapsed, 3), "reply_position": position + 1})
        time.sleep(.05)  # Allow asynchronous diagnostics to drain.
    timings = [{**fields, "id": trace_ids[fields["id"]]}
               for fields in runtime.trace if fields["id"] in trace_ids]
    assert len(timings) == 4 * rounds, "Trace sample count is incomplete"
    return cid, samples, timings


def exercise(binary, data, repo, wrapper, cid):
    db_path = data / "cirnetrules.sqlite3"
    with closing(sqlite3.connect(db_path)) as db:
        with db:
            db.executemany(
                "INSERT INTO messages VALUES(NULL,?,?,'fixture-digest',?,'USER',?,1)",
                [(cid, f"large-{i}", i, "x" * 16_000) for i in range(1, 5)],
            )
    with Runtime(binary, data, wrapper, read_stdout=False) as runtime:
        release = enter_gate(runtime, repo, "running-read")
        # Hold stdout while a large response is being written. Submit control
        # requests before reading it and keep the Git gate closed.
        runtime.send("large-read", "conversation.get", {"id": cid})
        runtime.send("queued-read", "repo.preview", {"path": str(repo)})
        for i in range(10):
            runtime.send(f"burst-{i}", "repo.preview", {"path": str(repo)})
        runtime.send("cancel-queued", "request.cancel", {"id": "queued-read"})
        runtime.send("cancel-running", "request.cancel", {"id": "running-read"})
        time.sleep(.25)
        runtime.resume_stdout()
        replies = {}
        for _ in range(len(runtime.sent)):
            _, wire = runtime.receive()
            replies[wire["id"]] = wire
        assert replies["cancel-queued"]["result"]["state"] == "CANCELLED"
        assert replies["cancel-running"]["result"]["state"] == "CANCELLATION_REQUESTED"
        for rid in ("running-read", "queued-read"):
            assert replies[rid]["error"]["code"] == "REQUEST_CANCELLED"
        assert replies["large-read"]["ok"]
        overloaded = sum(reply.get("error", {}).get("code") == "REQUEST_LIMIT"
                         for reply in replies.values())
        assert overloaded > 0, "Burst did not exercise bounded lane admission"
        assert not release.exists()
        release.touch()

    mutation = {"id": cid, "key": "receipt-fixture", "content": "Fixture message"}
    with Runtime(binary, data, wrapper, read_stdout=False) as runtime:
        runtime.send("blocked-stdout", "conversation.get", {"id": cid})
        time.sleep(.25)
        runtime.send("unread-mutation", "conversation.send", mutation)
        assert runtime.child.wait(timeout=8) != 0, "Runtime accepted indefinitely stalled stdout"
    assert runtime.pipe_stalled, "Runtime did not report its stall deadline"
    with closing(sqlite3.connect(db_path)) as db:
        first = db.execute("SELECT id FROM messages WHERE request_key=?", (mutation["key"],)).fetchall()
    assert len(first) == 1, "Writer did not persist its result before transport shutdown"
    with Runtime(binary, data, wrapper) as runtime:
        replay = runtime.call("retry-mutation", "conversation.send", mutation)
        assert replay["message"]["id"] == str(first[0][0])
    with closing(sqlite3.connect(db_path)) as db:
        assert first == db.execute(
            "SELECT id FROM messages WHERE request_key=?", (mutation["key"],)
        ).fetchall()
    return {
        "queued_cancel": True, "running_git_cancel": True, "lane_overload_replies": overloaded,
        "slow_stdout_disconnect": True, "same_key_recovers_unread_mutation": True,
    }


def process_resources(pid):
    if os.name != "nt":
        return {}
    command = (
        f"$p = Get-Process -Id {pid}; "
        "[pscustomobject]@{private_bytes=$p.PrivateMemorySize64; "
        "working_set_bytes=$p.WorkingSet64; handles=$p.HandleCount; "
        "threads=$p.Threads.Count} | ConvertTo-Json -Compress"
    )
    return json.loads(subprocess.check_output(
        ["powershell", "-NoProfile", "-Command", command], text=True,
        timeout=8,
    ))


def cancel_read(runtime, cid):
    runtime.send("soak-read", "conversation.get", {"id": cid})
    runtime.send("soak-cancel", "request.cancel", {"id": "soak-read"})
    replies = {}
    for _ in range(2):
        _, wire = runtime.receive()
        replies[wire["id"]] = wire
    state = replies["soak-cancel"]["result"]["state"]
    read = replies["soak-read"]
    if state == "NOT_PENDING":
        assert read["ok"] and read["result"]["conversation"]["id"] == cid
    else:
        assert state in {"CANCELLED", "CANCELLATION_REQUESTED"}
        assert read["error"]["code"] == "REQUEST_CANCELLED"
    return state


def soak(binary, data, wrapper, first):
    samples = []
    counts = {"CANCELLED": 0, "CANCELLATION_REQUESTED": 0, "NOT_PENDING": 0}
    with Runtime(binary, data, wrapper) as runtime:
        binding = runtime.call("soak-binding", "repo.list")["items"][0]["binding_id"]
        second = runtime.call("soak-second", "conversation.create", {
            "key": "fixture-second", "title": "Other fixture", "bindings": [binding],
        })["id"]
        assert first != second
        for i in range(100):
            cancel_read(runtime, (first, second)[i % 2])
        for i in range(1001):
            if i in (0, 250, 500, 750, 1000):
                time.sleep(.15)
                samples.append({"pairs": i, **process_resources(runtime.child.pid)})
            if i < 1000:
                counts[cancel_read(runtime, (first, second)[i % 2])] += 1
    for i in range(100):
        with Runtime(binary, data, wrapper) as runtime:
            cancel_read(runtime, (first, second)[i % 2])
    return {
        "warmup_pairs": 100, "pairs": 1000, "reconnects": 100,
        "read_cancel_states": counts, "runtime_process_samples": samples,
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("runtime", type=Path)
    parser.add_argument("--trace", type=Path, required=True)
    parser.add_argument("--iterations", type=int, default=10)
    parser.add_argument("--delay", type=float, default=1.0)
    parser.add_argument("--git", type=Path, help="Real Git binary, outside the disposable fixture")
    parser.add_argument("--exercise", action="store_true")
    parser.add_argument("--soak", action="store_true")
    args = parser.parse_args()
    if not (1 <= args.iterations <= 100 and 0 < args.delay <= 2):
        parser.error("Choose 1–100 rounds and a gate delay of at most 2 seconds.")
    binary = args.runtime.resolve(strict=True)
    git = args.git or Path(shutil.which("git") or "")
    if git.name.lower() not in {"git", "git.exe"}:
        parser.error("Provide the real Git executable with --git.")
    git = git.resolve(strict=True)
    if os.name == "nt":
        native_git = git.parent.parent / "mingw64" / "bin" / "git.exe"
        if native_git.is_file():
            git = native_git

    project = Path(__file__).resolve().parents[2]
    scratch = project / "target"
    scratch.mkdir(exist_ok=True)
    with tempfile.TemporaryDirectory(dir=scratch, prefix="runtime-perf-") as directory:
        root = Path(directory)
        data = root / "storage"
        repo, wrapper = git_fixture(git, root)
        cid, rows, trace = measure(binary, data, repo, wrapper, args.iterations, args.delay)
        checks = exercise(binary, data, repo, wrapper, cid) if args.exercise else {}
        resources = soak(binary, data, wrapper, cid) if args.soak else {}
    result = {
        "sha256": hashlib.sha256(binary.read_bytes()).hexdigest(), "gate_ms": args.delay * 1000,
        "iterations": args.iterations, "rows": rows, "trace": trace,
        "checks": checks, "soak": resources,
    }
    args.trace.write_text(json.dumps(result, indent=2) + "\n", encoding="utf-8")
    for tag in ("slow", "get", "info", "control"):
        elapsed = sorted(row["elapsed_ms"] for row in rows if row["id"].startswith(tag + "-"))
        p95, p99 = (elapsed[(len(elapsed) * percentile + 99) // 100 - 1] for percentile in (95, 99))
        wait = [int(row["queue_us"]) / 1000 for row in trace if row["id"].startswith(tag + "-")]
        dispatch = [int(row["dispatch_us"]) / 1000 for row in trace if row["id"].startswith(tag + "-")]
        print(f"{tag:7s} RTT p50/p95/p99/max {statistics.median(elapsed):.3f}/{p95:.3f}/"
              f"{p99:.3f}/{max(elapsed):.3f} ms  queue/dispatch p50 "
              f"{statistics.median(wait):.3f}/{statistics.median(dispatch):.3f} ms")
    if checks:
        print(json.dumps(checks))
    if resources:
        print(json.dumps(resources))


if __name__ == "__main__":
    main()
