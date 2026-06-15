mod model;

use model::{TenantProfile, TrafficEvent, TrafficPattern};

fn main() {
    println!("Rust TUI Traffic Producer");
    println!("-------------------------");

    let tenant = TenantProfile::new(
        "tenant_a",
        "Tenant A",
        TrafficPattern::Steady,
        10,
        512,
        1,
        60,
    );

    println!("Sample Tenant Profile:");
    println!("{:#?}", tenant);

    let event = TrafficEvent::new(
        100,
        &tenant.tenant_id,
        1,
        tenant.payload_size_bytes,
    );

    println!("\nSample Traffic Event:");
    println!("{:#?}", event);

    println!("\nJSON Output Example:");
    let json = serde_json::to_string_pretty(&event)
        .expect("Failed to serialize traffic event");

    println!("{}", json);
}