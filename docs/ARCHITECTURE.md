# Architecture Design

## Project Title

Rust TUI Traffic Producer for Tenant-Based Workload Generation

## 1. Project Overview

The goal of this project is to build a Rust-based terminal user interface (TUI) tool that allows users to define tenant profiles and generate traffic patterns from those profiles.

Each tenant represents a source of traffic. A tenant profile defines how that tenant behaves, such as its request rate, traffic pattern, payload size, and priority/weight.

The first version of the project will focus on creating, viewing, editing, saving, loading, and previewing tenant traffic profiles. The generated traffic will be exported in a structured format such as JSON lines or CSV so that it can later be consumed by a scheduler or load balancer.

This project is not starting with the scheduler/load balancer implementation. Instead, it focuses on building the traffic producer layer first.

---

## 2. High-Level Architecture

```text
+----------------------+
|      Rust TUI App    |
|  Terminal Interface  |
+----------+-----------+
           |
           v
+----------------------+
| Tenant Profile Layer |
| Create/Edit Tenants  |
+----------+-----------+
           |
           v
+----------------------+
|  Profile Storage     |
| Save/Load JSON/TOML  |
+----------+-----------+
           |
           v
+----------------------+
| Traffic Generator    |
| Steady/Burst/Random  |
+----------+-----------+
           |
           v
+----------------------+
| Output Layer         |
| JSON Lines / CSV     |
+----------+-----------+
           |
           v
+-----------------------------+
| Future Scheduler / Load     |
| Balancer Integration        |
+-----------------------------+
```

---

## 3. Main Components

### 3.1 TUI Layer

The TUI layer is the user-facing part of the application.

It allows the user to:

* view tenant profiles
* create a new tenant
* edit an existing tenant
* delete a tenant
* choose a traffic pattern
* preview generated traffic
* save and load profiles
* quit the application safely

The TUI should be keyboard-driven and simple to use.

Example screens:

```text
[ Tenants ] [ Edit Profile ] [ Traffic Preview ] [ Settings ]
```

The first version does not need to be visually complex. It only needs to provide a clean way to interact with tenant profiles.

---

### 3.2 Tenant Profile Layer

The tenant profile layer stores the core data model of the application.

Each tenant profile may contain:

```text
tenant_id
tenant_name
traffic_pattern
requests_per_second
burst_size
payload_size_bytes
priority_or_weight
duration_seconds
```

Example tenant profile:

```json
{
  "tenant_id": "tenant_a",
  "tenant_name": "Tenant A",
  "traffic_pattern": "steady",
  "requests_per_second": 10,
  "payload_size_bytes": 512,
  "priority_or_weight": 1,
  "duration_seconds": 60
}
```

This layer should be independent from the TUI so that tenant profiles can later be reused by other parts of the system.

---

### 3.3 Profile Storage Layer

The storage layer is responsible for saving and loading tenant profiles.

Initial supported formats:

* JSON
* optionally TOML later

The storage layer should allow:

* saving all tenant profiles to a file
* loading tenant profiles from a file
* validating loaded profiles

Example file:

```text
profiles/tenants.json
```

This makes the tool useful because users can define a set of tenants once and reuse them later.

---

### 3.4 Traffic Generator Layer

The traffic generator layer converts tenant profiles into simulated traffic events.

Initial traffic patterns:

1. Steady traffic
   A tenant generates requests at a constant rate.

2. Burst traffic
   A tenant generates a sudden spike of requests during a short time period.

3. Random traffic
   A tenant generates requests at random intervals within a configured range.

Example generated event:

```json
{
  "timestamp_ms": 0,
  "tenant_id": "tenant_a",
  "request_id": 1,
  "payload_size_bytes": 512
}
```

The traffic generator should not depend directly on the TUI. It should receive tenant profiles as input and produce traffic events as output.

---

### 3.5 Output Layer

The output layer is responsible for exporting generated traffic events.

Initial output formats:

* stdout
* JSON lines
* CSV

Example JSON lines output:

```json
{"timestamp_ms":0,"tenant_id":"tenant_a","request_id":1,"payload_size_bytes":512}
{"timestamp_ms":100,"tenant_id":"tenant_a","request_id":2,"payload_size_bytes":512}
{"timestamp_ms":200,"tenant_id":"tenant_a","request_id":3,"payload_size_bytes":512}
```

This output can later be connected to a scheduler or load balancer.

---

### 3.6 Future Scheduler / Load Balancer Boundary

The scheduler and load balancer are outside the first implementation scope.

However, the producer should be designed so that generated traffic events can later be passed into a scheduler/load balancer.

For now, the integration boundary will be:

```text
TrafficEvent output
```

This means the traffic producer only needs to generate structured traffic events. Another future component can consume those events.

---

## 4. Data Flow

```text
User opens TUI
      |
      v
User creates or edits tenant profiles
      |
      v
Profiles are stored in memory
      |
      v
User saves profiles to JSON
      |
      v
User selects preview/generate traffic
      |
      v
Traffic generator creates traffic events
      |
      v
Output layer writes events to stdout, JSON lines, or CSV
```

---

## 5. Suggested Rust Module Structure

```text
src/
├── main.rs
├── app.rs
├── model/
│   ├── mod.rs
│   ├── tenant.rs
│   └── traffic.rs
├── tui/
│   ├── mod.rs
│   ├── screens.rs
│   └── events.rs
├── storage/
│   ├── mod.rs
│   └── profile_store.rs
├── generator/
│   ├── mod.rs
│   └── traffic_generator.rs
└── output/
    ├── mod.rs
    └── json_output.rs
```

---

## 6. MVP Scope

The first minimum version should include:

* basic Rust project setup
* terminal app starts successfully
* hardcoded sample tenant profiles
* TUI displays tenant list
* user can navigate tenant list
* user can quit using `q`
* tenant data model is defined
* architecture documentation is completed

---

## 7. Next Milestones

### Milestone 1: Architecture and Codebase Setup

* Create project architecture document
* Create Rust project structure
* Define module layout
* Add initial tenant and traffic event models

### Milestone 2: Basic TUI

* Launch terminal UI
* Display tenant profiles
* Add keyboard navigation
* Add quit functionality

### Milestone 3: Tenant Profile Management

* Add tenant creation
* Add tenant editing
* Add tenant deletion
* Add validation

### Milestone 4: Save and Load Profiles

* Save tenant profiles to JSON
* Load tenant profiles from JSON
* Handle invalid profile files

### Milestone 5: Traffic Generation

* Generate steady traffic
* Generate burst traffic
* Generate random traffic
* Preview generated traffic in the TUI

### Milestone 6: Output Export

* Export generated traffic as JSON lines
* Export generated traffic as CSV
* Prepare output format for future scheduler/load balancer integration

---

## 8. Design Principles

The project should follow these principles:

* Keep the TUI separate from the traffic generation logic
* Keep tenant profiles reusable and easy to serialize
* Start with simple traffic patterns before adding complexity
* Export structured traffic events for future integration
* Avoid implementing the scheduler/load balancer too early
* Keep the first version small, testable, and demo-friendly
