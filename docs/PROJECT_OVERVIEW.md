# Project Overview

## Project Title

Rust TUI Traffic Producer for Tenant-Based Workload Generation

---

## 1. Summary

This project is a Rust-based terminal user interface (TUI) tool for defining tenant profiles and generating simulated traffic from those profiles.

A **tenant** represents a user, customer, application, or service that produces traffic. Each tenant can have a different traffic behavior, such as steady traffic, burst traffic, or random traffic.

The first goal of the project is to build the **traffic producer layer**. This layer will generate structured traffic events that can later be handled by a scheduler or load balancer.

This project does **not** start by building the scheduler or load balancer. Instead, it focuses on creating a clean and configurable traffic generation tool first.

---

## 2. Problem Being Solved

In real systems, different users or services do not produce traffic in the same way.

For example:

- one tenant may send traffic at a constant rate
- another tenant may send sudden bursts of traffic
- another tenant may send traffic randomly
- some tenants may have higher priority than others
- some requests may have larger payload sizes than others

A scheduler or load balancer needs realistic traffic input in order to be tested properly.

This project solves that problem by creating a tool that can define tenants and generate traffic patterns based on their profiles.

---

## 3. Main Goal

The main goal is to build a TUI tool where a user can:

- define tenant profiles
- configure traffic behavior for each tenant
- preview generated traffic
- save and load tenant profiles
- export generated traffic events

The generated traffic can later be used as input for a scheduler, load balancer, or benchmarking framework.

---

## 4. What is a Tenant?

A tenant is an entity that produces traffic.

A tenant could represent:

- a customer
- a user group
- an application
- a service
- a simulated workload source

Each tenant has a profile that describes how it behaves.

Example tenant profile:

```json
{
  "tenant_id": "tenant_a",
  "tenant_name": "Tenant A",
  "traffic_pattern": "steady",
  "requests_per_second": 10,
  "payload_size_bytes": 512,
  "priority": 1,
  "duration_seconds": 60
}
```

This means Tenant A generates steady traffic at 10 requests per second for 60 seconds.

---

## 5. What is a Traffic Pattern?

A traffic pattern defines how requests are generated over time.

The initial project will support three main traffic patterns.

### 5.1 Steady Traffic

Steady traffic means requests are produced at a constant rate.

Example:

```text
Tenant A generates 10 requests per second.
```

This is useful as a baseline workload.

### 5.2 Burst Traffic

Burst traffic means traffic suddenly increases for a short period of time.

Example:

```text
Tenant B normally has low traffic, but suddenly generates 100 requests in a burst.
```

This is useful for testing how future schedulers or load balancers respond to sudden spikes.

### 5.3 Random Traffic

Random traffic means requests arrive at unpredictable intervals.

Example:

```text
Tenant C generates between 5 and 30 requests per second randomly.
```

This is more realistic than perfectly steady traffic.

---

## 6. What is a Traffic Event?

A traffic event is one generated request from a tenant.

Example traffic event:

```json
{
  "timestamp_ms": 100,
  "tenant_id": "tenant_a",
  "request_id": 2,
  "payload_size_bytes": 512
}
```

This event means:

- the request happens at 100 milliseconds
- it belongs to Tenant A
- it is request number 2
- it has a payload size of 512 bytes

The future scheduler or load balancer can consume these events.

---

## 7. High-Level System Flow

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
User previews or generates traffic
      |
      v
Traffic generator creates traffic events
      |
      v
Events are printed or exported
      |
      v
Future scheduler/load balancer can consume them
```

---

## 8. High-Level Architecture

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

## 9. Main Components

### 9.1 TUI Layer

The TUI layer is the interactive terminal interface.

It allows the user to:

- view tenants
- create tenants
- edit tenant profiles
- delete tenants
- select traffic patterns
- preview traffic events
- save and load profiles
- quit the application

The first version can be simple. It does not need to be visually complex.

### 9.2 Tenant Profile Layer

The tenant profile layer stores tenant information.

It defines:

- tenant ID
- tenant name
- traffic pattern
- request rate
- payload size
- priority or weight
- duration

This layer should be independent from the TUI so that tenant profiles can be reused later.

### 9.3 Profile Storage Layer

The storage layer saves and loads tenant profiles.

Initial formats:

- JSON
- optionally TOML later

This allows the user to define tenant profiles once and reuse them later.

### 9.4 Traffic Generator Layer

The traffic generator reads tenant profiles and produces traffic events.

For example, if a tenant has steady traffic at 10 requests per second, the generator will create request events at regular intervals.

This layer should not depend directly on the TUI.

### 9.5 Output Layer

The output layer controls where generated traffic events go.

Initial output options:

- print to terminal
- export as JSON lines
- export as CSV

Later output options may include:

- TCP output
- UDP output
- direct scheduler input
- direct load balancer input

---

## 10. How This Relates to Existing Tools

Existing tools like `nc`/Netcat can send or listen to network traffic.

However, this project is different because it works at a higher level.

This tool understands:

- tenants
- tenant profiles
- traffic patterns
- request rates
- payload sizes
- priority/weight
- structured traffic events

So instead of just sending raw traffic, this tool creates configurable and reusable workload profiles.

---

## 11. Minimum Viable Product

The first version should include:

- Rust project setup
- basic project documentation
- basic tenant profile model
- sample tenant profiles
- simple terminal output
- traffic event format
- basic steady traffic generation

The first version does not need:

- real networking
- full scheduler
- full load balancer
- advanced UI
- complex benchmarking

---

## 12. Future Extensions

Possible future improvements include:

- full interactive TUI
- tenant creation/editing from the TUI
- profile validation
- save/load tenant profiles
- burst traffic generation
- random traffic generation
- JSON/CSV export
- TCP/UDP traffic output
- integration with a scheduler
- integration with a load balancer
- traffic visualization
- benchmarking support

---

## 13. Current Scope Reminder

The current scope is intentionally focused.

The project starts with:

```text
Rust TUI Traffic Producer
```

The project does not immediately build:

```text
Full scheduler
Full load balancer
Full distributed system
```

This keeps the project realistic and makes the first milestone easier to complete properly.

---

## 14. One-Line Explanation

This project is a Rust terminal tool that lets users define tenant traffic profiles and generate structured traffic events for future scheduler or load-balancer testing.
